use crate::network::identify::P2pNetworkIdentify;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum P2pNetworkIdentifyStreamKind {
    Incoming,
    Outgoing,
}

impl From<bool> for P2pNetworkIdentifyStreamKind {
    fn from(incoming: bool) -> Self {
        if incoming {
            P2pNetworkIdentifyStreamKind::Incoming
        } else {
            P2pNetworkIdentifyStreamKind::Outgoing
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum P2pNetworkIdentifyStreamState {
    #[default]
    Default,
    /// Prepare to receive the Identify message from the remote peer
    RecvIdentify,
    /// Prepare to send the Identify messge to the remote peer.
    SendIdentify,
    /// A portion of data from the stream is received.
    IncomingPartialData {
        len: usize,
        data: Vec<u8>,
    },
    // Identify message fully received from remote peer
    IdentifyReceived {
        data: P2pNetworkIdentify,
    },
    /// Error handling the stream.
    Error(String),
}

impl P2pNetworkIdentifyStreamState {
    pub fn new() -> Self {
        P2pNetworkIdentifyStreamState::Default
    }
}

impl From<P2pNetworkIdentify> for P2pNetworkIdentifyStreamState {
    fn from(data: P2pNetworkIdentify) -> Self {
        P2pNetworkIdentifyStreamState::IdentifyReceived { data }
    }
}
