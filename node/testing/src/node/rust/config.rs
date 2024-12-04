use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use node::account::AccountSecretKey;
use node::config::DEVNET_CONFIG;
use node::transition_frontier::genesis::GenesisConfig;
use node::{p2p::P2pTimeouts, BlockProducerConfig, SnarkerConfig};
use serde::{Deserialize, Serialize};

use crate::scenario::ListenerNode;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub enum TestPeerId {
    /// NOTE This option results a deterministic private key derived from the
    /// node index in the cluster. Be aware that when interacting with OCaml
    /// nodes or other nodes outside the cluster might be interfer by previous
    /// runs (e.g. peer_id might be blacklisted).
    #[default]
    Derived,
    Bytes([u8; 32]),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RustNodeTestingConfig {
    pub initial_time: redux::Timestamp,
    pub genesis: Arc<GenesisConfig>,
    pub max_peers: usize,
    #[serde(default)]
    pub initial_peers: Vec<ListenerNode>,
    #[serde(default)]
    pub peer_id: TestPeerId,
    #[serde(default)]
    pub snark_worker: Option<SnarkerConfig>,
    #[serde(default)]
    pub block_producer: Option<RustNodeBlockProducerTestingConfig>,
    #[serde(default)]
    pub timeouts: P2pTimeouts,
    #[serde(default)]
    pub libp2p_port: Option<u16>,
    #[serde(default)]
    pub recorder: Recorder,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub enum Recorder {
    #[default]
    None,
    StateWithInputActions,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RustNodeBlockProducerTestingConfig {
    pub sec_key: AccountSecretKey,
    pub config: BlockProducerConfig,
}

impl RustNodeTestingConfig {
    pub fn devnet_default() -> Self {
        Self {
            initial_time: redux::Timestamp::ZERO,
            genesis: DEVNET_CONFIG.clone(),
            max_peers: 100,
            initial_peers: Vec::new(),
            peer_id: TestPeerId::default(),
            block_producer: None,
            snark_worker: None,
            timeouts: P2pTimeouts::default(),
            libp2p_port: None,
            recorder: Default::default(),
        }
    }

    pub fn devnet_default_no_rpc_timeouts() -> Self {
        Self {
            initial_time: redux::Timestamp::ZERO,
            genesis: DEVNET_CONFIG.clone(),
            max_peers: 100,
            initial_peers: Vec::new(),
            peer_id: TestPeerId::default(),
            block_producer: None,
            snark_worker: None,
            timeouts: P2pTimeouts::without_rpc(),
            libp2p_port: None,
            recorder: Default::default(),
        }
    }

    pub fn max_peers(mut self, n: usize) -> Self {
        self.max_peers = n;
        self
    }

    pub fn initial_peers(mut self, v: Vec<ListenerNode>) -> Self {
        self.initial_peers = v;
        self
    }

    pub fn with_peer_id(mut self, bytes: [u8; 32]) -> Self {
        self.peer_id = TestPeerId::Bytes(bytes);
        self
    }

    pub fn with_timeouts(mut self, timeouts: P2pTimeouts) -> Self {
        self.timeouts = timeouts;
        self
    }

    pub fn with_libp2p_port(mut self, libp2p_port: u16) -> Self {
        self.libp2p_port = Some(libp2p_port);
        self
    }

    pub fn with_daemon_json<P: AsRef<Path>>(mut self, daemon_json: P) -> Self {
        self.genesis = Arc::new(GenesisConfig::DaemonJson(
            serde_json::from_reader(&mut File::open(daemon_json).expect("daemon json file"))
                .expect("daemon json"),
        ));
        self
    }
}
