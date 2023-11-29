use std::time::Duration;

use node::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum NodeTestingConfig {
    Rust(RustNodeTestingConfig),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RustNodeTestingConfig {
    pub chain_id: String,
    pub initial_time: redux::Timestamp,
    pub max_peers: usize,
    pub ask_initial_peers_interval: Duration,
    pub initial_peers: Vec<P2pConnectionOutgoingInitOpts>,
    pub libp2p_port: Option<u16>,
    pub randomize_peer_id: bool,
}

impl RustNodeTestingConfig {
    pub fn berkeley_default() -> Self {
        Self {
            chain_id: "3c41383994b87449625df91769dff7b507825c064287d30fada9286f3f1cb15e".to_owned(),
            initial_time: redux::Timestamp::ZERO,
            max_peers: 100,
            ask_initial_peers_interval: Duration::from_secs(10),
            initial_peers: vec![],
            libp2p_port: None,
            randomize_peer_id: false,
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

    pub fn initial_peers(mut self, v: Vec<P2pConnectionOutgoingInitOpts>) -> Self {
        self.initial_peers = v;
        self
    }

    pub fn libp2p_port(mut self, v: u16) -> Self {
        self.libp2p_port = Some(v);
        self
    }

    pub fn randomize_peer_id(mut self) -> Self {
        self.randomize_peer_id = true;
        self
    }
}
