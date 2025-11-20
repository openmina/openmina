//! Cluster Management for Multi-Node Testing
//!
//! This module provides the core infrastructure for managing clusters of
//! Mina nodes during testing scenarios. It supports both Rust and OCaml
//! node implementations, enabling cross-implementation testing and complex
//! multi-node scenarios.
//!
//! # Key Components
//!
//! - [`Cluster`] - Main cluster coordinator managing node lifecycle
//! - Node addition methods for different node types
//! - Port allocation and resource management
//! - Scenario execution and state tracking
//! - Network debugger integration
//!
//! # Node Addition Methods
//!
//! - [`Cluster::add_rust_node`] - Add Rust implementation nodes
//! - [`Cluster::add_ocaml_node`] - Add OCaml implementation nodes
//!
//! # Example
//!
//! ```rust,no_run
//! let mut cluster = Cluster::new(ClusterConfig::default());
//!
//! // Add Rust node with custom configuration
//! let rust_node = cluster.add_rust_node(RustNodeTestingConfig::default());
//!
//! // Add OCaml node for cross-implementation testing
//! let ocaml_node = cluster.add_ocaml_node(OcamlNodeTestingConfig::default());
//! ```

mod config;
pub use config::{ClusterConfig, ProofKind};

mod p2p_task_spawner;

mod node_id;
use mina_core::channels::Aborter;
pub use node_id::{ClusterNodeId, ClusterOcamlNodeId};

pub mod runner;

use std::{
    collections::{BTreeMap, VecDeque},
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex as StdMutex},
    time::Duration,
};

use libp2p::futures::{stream::FuturesUnordered, StreamExt};

use ledger::proofs::provers::BlockProver;
use mina_node_invariants::{InvariantResult, Invariants};
use mina_node_native::{http_server, NodeServiceBuilder};
use node::{
    account::{AccountPublicKey, AccountSecretKey},
    core::{
        consensus::ConsensusConstants,
        constants::constraint_constants,
        invariants::InvariantsState,
        log::{info, system_time, warn},
        requests::RpcId,
        thread,
    },
    event_source::Event,
    p2p::{
        channels::ChannelId, identity::SecretKey as P2pSecretKey, P2pConnectionEvent, P2pEvent,
        P2pLimits, P2pMeshsubConfig, PeerId,
    },
    service::{Recorder, Service},
    snark::{get_srs, BlockVerifier, TransactionVerifier, VerifierSRS},
    BuildEnv, Config, GlobalConfig, LedgerConfig, P2pConfig, SnarkConfig, State,
    TransitionFrontierConfig,
};
use serde::{de::DeserializeOwned, Serialize};
use temp_dir::TempDir;

use crate::{
    network_debugger::Debugger,
    node::{
        DaemonJson, Node, NodeTestingConfig, NonDeterministicEvent, OcamlNode, OcamlNodeConfig,
        OcamlNodeTestingConfig, OcamlStep, RustNodeTestingConfig, TestPeerId,
    },
    scenario::{ListenerNode, Scenario, ScenarioId, ScenarioStep},
    service::{NodeTestingService, PendingEventId},
};

#[allow(dead_code)]
fn mina_path<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache/mina").join(path))
}

#[allow(dead_code)]
fn read_index<T: DeserializeOwned>(name: &str) -> Option<T> {
    mina_path(name)
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
        .and_then(|mut file| {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).ok().and(Some(buf))
        })
        .and_then(|bytes| match postcard::from_bytes(&bytes) {
            Ok(v) => Some(v),
            Err(e) => {
                warn!(system_time(); "cannot read verifier index for {name}: {e}");
                None
            }
        })
}

#[allow(dead_code)]
fn write_index<T: Serialize>(name: &str, index: &T) -> Option<()> {
    mina_path(name)
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
        .and_then(|file| match postcard::to_io(index, file) {
            Ok(_) => Some(()),
            Err(e) => {
                warn!(system_time(); "cannot write verifier index for {name}: {e}");
                None
            }
        })
}

lazy_static::lazy_static! {
    static ref VERIFIER_SRS: Arc<VerifierSRS> = get_srs();
}

