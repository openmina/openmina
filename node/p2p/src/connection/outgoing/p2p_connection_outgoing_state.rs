use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionOutgoingState {
    Init {
        time: redux::Timestamp,
        addrs: Vec<libp2p::Multiaddr>,
    },
    Pending {
        time: redux::Timestamp,
    },
    Error {
        time: redux::Timestamp,
        error: String,
    },
    Success {
        time: redux::Timestamp,
    },
}

impl Default for P2pConnectionOutgoingState {
    fn default() -> Self {
        Self::Init {
            time: redux::Timestamp::ZERO,
            addrs: vec![],
        }
    }
}
