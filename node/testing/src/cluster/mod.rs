mod config;
pub use config::ClusterConfig;

mod p2p_task_spawner;
pub use p2p_task_spawner::P2pTaskSpawner;

mod node_id;
pub use node_id::ClusterNodeId;

use std::time::Duration;
use std::{collections::VecDeque, sync::Arc};

use ledger::proofs::{VerifierIndex, VerifierSRS};
use node::core::channels::mpsc;
use node::core::requests::RpcId;
use node::{
    event_source::Event,
    ledger::LedgerCtx,
    p2p::{
        channels::ChannelId,
        identity::SecretKey as P2pSecretKey,
        service_impl::{
            webrtc::P2pServiceCtx,
            webrtc_with_libp2p::{self, P2pServiceWebrtcWithLibp2p},
        },
        P2pEvent,
    },
    service::Recorder,
    snark::{get_srs, get_verifier_index, VerifierKind},
    BuildEnv, Config, GlobalConfig, LedgerConfig, P2pConfig, SnarkConfig, TransitionFrontierConfig,
};
use openmina_node_native::{http_server, rpc::RpcService, NodeService, RpcSender};
use rand::{rngs::StdRng, SeedableRng};
use serde::Serialize;

use crate::{
    node::{Node, NodeTestingConfig, RustNodeTestingConfig},
    scenario::{ListenerNode, Scenario, ScenarioId, ScenarioStep},
    service::{NodeTestingService, PendingEventId},
};

pub struct Cluster {
    pub config: ClusterConfig,
    scenario: ClusterScenarioRun,
    available_ports: Box<dyn Iterator<Item = u16> + Send>,
    nodes: Vec<Node>,

    rpc_counter: usize,

    verifier_srs: Arc<VerifierSRS>,
    block_verifier_index: Arc<VerifierIndex>,
    work_verifier_index: Arc<VerifierIndex>,
}

#[derive(Serialize)]
pub struct ClusterScenarioRun {
    chain: VecDeque<Scenario>,
    finished: Vec<Scenario>,
    cur_step: usize,
}

impl Cluster {
    pub fn new(config: ClusterConfig) -> Self {
        let available_ports = config
            .port_range()
            .filter(|port| std::net::TcpListener::bind(("0.0.0.0", *port)).is_ok());
        Self {
            config,
            scenario: ClusterScenarioRun {
                chain: Default::default(),
                finished: Default::default(),
                cur_step: 0,
            },
            available_ports: Box::new(available_ports),
            nodes: vec![],

            rpc_counter: 0,

            verifier_srs: get_srs().into(),
            block_verifier_index: get_verifier_index(VerifierKind::Blockchain).into(),
            work_verifier_index: get_verifier_index(VerifierKind::Transaction).into(),
        }
    }

    pub fn add_node(&mut self, testing_config: NodeTestingConfig) -> ClusterNodeId {
        match testing_config {
            NodeTestingConfig::Rust(testing_config) => self.add_rust_node(testing_config),
        }
    }

