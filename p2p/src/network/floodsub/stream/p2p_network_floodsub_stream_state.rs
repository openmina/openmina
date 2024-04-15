use crate::network::floodsub::P2pNetworkFloodsub;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum P2pNetworkFloodsubStreamKind {
    Incoming,
    Outgoing,
}

impl From<bool> for P2pNetworkFloodsubStreamKind {
    fn from(incoming: bool) -> Self {
        if incoming {
            P2pNetworkFloodsubStreamKind::Incoming
        } else {
            P2pNetworkFloodsubStreamKind::Outgoing
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum P2pNetworkFloodsubStreamState {
    #[default]
    Default,
    /// Wait for messages in inbound stream
    WaitForInput,
    /// A portion of data from the stream is received.
    IncomingPartialData {
        len: usize,
        data: Vec<u8>,
    },
    // Identify message fully received from remote peer
    MessageReceived {
        data: P2pNetworkFloodsub,
    },
    /// After the outbound connection is established we should send the topics we want to subscribe to.
    SendSubscriptions,
    /// Error handling the stream.
    Error(String),
}

impl P2pNetworkFloodsubStreamState {
    pub fn new() -> Self {
        P2pNetworkFloodsubStreamState::Default
    }
}

impl From<P2pNetworkFloodsub> for P2pNetworkFloodsubStreamState {
    fn from(data: P2pNetworkFloodsub) -> Self {
        P2pNetworkFloodsubStreamState::MessageReceived { data }
    }
}
