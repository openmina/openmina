mod config;
pub use config::ClusterConfig;

#[cfg(feature = "p2p-libp2p")]
mod p2p_task_spawner;
#[cfg(feature = "p2p-libp2p")]
pub use p2p_task_spawner::P2pTaskSpawner;

mod node_id;
pub use node_id::{ClusterNodeId, ClusterOcamlNodeId};

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::Duration;
use std::{collections::VecDeque, sync::Arc};

use libp2p::futures::{stream::FuturesUnordered, StreamExt};
use node::snark::{VerifierIndex, VerifierSRS};
use node::core::channels::mpsc;
use node::core::requests::RpcId;
use node::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
#[cfg(feature = "p2p-libp2p")]
use node::p2p::service_impl::{
    webrtc::P2pServiceCtx,
    webrtc_with_libp2p::{self, P2pServiceWebrtcWithLibp2p},
};
use node::p2p::{P2pConnectionEvent, P2pDiscoveryEvent, P2pEvent, PeerId};
use node::{
    account::{AccountPublicKey, AccountSecretKey},
    event_source::Event,
    ledger::LedgerCtx,
    p2p::{channels::ChannelId, identity::SecretKey as P2pSecretKey},
    service::Recorder,
    snark::{get_srs, get_verifier_index, VerifierKind},
    BuildEnv, Config, GlobalConfig, LedgerConfig, P2pConfig, SnarkConfig, State,
    TransitionFrontierConfig,
};
use openmina_node_invariants::{InvariantResult, Invariants};
use openmina_node_native::{http_server, rpc::RpcService, NodeService, RpcSender};
use rand::{rngs::StdRng, SeedableRng};
use serde::Serialize;

use crate::node::{DaemonJson, NonDeterministicEvent, OcamlStep, TestPeerId};
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
    static ref VERIFIER_SRS: Arc<Mutex<VerifierSRS>> = get_srs();
    static ref BLOCK_VERIFIER_INDEX: Arc<VerifierIndex> = get_verifier_index(VerifierKind::Blockchain).into();
    static ref WORK_VERIFIER_INDEX: Arc<VerifierIndex> = get_verifier_index(VerifierKind::Transaction).into();
}

pub struct Cluster {
    pub config: ClusterConfig,
    scenario: ClusterScenarioRun,
    available_ports: Box<dyn Iterator<Item = u16> + Send>,
    account_sec_keys: BTreeMap<AccountPublicKey, AccountSecretKey>,
    nodes: Vec<Node>,
    ocaml_nodes: Vec<Option<OcamlNode>>,

    rpc_counter: usize,
    ocaml_libp2p_keypair_i: usize,

    verifier_srs: Arc<Mutex<VerifierSRS>>,
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
            account_sec_keys: Default::default(),
            nodes: Vec::new(),
            ocaml_nodes: Vec::new(),

            rpc_counter: 0,
            ocaml_libp2p_keypair_i: 0,

            verifier_srs: VERIFIER_SRS.clone(),
            block_verifier_index: BLOCK_VERIFIER_INDEX.clone(),
            work_verifier_index: WORK_VERIFIER_INDEX.clone(),