/// Manages a cluster of Mina nodes for testing scenarios.
///
/// The `Cluster` struct coordinates multiple node instances, handling
/// resource allocation, configuration, and lifecycle management. It supports
/// both Rust and OCaml node implementations for comprehensive testing.
///
/// # Default Behaviors
///
/// - **Port allocation**: Automatically assigns available ports from the
///   configured range, testing availability before assignment
/// - **Keypair management**: Uses deterministic keypairs for Rust nodes and
///   rotates through predefined keypairs for OCaml nodes
/// - **Resource isolation**: Each node gets isolated temporary directories
/// - **Verifier indices**: Shared verifier SRS and indices across all nodes
/// - **Network debugging**: Optional debugger integration for CI environments
///
/// # Node Addition
///
/// The cluster provides specialized methods for adding different node types:
/// - Rust nodes via [`add_rust_node`](Self::add_rust_node)
/// - OCaml nodes via [`add_ocaml_node`](Self::add_ocaml_node)
pub struct Cluster {
    /// Cluster-wide configuration settings
    pub config: ClusterConfig,
    /// Current scenario execution state
    scenario: ClusterScenarioRun,
    /// Iterator over available ports for node allocation
    available_ports: Box<dyn Iterator<Item = u16> + Send>,
    /// Registry of account secret keys for deterministic testing
    account_sec_keys: BTreeMap<AccountPublicKey, AccountSecretKey>,
    /// Collection of active Rust nodes
    nodes: Vec<Node>,
    /// Collection of active OCaml nodes (Option for lifecycle management)
    ocaml_nodes: Vec<Option<OcamlNode>>,
    /// Genesis timestamp for deterministic time progression
    initial_time: Option<redux::Timestamp>,

    /// Counter for generating unique RPC request IDs
    rpc_counter: usize,
    /// Index for rotating OCaml LibP2P keypairs
    ocaml_libp2p_keypair_i: usize,

    /// Shared verifier SRS for proof verification
    verifier_srs: Arc<VerifierSRS>,
    /// Block verifier index for consensus validation
    block_verifier_index: BlockVerifier,
    /// Transaction verifier index for transaction validation
    work_verifier_index: TransactionVerifier,

    /// Optional network traffic debugger
    debugger: Option<Debugger>,
    /// Shared state for invariant checking across nodes
    invariants_state: Arc<StdMutex<InvariantsState>>,
}

/// Tracks the execution state of scenario chains within a cluster.
///
/// Manages the progression through scenario steps and maintains history
/// of completed scenarios for debugging and analysis.
#[derive(Serialize)]
pub struct ClusterScenarioRun {
    /// Queue of scenarios to be executed (supports scenario inheritance)
    chain: VecDeque<Scenario>,
    /// History of completed scenarios
    finished: Vec<Scenario>,
    /// Current step index within the active scenario
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
            initial_time: None,

            rpc_counter: 0,
            ocaml_libp2p_keypair_i: 0,

            verifier_srs: VERIFIER_SRS.clone(),
            block_verifier_index: BlockVerifier::make(),
            work_verifier_index: TransactionVerifier::make(),

