use openmina_core::ActionEvent;

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::{token::BroadcastAlgorithm, Data, P2pState, PeerId, StreamId};

use super::pb;

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum P2pNetworkPubsubAction {
    NewStream {
        incoming: bool,
        peer_id: PeerId,
        addr: SocketAddr,
        stream_id: StreamId,
        protocol: BroadcastAlgorithm,
    },
    IncomingData {
        peer_id: PeerId,
        addr: SocketAddr,
        stream_id: StreamId,
        data: Data,
    },
    Broadcast {
        data: Data,
        topic: String,
        key: Option<Data>,
    },
    OutgoingMessage {
        msg: pb::Rpc,
        peer_id: PeerId,
    },
    OutgoingData {
        data: Data,
        peer_id: PeerId,
    },
}

impl From<P2pNetworkPubsubAction> for crate::P2pAction {
    fn from(value: P2pNetworkPubsubAction) -> Self {
        crate::P2pAction::Network(value.into())
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkPubsubAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}