            debugger,
        }
    }

    pub fn available_port(&mut self) -> Option<u16> {
        self.available_ports.next()
    }

    pub fn add_account_sec_key(&mut self, sec_key: AccountSecretKey) {
        self.account_sec_keys.insert(sec_key.public_key(), sec_key);
    }

    pub fn get_account_sec_key(&self, pub_key: &AccountPublicKey) -> Option<&AccountSecretKey> {
        self.account_sec_keys.get(pub_key)
    }

    pub fn add_rust_node(&mut self, testing_config: RustNodeTestingConfig) -> ClusterNodeId {
        let node_config = testing_config.clone();
        let node_id = ClusterNodeId::new_unchecked(self.nodes.len());
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        let secret_key = P2pSecretKey::from_bytes(match testing_config.peer_id {
            TestPeerId::Derived => {
                let mut bytes = [0; 32];
                let bytes_len = bytes.len();
                let i_bytes = node_id.index().to_be_bytes();
                let i = bytes_len - i_bytes.len();
                bytes[i..bytes_len].copy_from_slice(&i_bytes);
                bytes
            }
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
        let libp2p_port = self
            .available_ports
            .next()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "couldn't find available port in port range: {:?}",
                    self.config.port_range()
                )
            })
            .unwrap();

        let (block_producer_sec_key, block_producer_config) = testing_config
            .block_producer
            .map(|v| (v.sec_key, v.config))
            .unzip();

        let initial_peers = testing_config
            .initial_peers
            .into_iter()
            .map(|node| match node {
                ListenerNode::Rust(id) => self.node(id).unwrap().dial_addr(),
                ListenerNode::Ocaml(id) => self.ocaml_node(id).unwrap().dial_addr(),
                ListenerNode::Custom(addr) => addr,
            })
            .collect();

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
                snarker: testing_config.snark_worker,
            },
            p2p: P2pConfig {
                libp2p_port: Some(libp2p_port),
                listen_port: http_port,
                identity_pub_key: pub_key,
                initial_peers,
                max_peers: testing_config.max_peers,
                ask_initial_peers_interval: testing_config.ask_initial_peers_interval,
                enabled_channels: ChannelId::iter_all().collect(),
            },
            transition_frontier: TransitionFrontierConfig::default(),
            block_producer: block_producer_config,
        };

        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        #[cfg(feature = "p2p-libp2p")]
        let webrtc_with_libp2p::P2pServiceCtx {
            libp2p,
            webrtc: P2pServiceCtx { cmd_sender, peers },
        } = <NodeService as P2pServiceWebrtcWithLibp2p>::init(
            Some(libp2p_port),
            secret_key,
            testing_config.chain_id,
            event_sender.clone(),
            p2p_task_spawner::P2pTaskSpawner::new(shutdown_tx.clone()),
        );
        #[cfg(not(feature = "p2p-libp2p"))]
        let (cmd_sender, peers) = { (mpsc::unbounded_channel().0, Default::default()) };

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
        let mut real_service = NodeService {
            rng: StdRng::seed_from_u64(0),
            event_sender,
            event_receiver: event_receiver.into(),
            cmd_sender,
            ledger,
            peers,
            #[cfg(feature = "p2p-libp2p")]
            libp2p,
            block_producer: None,
            snark_worker_sender: None,
            rpc: rpc_service,
            stats: node::stats::Stats::new(),
            recorder: Recorder::None,
            replayer: None,
            invariants_state: Default::default(),
        };
        if let Some(producer_key) = block_producer_sec_key {
            real_service.block_producer_start(producer_key.into());
        }
        let mut service = NodeTestingService::new(real_service, node_id, shutdown_rx);
        if self.config.all_rust_to_rust_use_webrtc() {
            service.set_rust_to_rust_use_webrtc();
        }
        if self.config.is_replay() {
            service.set_replay();
        }

        let state = node::State::new(config);
        fn effects(store: &mut node::Store<NodeTestingService>, action: node::ActionWithMeta) {
            // if action.action().kind().to_string().starts_with("BlockProducer") {
            //     dbg!(action.action());
            // }
            store.service.dyn_effects(store.state.get(), &action);
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
        let node = Node::new(node_config, store);

        self.nodes.push(node);
        node_id
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
            libp2p_keypair_i: self.ocaml_libp2p_keypair_i,
            libp2p_port: next_port().unwrap(),
            graphql_port: next_port().unwrap(),
            client_port: next_port().unwrap(),
            initial_peers: testing_config.initial_peers,
            daemon_json: testing_config.daemon_json,
        })
        .expect("failed to start ocaml node");

        self.ocaml_libp2p_keypair_i += 1;

        self.ocaml_nodes.push(Some(node));
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

    pub fn ocaml_nodes_iter(&self) -> impl Iterator<Item = (ClusterOcamlNodeId, &OcamlNode)> {
        self.ocaml_nodes
            .iter()
            .enumerate()
            .filter_map(|(i, node)| node.as_ref().map(|node| (i, node)))
            .map(|(i, node)| (ClusterOcamlNodeId::new_unchecked(i), node))
    }

    pub fn node(&self, node_id: ClusterNodeId) -> Option<&Node> {
        self.nodes.get(node_id.index())
    }

    pub fn node_by_peer_id(&self, peer_id: PeerId) -> Option<&Node> {
        self.nodes_iter()
            .find(|(_, node)| node.peer_id() == peer_id)
            .map(|(_, node)| node)
    }

    pub fn node_mut(&mut self, node_id: ClusterNodeId) -> Option<&mut Node> {
        self.nodes.get_mut(node_id.index())
    }

    pub fn ocaml_node(&self, node_id: ClusterOcamlNodeId) -> Option<&OcamlNode> {
        self.ocaml_nodes
            .get(node_id.index())
            .map(|opt| opt.as_ref().expect("tried to access removed ocaml node"))
    }

    pub fn ocaml_node_by_peer_id(&self, peer_id: PeerId) -> Option<&OcamlNode> {
        self.ocaml_nodes_iter()
            .find(|(_, node)| node.peer_id() == peer_id)
            .map(|(_, node)| node)
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
            .ok_or_else(|| anyhow::anyhow!("node {node_id:?} not found"))?;
        Ok(node.pending_events_with_state())
    }

    pub async fn wait_for_pending_events(&mut self) {
        let mut nodes = &mut self.nodes[..];
        let mut futures = FuturesUnordered::new();

        while let Some((node, nodes_rest)) = nodes.split_first_mut() {
            nodes = nodes_rest;
            futures.push(async { node.wait_for_next_pending_event().await.is_some() });
        }

        while let Some(has_event) = futures.next().await {
            if has_event {
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

    pub async fn wait_for_pending_event(
        &mut self,
        node_id: ClusterNodeId,
        event_pattern: &str,
    ) -> anyhow::Result<PendingEventId> {
        let node = self
            .nodes
            .get_mut(node_id.index())
            .ok_or_else(|| anyhow::anyhow!("node {node_id:?} not found"))?;
        let timeout = tokio::time::sleep(Duration::from_secs(60));
        tokio::select! {
            opt = node.wait_for_event(&event_pattern) => opt.ok_or_else(|| anyhow::anyhow!("wait_for_event: None")),
            _ = timeout => {
                let pending_events = node.pending_events().map(|(_, event)| event.to_string()).collect::<Vec<_>>();
                return Err(anyhow::anyhow!("waiting for event timed out! node {node_id:?}, event: \"{event_pattern}\"\n{pending_events:?}"));
            }
        }
    }

    pub async fn wait_for_event_and_dispatch(
        &mut self,
        node_id: ClusterNodeId,
        event_pattern: &str,
    ) -> anyhow::Result<bool> {
        let event_id = self.wait_for_pending_event(node_id, event_pattern).await?;
        let node = self.nodes.get_mut(node_id.index()).unwrap();
        Ok(node.take_event_and_dispatch(event_id))
    }

    pub async fn add_steps_and_save(&mut self, steps: impl IntoIterator<Item = ScenarioStep>) {
        let scenario = self.scenario.chain.back_mut().unwrap();
        steps
            .into_iter()
            .for_each(|step| scenario.add_step(step).unwrap());
        scenario.save().await.unwrap();
    }

    pub async fn exec_to_end(&mut self) -> Result<(), anyhow::Error> {
        let mut i = 0;
        let total = self.scenario.cur_scenario().steps.len();
        loop {
            eprintln!("[step]: {i}/{total}");
            if !self.exec_next().await? {
                break Ok(());
            }
            i += 1;
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

    pub async fn exec_step(&mut self, step: ScenarioStep) -> anyhow::Result<bool> {
        fn node_addr_by_peer_id(
            cluster: &Cluster,
            peer_id: PeerId,
        ) -> anyhow::Result<P2pConnectionOutgoingInitOpts> {
            cluster
                .node_by_peer_id(peer_id)
                .map(|node| node.dial_addr())
                .or_else(|| {
                    cluster
                        .ocaml_node_by_peer_id(peer_id)
                        .map(|node| node.dial_addr())
                })
                .ok_or_else(|| anyhow::anyhow!("node with peer_id: '{peer_id}' not found"))
        }

        Ok(match step {
            ScenarioStep::Event { node_id, event } => {
                return self.wait_for_event_and_dispatch(node_id, &event).await;
            }
            ScenarioStep::ManualEvent { node_id, event } => self
                .nodes
                .get_mut(node_id.index())
                .ok_or_else(|| anyhow::anyhow!("node {node_id:?} not found"))?
                .dispatch_event(*event),
            ScenarioStep::NonDeterministicEvent { node_id, event } => {
                let event = match *event {
                    NonDeterministicEvent::P2pListen => return Ok(true),
                    NonDeterministicEvent::P2pConnectionClosed(peer_id) => {
                        let node = self
                            .nodes
                            .get_mut(node_id.index())
                            .ok_or_else(|| anyhow::anyhow!("node {node_id:?} not found"))?;
                        node.p2p_disconnect(peer_id);
                        let event =
                            Event::P2p(P2pEvent::Connection(P2pConnectionEvent::Closed(peer_id)));
                        return self
                            .wait_for_event_and_dispatch(node_id, &event.to_string())
                            .await;
                    }
                    NonDeterministicEvent::P2pConnectionFinalized(peer_id, res) => {
                        let node = self
                            .nodes
                            .get(node_id.index())
                            .ok_or_else(|| anyhow::anyhow!("node {node_id:?} not found"))?;
                        let res_is_ok = res.is_ok();
                        let event = Event::P2p(P2pEvent::Connection(
                            P2pConnectionEvent::Finalized(peer_id, res),
                        ));

                        if res_is_ok {
                            let is_peer_connected =
                                node.state().p2p.get_ready_peer(&peer_id).is_some();
                            // deduce if kad initiated this conn.
                            #[cfg(feature = "p2p-libp2p")]
                            {
                                if !node.state().p2p.is_peer_connected_or_connecting(&peer_id) {
                                    let my_addr = node.dial_addr();
                                    let peer = self
                                        .nodes
                                        .iter_mut()
                                        .find(|node| node.peer_id() == peer_id)
                                        .ok_or_else(|| {
                                            anyhow::anyhow!(
                                                "node with peer_id: '{peer_id}' not found"
                                            )
                                        })?;

                                    if !peer.state().p2p.is_peer_connecting(my_addr.peer_id()) {
                                        // kad initiated this connection so replay that.
                                        eprintln!(
                                            "p2p_kad_outgoing_init({:?}) -> {:?} - {}",
                                            peer.node_id(),
                                            node_id,
                                            my_addr
                                        );
                                        peer.p2p_kad_outgoing_init(my_addr);
                                    }
                                }
                            }
                            if is_peer_connected {
                                // we are already connected, so skip the extra event.
                                return Ok(true);
                            }
                            eprintln!("non_deterministic_wait_for_event_and_dispatch({node_id:?}): {event}");
                            return self
                                .wait_for_event_and_dispatch(node_id, &event.to_string())
                                .await;
                        } else {
                            event
                        }
                    }
                    #[cfg(feature = "p2p-libp2p")]
                    NonDeterministicEvent::P2pLibp2pIdentify(peer_id) => {
                        let addr = match node_addr_by_peer_id(self, peer_id)? {
                            P2pConnectionOutgoingInitOpts::LibP2P(v) => (&v).into(),
                            _ => unreachable!(),
                        };
                        P2pEvent::Libp2pIdentify(peer_id, addr).into()
                    }
                    NonDeterministicEvent::P2pDiscoveryReady => {
                        P2pEvent::Discovery(P2pDiscoveryEvent::Ready).into()
                    }
                    NonDeterministicEvent::P2pDiscoveryDidFindPeers(ids) => {
                        P2pEvent::Discovery(P2pDiscoveryEvent::DidFindPeers(ids)).into()
                    }
                    NonDeterministicEvent::P2pDiscoveryDidFindPeersError(err) => {
                        P2pEvent::Discovery(P2pDiscoveryEvent::DidFindPeersError(err)).into()
                    }
                    NonDeterministicEvent::P2pDiscoveryAddRoute(id, ids) => {
                        let addrs = ids
                            .into_iter()
                            .map(|id| node_addr_by_peer_id(&self, id))
                            .collect::<Result<Vec<_>, _>>()?;
                        P2pEvent::Discovery(P2pDiscoveryEvent::AddRoute(id, addrs)).into()
                    }
                    NonDeterministicEvent::RpcReadonly(id, req) => Event::Rpc(id, req).into(),
                };
                eprintln!("non_deterministic_event_dispatch({node_id:?}): {event}");
                self.nodes
                    .get_mut(node_id.index())
                    .ok_or_else(|| anyhow::anyhow!("node {node_id:?} not found"))?
                    .dispatch_event(event)
            }
            ScenarioStep::AddNode { config } => match config {
                NodeTestingConfig::Rust(config) => {
                    self.add_rust_node(config);
                    // TODO(binier): wait for node ports to be opened instead.
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    true
                }
                NodeTestingConfig::Ocaml(config) => {
                    // before starting ocaml node, read and save secret
                    // keys from daemon.json.
                    let mut json_owned = None;
                    let json = match &config.daemon_json {
                        DaemonJson::Custom(path) => {
                            let bytes = tokio::fs::read(path).await.map_err(|err| {
                                anyhow::anyhow!(
                                    "error reading daemon.json from path({path}): {err}"
                                )
                            })?;
                            let json = serde_json::from_slice(&bytes).map_err(|err| {
                                anyhow::anyhow!(
                                    "failed to parse damon.json from path({path}): {err}"
                                )
                            })?;
                            json_owned.insert(json)
                        }
                        DaemonJson::InMem(json) => json,
                    };
                    let accounts = json["ledger"]["accounts"].as_array().ok_or_else(|| {
                        anyhow::anyhow!("daemon.json `.ledger.accounts` is not array")
                    })?;

                    accounts
                        .iter()
                        .filter_map(|account| account["sk"].as_str())
                        .filter_map(|sk| sk.parse().ok())
                        .for_each(|sk| self.add_account_sec_key(sk));

                    self.add_ocaml_node(config);
                    true
                }
            },
            ScenarioStep::ConnectNodes { dialer, listener } => {
                let listener_addr = match listener {
                    ListenerNode::Rust(listener) => {
                        let listener = self
                            .nodes
                            .get(listener.index())
                            .ok_or_else(|| anyhow::anyhow!("node {listener:?} not found"))?;

                        listener.dial_addr()
                    }
                    ListenerNode::Ocaml(listener) => {
                        let listener = self
                            .ocaml_nodes
                            .get(listener.index())
                            .ok_or_else(|| anyhow::anyhow!("ocaml node {listener:?} not found"))?
                            .as_ref()
                            .ok_or_else(|| {
                                anyhow::anyhow!("tried to access removed ocaml node {listener:?}")
                            })?;

                        listener.dial_addr()
                    }
                    ListenerNode::Custom(addr) => addr.clone(),
                };

                self.rpc_counter += 1;
                let rpc_id = RpcId::new_unchecked(usize::MAX, self.rpc_counter);
                let dialer = self
                    .nodes
                    .get_mut(dialer.index())
                    .ok_or_else(|| anyhow::anyhow!("node {dialer:?} not found"))?;

                let req = node::rpc::RpcRequest::P2pConnectionOutgoing(listener_addr);
                dialer.dispatch_event(Event::Rpc(rpc_id, req))
            }
            ScenarioStep::CheckTimeouts { node_id } => {
                let node = self
                    .nodes
                    .get_mut(node_id.index())
                    .ok_or_else(|| anyhow::anyhow!("node {node_id:?} not found"))?;
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
                    .ok_or_else(|| anyhow::anyhow!("node {node_id:?} not found"))?;
                node.advance_time(by_nanos);
                true
            }
            ScenarioStep::Ocaml { node_id, step } => {
                let node = self.ocaml_nodes.get_mut(node_id.index());
                let node =
                    node.ok_or_else(|| anyhow::anyhow!("ocaml node {node_id:?} not found"))?;
                if matches!(step, OcamlStep::KillAndRemove) {
                    let mut node = node.take().ok_or_else(|| {
                        anyhow::anyhow!("tried to access removed ocaml node {node_id:?}")
                    })?;
                    node.exec(step).await?
                } else {
                    let node = node.as_mut().ok_or_else(|| {
                        anyhow::anyhow!("tried to access removed ocaml node {node_id:?}")
                    })?;
                    node.exec(step).await?
                }
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