            debugger,
            invariants_state: Arc::new(StdMutex::new(Default::default())),
        }
    }

    pub fn available_port(&mut self) -> Option<u16> {
        self.available_ports.next()
    }

    pub fn add_account_sec_key(&mut self, sec_key: AccountSecretKey) {
        self.account_sec_keys.insert(sec_key.public_key(), sec_key);
    }

    pub fn get_account_sec_key(&self, pub_key: &AccountPublicKey) -> Option<&AccountSecretKey> {
        self.account_sec_keys.get(pub_key).or_else(|| {
            AccountSecretKey::deterministic_iter().find(|sec_key| &sec_key.public_key() == pub_key)
        })
    }

    pub fn set_initial_time(&mut self, initial_time: redux::Timestamp) {
        self.initial_time = Some(initial_time)
    }

    pub fn get_initial_time(&self) -> Option<redux::Timestamp> {
        self.initial_time
    }

    /// Add a new Rust implementation node to the cluster.
    ///
    /// Creates and configures a Rust Mina node with the specified testing
    /// configuration. This method handles all aspects of node initialization
    /// including port allocation, key generation, service setup, and state
    /// initialization.
    ///
    /// # Default Behaviors
    ///
    /// - **Port allocation**: HTTP and LibP2P ports automatically assigned
    ///   from available port range
    /// - **Peer identity**: Deterministic LibP2P keypair based on node index
    /// - **Work directory**: Isolated temporary directory per node
    /// - **Invariants**: Automatic invariant checking enabled
    /// - **HTTP server**: Spawned on separate thread for API access
    /// - **Proof verification**: Shared verifier indices and SRS
    ///
    /// # Configuration Options
    ///
    /// - `peer_id`: Deterministic or custom LibP2P identity
    /// - `libp2p_port`: Custom P2P port (auto-assigned if None)
    /// - `initial_peers`: Peer connection targets (supports node references)
    /// - `block_producer`: Optional block production configuration
    /// - `genesis`: Genesis ledger and protocol constants
    /// - `snark_worker`: SNARK work generation settings
    ///
    /// # Returns
    ///
    /// Returns a [`ClusterNodeId`] that can be used to reference this node
    /// in scenarios and for inter-node connections.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - No available ports in the configured range
    /// - Node service initialization fails
    /// - Invalid genesis configuration
    pub fn add_rust_node(&mut self, testing_config: RustNodeTestingConfig) -> ClusterNodeId {
        let rng_seed = [0; 32];
        let node_config = testing_config.clone();
        let node_id = ClusterNodeId::new_unchecked(self.nodes.len());

        info!(
            system_time();
            "Adding Rust node {} with config: max_peers={}, snark_worker={:?}, \
             block_producer={}",
            node_id.index(),
            testing_config.max_peers,
            testing_config.snark_worker,
            testing_config.block_producer.is_some()
        );

        let work_dir = TempDir::new().unwrap();
        let shutdown_initiator = Aborter::default();
        let shutdown_listener = shutdown_initiator.aborted();
        let p2p_sec_key = match testing_config.peer_id {
            TestPeerId::Derived => {
                info!(system_time(); "Using deterministic peer ID for node {}", node_id.index());
                P2pSecretKey::deterministic(node_id.index())
            }
            TestPeerId::Bytes(bytes) => {
                info!(system_time(); "Using custom peer ID for node {}", node_id.index());
                P2pSecretKey::from_bytes(bytes)
            }
        };

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

        info!(
            system_time();
            "Assigned ports for Rust node {}: HTTP={}, LibP2P={}",
            node_id.index(),
            http_port,
            libp2p_port
        );

        let (block_producer_sec_key, block_producer_config) = testing_config
            .block_producer
            .map(|v| {
                info!(
                    system_time();
                    "Configuring block producer for Rust node {} with public key: {}",
                    node_id.index(),
                    v.sec_key.public_key()
                );
                (v.sec_key, v.config)
            })
            .unzip();

        let initial_peers: Vec<_> = testing_config
            .initial_peers
            .into_iter()
            .map(|node| {
                let addr = match &node {
                    ListenerNode::Rust(id) => {
                        info!(system_time(); "Adding Rust peer {} as initial peer", id.index());
                        self.node(*id).unwrap().dial_addr()
                    }
                    ListenerNode::Ocaml(id) => {
                        info!(system_time(); "Adding OCaml peer {} as initial peer", id.index());
                        self.ocaml_node(*id).unwrap().dial_addr()
                    }
                    ListenerNode::Custom(addr) => {
                        info!(system_time(); "Adding custom peer: {:?}", addr);
                        addr.clone()
                    }
                };
                addr
            })
            .collect();

        if !initial_peers.is_empty() {
            info!(
                system_time();
                "Rust node {} configured with {} initial peers",
                node_id.index(),
                initial_peers.len()
            );
        } else {
            info!(system_time(); "Rust node {} configured as seed node (no initial peers)", node_id.index());
        }

        let protocol_constants = testing_config
            .genesis
            .protocol_constants()
            .expect("wrong protocol constants");
        let consensus_consts =
            ConsensusConstants::create(constraint_constants(), &protocol_constants);

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
                consensus_constants: consensus_consts.clone(),
                client_port: Some(http_port),
                testing_run: true,
            },
            p2p: P2pConfig {
                libp2p_port: Some(libp2p_port),
                listen_port: Some(http_port),
                identity_pub_key: p2p_sec_key.public_key(),
                initial_peers,
                external_addrs: vec![],
                enabled_channels: ChannelId::iter_all().collect(),
                peer_discovery: testing_config.peer_discovery,
                timeouts: testing_config.timeouts,
                limits: P2pLimits::default().with_max_peers(Some(testing_config.max_peers)),
                meshsub: P2pMeshsubConfig {
                    initial_time: testing_config
                        .initial_time
                        .checked_sub(redux::Timestamp::ZERO)
                        .unwrap_or_default(),
                    ..Default::default()
                },
            },
            transition_frontier: TransitionFrontierConfig::new(testing_config.genesis),
            block_producer: block_producer_config,
            archive: None,
            tx_pool: ledger::transaction_pool::Config {
                trust_system: (),
                pool_max_size: 3000,
                slot_tx_end: None,
            },
        };

        let mut service_builder = NodeServiceBuilder::new(rng_seed);
        service_builder
            .ledger_init()
            .p2p_init_with_custom_task_spawner(
                p2p_sec_key.clone(),
                p2p_task_spawner::P2pTaskSpawner::new(shutdown_listener.clone()),
            )
            .gather_stats()
            .record(match testing_config.recorder {
                crate::node::Recorder::None => Recorder::None,
                crate::node::Recorder::StateWithInputActions => {
                    Recorder::only_input_actions(work_dir.path())
                }
            });

        if let Some(keypair) = block_producer_sec_key {
            info!(system_time(); "Initializing block producer for Rust node {}", node_id.index());
            let provers = BlockProver::make(None, None);
            service_builder.block_producer_init(keypair, Some(provers));
        }

        let real_service = service_builder
            .build()
            .map_err(|err| anyhow::anyhow!("node service build failed! error: {err}"))
            .unwrap();

        // spawn http-server
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let shutdown = shutdown_listener.clone();
        let rpc_sender = real_service.rpc_sender();
        thread::Builder::new()
            .name("mina_http_server".to_owned())
            .spawn(move || {
                let local_set = tokio::task::LocalSet::new();
                let task = async {
                    tokio::select! {
                        _ = shutdown.wait() => {}
                        _ = http_server::run(http_port, rpc_sender) => {}
                    }
                };
                local_set.block_on(&runtime, task);
            })
            .unwrap();

        let invariants_state = self.invariants_state.clone();
        let mut service =
            NodeTestingService::new(real_service, node_id, invariants_state, shutdown_initiator);

        service.set_proof_kind(self.config.proof_kind());
        if self.config.all_rust_to_rust_use_webrtc() {
            service.set_rust_to_rust_use_webrtc();
        }
        if self.config.is_replay() {
            service.set_replay();
        }

        let state = node::State::new(config, &consensus_consts, testing_config.initial_time);
        fn effects(store: &mut node::Store<NodeTestingService>, action: node::ActionWithMeta) {
            // if action.action().kind().to_string().starts_with("BlockProducer") {
            //     dbg!(action.action());
            // }

            store.service.dyn_effects(store.state.get(), &action);
            let peer_id = store.state().p2p.my_id();
            mina_core::log::trace!(action.time(); "{peer_id}: {:?}", action.action().kind());

            for (invariant, res) in Invariants::check_all(store, &action) {
                // TODO(binier): record instead of panicing.
                match res {
                    InvariantResult::Ignored(reason) => {
                        unreachable!("No invariant should be ignored! ignore reason: {reason:?}");
                    }
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
        let mut store = node::Store::new(
            node::reducer,
            effects,
            service,
            testing_config.initial_time.into(),
            state,
        );
        // record initial state.
        {
            store
                .service
                .recorder()
                .initial_state(rng_seed, p2p_sec_key, store.state.get());
        }

        let node = Node::new(work_dir, node_config, store);

        info!(
            system_time();
            "Successfully created Rust node {} at ports HTTP={}, LibP2P={}",
            node_id.index(),
            http_port,
            libp2p_port
        );

        self.nodes.push(node);
        node_id
    }

    /// Add a new OCaml implementation node to the cluster.
    ///
    /// Creates and spawns an OCaml Mina daemon process with the specified
    /// configuration. This method handles process spawning, port allocation,
    /// directory setup, and daemon configuration.
    ///
    /// # Default Behaviors
    ///
    /// - **Executable selection**: Automatically detects local binary or
    ///   falls back to default Docker image
    /// - **Port allocation**: LibP2P, GraphQL, and client ports automatically
    ///   assigned from available range
    /// - **Keypair rotation**: Uses predefined LibP2P keypairs, rotating
    ///   through the set for each new node
    /// - **Process management**: Spawns daemon with proper environment
    ///   variables and argument configuration
    /// - **Logging**: Stdout/stderr forwarded with port-based prefixes
    /// - **Docker support**: Automatic container management when using Docker
    ///
    /// # Configuration Options
    ///
    /// - `initial_peers`: List of peer connection targets
    /// - `daemon_json`: Genesis configuration (file path or in-memory JSON)
    /// - `block_producer`: Optional block production key
    ///
    /// # Docker vs Local Execution
    ///
    /// The method automatically determines execution mode:
    /// 1. Attempts to use locally installed `mina` binary
    /// 2. Falls back to Docker with default image if binary not found
    /// 3. Custom Docker images supported via configuration
    ///
    /// # Returns
    ///
    /// Returns a [`ClusterOcamlNodeId`] for referencing this OCaml node
    /// in scenarios and peer connections.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - No available ports in the configured range
    /// - Temporary directory creation fails
    /// - OCaml daemon process spawn fails
    pub fn add_ocaml_node(&mut self, testing_config: OcamlNodeTestingConfig) -> ClusterOcamlNodeId {
        let node_i = self.ocaml_nodes.len();

        info!(
            system_time();
            "Adding OCaml node {} with {} initial peers, block_producer={}",
            node_i,
            testing_config.initial_peers.len(),
            testing_config.block_producer.is_some()
        );

        let executable = self.config.ocaml_node_executable();
        let mut next_port = || {
            self.available_ports.next().ok_or_else(|| {
                anyhow::anyhow!(
                    "couldn't find available port in port range: {:?}",
                    self.config.port_range()
                )
            })
        };

        let temp_dir = temp_dir::TempDir::new().expect("failed to create tempdir");
        let libp2p_port = next_port().unwrap();
        let graphql_port = next_port().unwrap();
        let client_port = next_port().unwrap();

        info!(
            system_time();
            "Assigned ports for OCaml node {}: LibP2P={}, GraphQL={}, Client={}",
            node_i,
            libp2p_port,
            graphql_port,
            client_port
        );

        let node = OcamlNode::start(OcamlNodeConfig {
            executable,
            dir: temp_dir,
            libp2p_keypair_i: self.ocaml_libp2p_keypair_i,
            libp2p_port,
            graphql_port,
            client_port,
            initial_peers: testing_config.initial_peers,
            daemon_json: testing_config.daemon_json,
            block_producer: testing_config.block_producer,
        })
        .expect("failed to start ocaml node");

        info!(
            system_time();
            "Successfully started OCaml node {} with keypair index {}",
            node_i,
            self.ocaml_libp2p_keypair_i
        );

        self.ocaml_libp2p_keypair_i += 1;

        self.ocaml_nodes.push(Some(node));
        ClusterOcamlNodeId::new_unchecked(node_i)
    }

    pub async fn start(&mut self, scenario: Scenario) -> Result<(), anyhow::Error> {
        let mut parent_id = scenario.info.parent_id.clone();
        self.scenario.chain.push_back(scenario);

        while let Some(ref id) = parent_id {
            let scenario = Scenario::load(id).await?;
            parent_id.clone_from(&scenario.info.parent_id);
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
        let timeout = tokio::time::sleep(Duration::from_secs(300));
        tokio::select! {
            opt = node.wait_for_event(event_pattern) => opt.ok_or_else(|| anyhow::anyhow!("wait_for_event: None")),
            _ = timeout => {
                let pending_events = node.pending_events(false).map(|(_, event)| event.to_string()).collect::<Vec<_>>();
                 Err(anyhow::anyhow!("waiting for event timed out! node {node_id:?}, event: \"{event_pattern}\"\n{pending_events:?}"))
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
            info!(system_time(); "Executing step {}/{}", i + 1, total);
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
            .is_some_and(|(scenario, _)| scenario.info.id != target_scenario)
        {
            if !self.exec_next().await? {
                break;
            }
        }

        while self
            .scenario
            .peek()
            .is_some_and(|(scenario, _)| scenario.info.id == target_scenario)
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
                    NonDeterministicEvent::RpcReadonly(id, req) => Event::Rpc(id, req),
                };
                eprintln!("non_deterministic_event_dispatch({node_id:?}): {event}");
                self.nodes
                    .get_mut(node_id.index())
                    .ok_or_else(|| anyhow::anyhow!("node {node_id:?} not found"))?
                    .dispatch_event(event)
            }
            ScenarioStep::AddNode { config } => match *config {
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
                dialer.dispatch_event(Event::Rpc(rpc_id, Box::new(req)))
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
