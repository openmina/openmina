use super::pb;
use crate::{token::BroadcastAlgorithm, ConnectionAddr, Data, P2pState, PeerId, StreamId};
use mina_p2p_messages::gossip::GossipNetMessageV2;
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum P2pNetworkPubsubAction {
    NewStream {
        incoming: bool,
        peer_id: PeerId,
        addr: ConnectionAddr,
        stream_id: StreamId,
        protocol: BroadcastAlgorithm,
    },
    IncomingData {
        peer_id: PeerId,
        addr: ConnectionAddr,
        stream_id: StreamId,
        data: Data,
        seen_limit: usize,
    },
    IncomingMessage {
        peer_id: PeerId,
        message: pb::Message,
        seen_limit: usize,
    },
    Graft {
        peer_id: PeerId,
        topic_id: String,
    },
    Prune {
        peer_id: PeerId,
        topic_id: String,
    },
    Broadcast {
        message: Box<GossipNetMessageV2>,
    },
    Sign {
        seqno: u64,
        author: PeerId,
        data: Data,
        topic: String,
    },
    #[action_event(level = warn, fields(display(author), display(topic)))]
    SignError {
        author: PeerId,
        topic: String,
    },
    BroadcastSigned {
        signature: Data,
    },
    OutgoingMessage {
        msg: pb::Rpc,
        peer_id: PeerId,
    },
    #[action_event(level = warn, fields(display(peer_id), debug(msg)))]
    OutgoingMessageError {
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
