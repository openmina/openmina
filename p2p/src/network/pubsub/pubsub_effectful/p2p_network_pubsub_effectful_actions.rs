use crate::{pubsub::pb::Message, ConnectionAddr, P2pState, PeerId};
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

/// Eeffectful actions within the P2P Network PubSub system.
#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum P2pNetworkPubsubEffectfulAction {
    /// Initiate the signing of a message before broadcasting.
    ///
    /// **Fields:**
    /// - `author`: The identifier of the peer authoring the message.
    /// - `message`: The protobuf message to be signed.
    Sign { author: PeerId, message: Message },

    /// Validate a batch of incoming messages from a peer.
    ///
    /// **Fields:**
    /// - `peer_id`: The identifier of the peer sending the messages.
    /// - `seen_limit`: The limit for tracking seen messages to prevent duplication.
    /// - `addr`: The connection address of the peer.
    /// - `messages`: Decoded protobuf messages.
    ValidateIncomingMessages {
        peer_id: PeerId,
        seen_limit: usize,
        addr: ConnectionAddr,
        messages: Vec<Message>,
    },
}

impl From<P2pNetworkPubsubEffectfulAction> for crate::P2pEffectfulAction {
    fn from(value: P2pNetworkPubsubEffectfulAction) -> crate::P2pEffectfulAction {
        crate::P2pEffectfulAction::Network(crate::P2pNetworkEffectfulAction::Pubsub(value))
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkPubsubEffectfulAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}
