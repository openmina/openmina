mod config;
pub use config::ClusterConfig;

mod p2p_task_spawner;
use openmina_node_invariants::{InvariantResult, Invariants};
pub use p2p_task_spawner::P2pTaskSpawner;

mod node_id;
pub use node_id::{ClusterNodeId, ClusterOcamlNodeId};

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
    BuildEnv, Config, GlobalConfig, LedgerConfig, P2pConfig, SnarkConfig, State,
    TransitionFrontierConfig,
};
use openmina_node_native::{http_server, rpc::RpcService, NodeService, RpcSender};
use rand::{rngs::StdRng, SeedableRng};
use serde::Serialize;

use crate::node::TestPeerId;
use crate::{
    network_debugger::Debugger,
    node::{
        Node, NodeTestingConfig, OcamlNode, OcamlNodeConfig, OcamlNodeTestingConfig,
        RustNodeTestingConfig,
    },
    scenario::{ListenerNode, Scenario, ScenarioId, ScenarioStep},
    service::{NodeTestingService, PendingEventId},
};

lazy_static::lazy_static! {
    static ref VERIFIER_SRS: Arc<VerifierSRS> = get_srs().into();
    static ref BLOCK_VERIFIER_INDEX: Arc<VerifierIndex> = get_verifier_index(VerifierKind::Blockchain).into();
    static ref WORK_VERIFIER_INDEX: Arc<VerifierIndex> = get_verifier_index(VerifierKind::Transaction).into();
}

pub struct Cluster {
    pub config: ClusterConfig,
    scenario: ClusterScenarioRun,
    available_ports: Box<dyn Iterator<Item = u16> + Send>,
    nodes: Vec<Node>,
    ocaml_nodes: Vec<OcamlNode>,

    rpc_counter: usize,

    verifier_srs: Arc<VerifierSRS>,
    block_verifier_index: Arc<VerifierIndex>,
    work_verifier_index: Arc<VerifierIndex>,