    pub fn add_rust_node(&mut self, testing_config: RustNodeTestingConfig) -> ClusterNodeId {
        let node_i = self.nodes.len();
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        let secret_key = {
            let mut bytes = [0; 32];
            let bytes_len = bytes.len();
            let i_bytes = node_i.to_be_bytes();
            let i = bytes_len - i_bytes.len();
            bytes[i..bytes_len].copy_from_slice(&i_bytes);
            P2pSecretKey::from_bytes(bytes)
        };
        let pub_key = secret_key.public_key();

        let config = Config {
            ledger: LedgerConfig {},
            snark: SnarkConfig {
                // TODO(binier): use cache
                block_verifier_index: self.block_verifier_index.clone(),
                block_verifier_srs: self.verifier_srs.clone(),
                work_verifier_index: self.work_verifier_index.clone(),
                work_verifier_srs: self.verifier_srs.clone(),
            },
            global: GlobalConfig {
                build: BuildEnv::get().into(),
                snarker: None,
            },
            p2p: P2pConfig {
                identity_pub_key: pub_key,
                initial_peers: vec![],
                max_peers: 100,
                enabled_channels: ChannelId::iter_all().collect(),
            },
            transition_frontier: TransitionFrontierConfig::default(),
        };

        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let (p2p_event_sender, mut rx) = mpsc::unbounded_channel::<P2pEvent>();

        let webrtc_with_libp2p::P2pServiceCtx {
            libp2p,
            webrtc: P2pServiceCtx { cmd_sender, peers },
        } = <NodeService as P2pServiceWebrtcWithLibp2p>::init(
            secret_key,
            testing_config.chain_id,
            p2p_event_sender.clone(),
            P2pTaskSpawner::new(shutdown_tx.clone()),
        );

        let ev_sender = event_sender.clone();
        tokio::spawn(async move {
            while let Some(v) = rx.recv().await {
                if let Err(_) = ev_sender.send(v.into()) {
                    break;
                }
            }
        });

        let mut rpc_service = RpcService::new();

        let http_port = self
            .available_ports
            .next()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "couldn't find available port in port range: {:?}",
                    self.config.port_range()
                )
            })
            .unwrap();
        let rpc_sender = RpcSender::new(rpc_service.req_sender().clone());

        // spawn http-server
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let shutdown = shutdown_tx.clone();
        std::thread::Builder::new()
            .name("openmina_http_server".to_owned())
            .spawn(move || {
                let local_set = tokio::task::LocalSet::new();
                let task = async {
                    tokio::select! {
                        _ = shutdown.closed() => {}
                        _ = http_server::run(http_port, rpc_sender) => {}
                    }
                };
                local_set.block_on(&runtime, task);
            })
            .unwrap();

        let ledger = LedgerCtx::default();
        let real_service = NodeService {
            rng: StdRng::seed_from_u64(0),
            event_sender,
            p2p_event_sender,
            event_receiver: event_receiver.into(),
            cmd_sender,
            ledger,
            peers,
            libp2p,
            rpc: rpc_service,
            snark_worker_sender: None,
            stats: node::stats::Stats::new(),
            recorder: Recorder::None,
            replayer: None,
        };
        let service = NodeTestingService::new(real_service, http_port, shutdown_rx);
        let state = node::State::new(config);
        fn effects<S: node::Service>(store: &mut node::Store<S>, action: node::ActionWithMeta) {
            let peer_id = store.state().p2p.config.identity_pub_key.peer_id();
            eprintln!("{peer_id}: {:?}", action.action().kind());
            node::effects(store, action)
        }
        let store = node::Store::new(
            node::reducer,
            effects,
            service,
            testing_config.initial_time.into(),
            state,
        );
        let node = Node::new(store);

        self.nodes.push(node);
        ClusterNodeId::new_unchecked(node_i)
    }

    pub async fn start(&mut self, scenario: Scenario) -> Result<(), anyhow::Error> {
        let mut parent_id = scenario.info.parent_id.clone();
        self.scenario.chain.push_back(scenario);

        while let Some(id) = parent_id {
            let scenario = Scenario::load(&id).await?;
            parent_id = scenario.info.parent_id.clone();
            self.scenario.chain.push_back(scenario);
        }

        let scenario = self.scenario.cur_scenario();

        for config in scenario.info.nodes.clone() {
            self.add_node(config.clone());
        }

        Ok(())
    }

    pub async fn reload_scenarios(&mut self) -> Result<(), anyhow::Error> {
        for scenario in &mut self.scenario.chain {
            scenario.reload().await?;
        }
        Ok(())
    }

    pub fn next_scenario_and_step(&self) -> Option<(&ScenarioId, usize)> {
        self.scenario
            .peek_i()
            .map(|(scenario_i, step_i)| (&self.scenario.chain[scenario_i].info.id, step_i))
    }

    pub fn target_scenario(&self) -> Option<&ScenarioId> {
        self.scenario.target_scenario().map(|v| &v.info.id)
    }

    pub fn pending_events(
        &mut self,
    ) -> impl Iterator<
        Item = (
            ClusterNodeId,
            impl Iterator<Item = (PendingEventId, &Event)>,
        ),
    > {
        self.nodes
            .iter_mut()
            .enumerate()
            .map(|(i, node)| (ClusterNodeId::new_unchecked(i), node.pending_events()))
    }

    pub fn node_pending_events(
        &mut self,
        node_id: ClusterNodeId,
    ) -> Result<impl Iterator<Item = (PendingEventId, &Event)>, anyhow::Error> {
        let node = self
            .nodes
            .get_mut(node_id.index())
            .ok_or(anyhow::anyhow!("node {node_id:?} not found"))?;
        Ok(node.pending_events())
    }

    pub async fn exec_to_end(&mut self) -> Result<(), anyhow::Error> {
        loop {
            if !self.exec_next().await? {
                break Ok(());
            }
        }
    }

    pub async fn exec_until(
        &mut self,
        target_scenario: ScenarioId,
        step_i: Option<usize>,
    ) -> Result<(), anyhow::Error> {
        if self
            .scenario
            .finished
            .iter()
            .any(|v| v.info.id == target_scenario)
        {
            return Err(anyhow::anyhow!(
                "cluster already finished '{target_scenario}' scenario"
            ));
        }

        while self
            .scenario
            .peek()
            .map_or(false, |(scenario, _)| scenario.info.id != target_scenario)
        {
            if !self.exec_next().await? {
                break;
            }
        }

        while self
            .scenario
            .peek()
            .map_or(false, |(scenario, _)| scenario.info.id == target_scenario)
        {
            if let Some(step_i) = step_i {
                if self.scenario.peek_i().unwrap().1 >= step_i {
                    break;
                }
            }
            if !self.exec_next().await? {
                break;
            }
        }

        Ok(())
    }

    pub async fn exec_next(&mut self) -> Result<bool, anyhow::Error> {
        let (_scenario, step) = match self.scenario.peek() {
            Some(v) => v,
            None => return Ok(false),
        };
        let dispatched = match step {
            ScenarioStep::ManualEvent { node_id, event } => {
                let node = self
                    .nodes
                    .get_mut(node_id.index())
                    .ok_or(anyhow::anyhow!("node {node_id:?} not found"))?;
                node.dispatch_event((**event).clone())
            }
            ScenarioStep::Event { node_id, event } => {
                let node = self
                    .nodes
                    .get_mut(node_id.index())
                    .ok_or(anyhow::anyhow!("node {node_id:?} not found"))?;
                let timeout = tokio::time::sleep(Duration::from_secs(5));
                tokio::select! {
                    res = node.wait_for_event_and_dispatch(&event) => res,
                    _ = timeout => {
                        return Err(anyhow::anyhow!("waiting for event timed out! node {node_id:?}, event: \"{event}\""));
                    }
                }
            }
            ScenarioStep::ConnectNodes { dialer, listener } => {
                let listener_addr = match listener {
                    ListenerNode::Rust(listener) => {
                        let listener = self
                            .nodes
                            .get_mut(listener.index())
                            .ok_or(anyhow::anyhow!("node {listener:?} not found"))?;

                        listener.dial_addr()
                    }
                    ListenerNode::Custom(addr) => addr.clone(),
                };

                self.rpc_counter += 1;
                let rpc_id = RpcId::new_unchecked(usize::MAX, self.rpc_counter);
                let dialer = self
                    .nodes
                    .get_mut(dialer.index())
                    .ok_or(anyhow::anyhow!("node {dialer:?} not found"))?;

                let req = node::rpc::RpcRequest::P2pConnectionOutgoing(listener_addr);
                dialer.dispatch_event(Event::Rpc(rpc_id, req))
            }
            ScenarioStep::CheckTimeouts { node_id } => {
                let node = self
                    .nodes
                    .get_mut(node_id.index())
                    .ok_or(anyhow::anyhow!("node {node_id:?} not found"))?;
                node.check_timeouts();
                true
            }
            ScenarioStep::AdvanceTime { by_nanos } => {
                for node in &mut self.nodes {
                    node.advance_time(*by_nanos)
                }
                true
            }
            ScenarioStep::AdvanceNodeTime { node_id, by_nanos } => {
                let node = self
                    .nodes
                    .get_mut(node_id.index())
                    .ok_or(anyhow::anyhow!("node {node_id:?} not found"))?;
                node.advance_time(*by_nanos);
                true
            }
        };

        if dispatched {
            self.scenario.advance();
        }

        Ok(true)
    }
}

impl ClusterScenarioRun {
    pub fn target_scenario(&self) -> Option<&Scenario> {
        self.chain.back().or_else(|| self.finished.last())
    }

    pub fn cur_scenario(&self) -> &Scenario {
        self.chain.front().unwrap()
    }

    pub fn peek_i(&self) -> Option<(usize, usize)> {
        self.chain
            .iter()
            .enumerate()
            .filter_map(|(i, scenario)| {
                let step_i = if i == 0 { self.cur_step } else { 0 };
                scenario.steps.get(step_i)?;
                Some((i, step_i))
            })
            .nth(0)
    }

    pub fn peek(&self) -> Option<(&Scenario, &ScenarioStep)> {
        self.peek_i().map(|(scenario_i, step_i)| {
            let scenario = &self.chain[scenario_i];
            let step = &scenario.steps[step_i];
            (scenario, step)
        })
    }

    fn advance(&mut self) {
        if let Some((scenario_i, step_i)) = self.peek_i() {
            self.finished.extend(self.chain.drain(..scenario_i));
            if self.cur_step == step_i {
                self.cur_step += 1;
            } else {
                self.cur_step = step_i;
            }
        }
    }
}
