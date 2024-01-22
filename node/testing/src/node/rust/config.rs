use std::time::Duration;

use node::{account::AccountSecretKey, BlockProducerConfig};
use serde::{Deserialize, Serialize};

use crate::scenario::ListenerNode;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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
    pub chain_id: String,
    pub initial_time: redux::Timestamp,
    pub max_peers: usize,
    pub ask_initial_peers_interval: Duration,
    pub initial_peers: Vec<ListenerNode>,
    pub peer_id: TestPeerId,
    pub block_producer: Option<RustNodeBlockProducerTestingConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RustNodeBlockProducerTestingConfig {
    pub sec_key: AccountSecretKey,
    pub config: BlockProducerConfig,
}

impl RustNodeTestingConfig {
    pub fn berkeley_default() -> Self {
        Self {
            chain_id: "3c41383994b87449625df91769dff7b507825c064287d30fada9286f3f1cb15e".to_owned(),
            initial_time: redux::Timestamp::ZERO,
            max_peers: 100,
            ask_initial_peers_interval: Duration::from_secs(10),
            initial_peers: Vec::new(),
            peer_id: TestPeerId::default(),
            block_producer: None,
        }
    }

    pub fn max_peers(mut self, n: usize) -> Self {
        self.max_peers = n;
        self
    }

    pub fn chain_id(mut self, s: impl AsRef<str>) -> Self {
        self.chain_id = s.as_ref().to_owned();
        self
    }

    pub fn ask_initial_peers_interval(mut self, d: Duration) -> Self {
        self.ask_initial_peers_interval = d;
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
}