    debugger: Option<Debugger>,
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
        let debugger = if config.is_use_debugger() {
            Some(Debugger::drone_ci())
        } else {
            None
        };
        Self {
            config,
            scenario: ClusterScenarioRun {
                chain: Default::default(),
                finished: Default::default(),
                cur_step: 0,
            },
            available_ports: Box::new(available_ports),
            nodes: Vec::new(),
            ocaml_nodes: Vec::new(),

            rpc_counter: 0,

            verifier_srs: VERIFIER_SRS.clone(),
            block_verifier_index: BLOCK_VERIFIER_INDEX.clone(),
            work_verifier_index: WORK_VERIFIER_INDEX.clone(),

            debugger,
        }
    }

    pub fn available_port(&mut self) -> Option<u16> {
        self.available_ports.next()
    }

    pub fn add_rust_node(&mut self, testing_config: RustNodeTestingConfig) -> ClusterNodeId {
        let node_i = self.nodes.len();
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        let secret_key = P2pSecretKey::from_bytes(match testing_config.peer_id {
            TestPeerId::Derived => {
                let mut bytes = [0; 32];
                let bytes_len = bytes.len();
                let i_bytes = node_i.to_be_bytes();
                let i = bytes_len - i_bytes.len();
                bytes[i..bytes_len].copy_from_slice(&i_bytes);
                bytes
            }
            TestPeerId::Random => rand::random(),
            TestPeerId::Bytes(bytes) => bytes,
        });
        let pub_key = secret_key.public_key();

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
        let libp2p_port = testing_config.libp2p_port.unwrap_or_else(|| {
            self.available_ports
                .next()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "couldn't find available port in port range: {:?}",
                        self.config.port_range()
                    )
                })
                .unwrap()
        });

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
                libp2p_port: Some(libp2p_port),
                listen_port: http_port,
                identity_pub_key: pub_key,
                initial_peers: testing_config.initial_peers,
                max_peers: testing_config.max_peers,
                ask_initial_peers_interval: testing_config.ask_initial_peers_interval,
                enabled_channels: ChannelId::iter_all().collect(),
            },
            transition_frontier: TransitionFrontierConfig::default(),
            block_producer: None,
        };

        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let (p2p_event_sender, mut rx) = mpsc::unbounded_channel::<P2pEvent>();

        let webrtc_with_libp2p::P2pServiceCtx {
            libp2p,
            webrtc: P2pServiceCtx { cmd_sender, peers },
        } = <NodeService as P2pServiceWebrtcWithLibp2p>::init(
            Some(libp2p_port),
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
            block_producer: None,
            snark_worker_sender: None,
            rpc: rpc_service,
            stats: node::stats::Stats::new(),
            recorder: Recorder::None,
            replayer: None,
            invariants_state: Default::default(),
        };
        let mut service = NodeTestingService::new(real_service, shutdown_rx);
        if self.config.all_rust_to_rust_use_webrtc() {
            service.set_rust_to_rust_use_webrtc();
        }

        let state = node::State::new(config);
        fn effects(store: &mut node::Store<NodeTestingService>, action: node::ActionWithMeta) {
            let peer_id = store.state().p2p.my_id();
            openmina_core::log::trace!(action.time(); "{peer_id}: {:?}", action.action().kind());

            for (invariant, res) in Invariants::check_all(store, &action) {
                // TODO(binier): record instead of panicing.
                match res {
                    InvariantResult::Violation(violation) => {
                        panic!(
                            "Invariant({}) violated! violation: {violation}",
                            invariant.to_str()
                        );
                    }
                    InvariantResult::Updated => {}
                    InvariantResult::Ok => {}
                }
            }

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

    pub fn add_ocaml_node(&mut self, testing_config: OcamlNodeTestingConfig) -> ClusterOcamlNodeId {
        let node_i = self.ocaml_nodes.len();

        let mut next_port = || {
            self.available_ports.next().ok_or_else(|| {
                anyhow::anyhow!(
                    "couldn't find available port in port range: {:?}",
                    self.config.port_range()
                )
            })
        };

        let temp_dir = temp_dir::TempDir::new().expect("failed to create tempdir");
        let node = OcamlNode::start(OcamlNodeConfig {
            executable: self.config.ocaml_node_executable().clone(),
            dir: temp_dir,
            libp2p_port: next_port().unwrap(),
            graphql_port: next_port().unwrap(),
            client_port: next_port().unwrap(),
            initial_peers: testing_config.initial_peers,
            daemon_json: testing_config.daemon_json,
            daemon_json_update_timestamp: testing_config.daemon_json_update_timestamp,
        });

        self.ocaml_nodes
            .push(node.expect("failed to start ocaml node"));
        ClusterOcamlNodeId::new_unchecked(node_i)
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
            match config {
                NodeTestingConfig::Rust(config) => {
                    self.add_rust_node(config.clone());
                }
                NodeTestingConfig::Ocaml(config) => {
                    self.add_ocaml_node(config.clone());
                }
            }
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

    pub fn nodes_iter(&self) -> impl Iterator<Item = (ClusterNodeId, &Node)> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (ClusterNodeId::new_unchecked(i), node))
    }

    pub fn node(&self, node_id: ClusterNodeId) -> Option<&Node> {
        self.nodes.get(node_id.index())
    }

    pub fn ocaml_node(&self, node_id: ClusterOcamlNodeId) -> Option<&OcamlNode> {
        self.ocaml_nodes.get(node_id.index())
    }

    pub fn pending_events(
        &mut self,
    ) -> impl Iterator<
        Item = (
            ClusterNodeId,
            &State,
            impl Iterator<Item = (PendingEventId, &Event)>,
        ),
    > {
        self.nodes.iter_mut().enumerate().map(|(i, node)| {
            let node_id = ClusterNodeId::new_unchecked(i);
            let (state, pending_events) = node.pending_events_with_state();
            (node_id, state, pending_events)
        })
    }

    pub fn node_pending_events(
        &mut self,
        node_id: ClusterNodeId,
    ) -> Result<(&State, impl Iterator<Item = (PendingEventId, &Event)>), anyhow::Error> {
        let node = self
            .nodes
            .get_mut(node_id.index())
            .ok_or(anyhow::anyhow!("node {node_id:?} not found"))?;
        Ok(node.pending_events_with_state())
    }

    pub async fn wait_for_pending_events(&mut self) {
        loop {
            tokio::time::sleep(Duration::from_millis(10)).await;
            if self
                .nodes
                .iter_mut()
                .any(|node| node.pending_events().next().is_some())
            {
                break;
            }
        }
    }

    pub async fn wait_for_pending_events_with_timeout(&mut self, timeout: Duration) -> bool {
        let timeout = tokio::time::sleep(timeout);

        tokio::select! {
            _ = self.wait_for_pending_events() => true,
            _ = timeout => false,
        }
    }

    pub async fn add_steps_and_save(&mut self, steps: impl IntoIterator<Item = ScenarioStep>) {
        let scenario = self.scenario.chain.back_mut().unwrap();
        steps
            .into_iter()
            .for_each(|step| scenario.add_step(step).unwrap());
        scenario.save().await.unwrap();
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
        let dispatched = self.exec_step(step.clone()).await?;

        if dispatched {
            self.scenario.advance();
        }

        Ok(dispatched)
    }

    pub async fn exec_step(&mut self, step: ScenarioStep) -> Result<bool, anyhow::Error> {
        Ok(match step {
            ScenarioStep::ManualEvent { node_id, event } => self
                .nodes
                .get_mut(node_id.index())
                .ok_or(anyhow::anyhow!("node {node_id:?} not found"))?
                .dispatch_event(*event),
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
                    ListenerNode::Ocaml(listener) => {
                        let listener = self
                            .ocaml_nodes
                            .get_mut(listener.index())
                            .ok_or(anyhow::anyhow!("ocaml node {listener:?} not found"))?;

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
                    node.advance_time(by_nanos)
                }
                true
            }
            ScenarioStep::AdvanceNodeTime { node_id, by_nanos } => {
                let node = self
                    .nodes
                    .get_mut(node_id.index())
                    .ok_or(anyhow::anyhow!("node {node_id:?} not found"))?;
                node.advance_time(by_nanos);
                true
            }
            ScenarioStep::Ocaml { node_id, step: cmd } => {
                let node = self
                    .ocaml_nodes
                    .get_mut(node_id.index())
                    .ok_or(anyhow::anyhow!("ocaml node {node_id:?} not found"))?;
                node.exec(cmd).await?
            }
        })
    }

    pub fn debugger(&self) -> Option<&Debugger> {
        self.debugger.as_ref()
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
