mod config;
pub use config::{ClusterConfig, ProofKind};

mod p2p_task_spawner;
use node::account::{AccountPublicKey, AccountSecretKey};
use openmina_core::ChainId;
pub use p2p_task_spawner::P2pTaskSpawner;

mod node_id;
pub use node_id::{ClusterNodeId, ClusterOcamlNodeId};
use serde::de::DeserializeOwned;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;
use std::{collections::VecDeque, sync::Arc};

use libp2p::futures::{stream::FuturesUnordered, StreamExt};
use libp2p::identity::Keypair;
use node::core::channels::mpsc;
use node::core::log::system_time;
use node::core::requests::RpcId;
use node::core::warn;
use node::p2p::service_impl::{
    webrtc::P2pServiceCtx, webrtc_with_libp2p::P2pServiceWebrtcWithLibp2p,
};
use node::p2p::{P2pConnectionEvent, P2pEvent, PeerId};
use node::snark::{VerifierIndex, VerifierSRS};
use node::{
    event_source::Event,
    ledger::{LedgerCtx, LedgerManager},
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

#[allow(dead_code)]
fn openmina_path<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache/openmina").join(path))
}

#[allow(dead_code)]
fn read_index<T: DeserializeOwned>(name: &str) -> Option<T> {
    openmina_path(name)
        .and_then(|path| {
            if !path.exists() {
                return None;
            }
            match std::fs::File::open(path) {
                Ok(v) => Some(v),
                Err(e) => {
                    warn!(system_time(); "cannot find verifier index for {name}: {e}");
                    None
                }
            }
        })
        .and_then(|file| match serde_cbor::from_reader(file) {
            Ok(v) => Some(v),
            Err(e) => {
                warn!(system_time(); "cannot read verifier index for {name}: {e}");
                None
            }
        })
}

#[allow(dead_code)]
fn write_index<T: Serialize>(name: &str, index: &T) -> Option<()> {
    openmina_path(name)
        .and_then(|path| {
            let Some(parent) = path.parent() else {
                warn!(system_time(); "cannot get parent for {path:?}");
                return None;
            };
            if let Err(e) = std::fs::create_dir_all(parent) {
                warn!(system_time(); "cannot create parent dir for {parent:?}: {e}");
                return None;
            }
            match std::fs::File::create(&path) {
                Ok(v) => Some(v),
                Err(e) => {
                    warn!(system_time(); "cannot create file {path:?}: {e}");
                    None
                }
            }
        })
        .and_then(|file| match serde_cbor::to_writer(file, index) {
            Ok(_) => Some(()),
            Err(e) => {
                warn!(system_time(); "cannot write verifier index for {name}: {e}");
                None
            }
        })
}

lazy_static::lazy_static! {
    static ref VERIFIER_SRS: Arc<Mutex<VerifierSRS>> = get_srs();
    static ref BLOCK_VERIFIER_INDEX: Arc<VerifierIndex> = get_verifier_index(VerifierKind::Blockchain).into();
    static ref WORK_VERIFIER_INDEX: Arc<VerifierIndex> = get_verifier_index(VerifierKind::Transaction).into();
}

lazy_static::lazy_static! {
    static ref DETERMINISTIC_ACCOUNT_SEC_KEYS: BTreeMap<AccountPublicKey, AccountSecretKey> = (0..1000)
        .map(|i| AccountSecretKey::deterministic(i))
        .map(|sec_key| (sec_key.public_key(), sec_key))
        .collect();
}

pub struct Cluster {
    pub config: ClusterConfig,
    scenario: ClusterScenarioRun,
    available_ports: Box<dyn Iterator<Item = u16> + Send>,
    account_sec_keys: BTreeMap<AccountPublicKey, AccountSecretKey>,
    nodes: Vec<Node>,
    ocaml_nodes: Vec<Option<OcamlNode>>,
    // TODO: remove option if this is viable in the future
    chain_id: Option<ChainId>,
    initial_time: Option<redux::Timestamp>,

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
            chain_id: None,
            initial_time: None,

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
        self.account_sec_keys
            .get(pub_key)
            .or_else(|| DETERMINISTIC_ACCOUNT_SEC_KEYS.get(pub_key))
    }

    pub fn set_chain_id(&mut self, chain_id: ChainId) {
        self.chain_id = Some(chain_id)
    }

    pub fn set_initial_time(&mut self, initial_time: redux::Timestamp) {
        self.initial_time = Some(initial_time)
    }

    pub fn get_chain_id(&self) -> Option<ChainId> {
        self.chain_id.clone()
    }

    pub fn get_initial_time(&self) -> Option<redux::Timestamp> {
        self.initial_time
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
                timeouts: testing_config.timeouts,
                chain_id: testing_config.chain_id.clone(),
                peer_discovery: true,
                initial_time: Duration::ZERO,
            },
            transition_frontier: TransitionFrontierConfig::new(testing_config.genesis),
            block_producer: block_producer_config,
        };

        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let keypair = Keypair::ed25519_from_bytes(secret_key.to_bytes())
            .expect("secret key bytes must be valid");

        let p2p_service_ctx = <NodeService as P2pServiceWebrtcWithLibp2p>::init(
            Some(libp2p_port),
            secret_key.clone(),
            testing_config.chain_id,
            event_sender.clone(),
            p2p_task_spawner::P2pTaskSpawner::new(shutdown_tx.clone()),
        );

        let P2pServiceCtx { cmd_sender, peers } = p2p_service_ctx.webrtc;

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

        let mut ledger = LedgerCtx::default();

        // TODO(tizoc): Only used for the current workaround to make staged ledger
        // reconstruction async, can be removed when the ledger services are made async
        ledger.set_event_sender(event_sender.clone());

        let mut real_service = NodeService {
            rng: StdRng::seed_from_u64(0),
            event_sender,
            event_receiver: event_receiver.into(),
            cmd_sender,
            ledger_manager: LedgerManager::spawn(ledger),
            peers,
            #[cfg(feature = "p2p-libp2p")]
            mio: p2p_service_ctx.mio,
            network: Default::default(),
            block_producer: None,
            keypair,
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
        service.set_proof_kind(self.config.proof_kind());
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
            block_producer: testing_config.block_producer,
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
        poll: bool,
    ) -> impl Iterator<
        Item = (
            ClusterNodeId,
            &State,
            impl Iterator<Item = (PendingEventId, &Event)>,
        ),
    > {
        self.nodes.iter_mut().enumerate().map(move |(i, node)| {
            let node_id = ClusterNodeId::new_unchecked(i);
            let (state, pending_events) = node.pending_events_with_state(poll);
            (node_id, state, pending_events)
        })
    }

    pub fn node_pending_events(
        &mut self,
        node_id: ClusterNodeId,
        poll: bool,
    ) -> Result<(&State, impl Iterator<Item = (PendingEventId, &Event)>), anyhow::Error> {
        let node = self
            .nodes
            .get_mut(node_id.index())
            .ok_or_else(|| anyhow::anyhow!("node {node_id:?} not found"))?;
        Ok(node.pending_events_with_state(poll))
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
                let pending_events = node.pending_events(false).map(|(_, event)| event.to_string()).collect::<Vec<_>>();
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
