use super::{p2p_network_pubsub_state::P2pNetworkPubsubMessageCacheId, pb, BroadcastMessageId};
use crate::{token::BroadcastAlgorithm, ConnectionAddr, Data, P2pState, PeerId, StreamId};
use mina_p2p_messages::gossip::GossipNetMessageV2;
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

/// Actions that can occur within the P2P Network PubSub system.
///
/// Managing pubsub streams, handling incoming and outgoing messages,
/// and maintaining the mesh network topology.
///
/// **Common Fields:**
/// - `peer_id`: The identifier of the peer associated with the action.
/// - `addr`: The connection address of the peer.
/// - `stream_id`: The unique identifier of the stream.
/// - `topic_id`: The identifier of the topic involved in the action.
#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum P2pNetworkPubsubAction {
    /// Create a new stream, either incoming or outgoing.
    ///
    /// **Fields:**
    /// - `incoming`: Indicates if the stream is incoming (`true`) or outgoing (`false`).
    /// - `protocol`: The broadcast algorithm used for the stream.
    NewStream {
        incoming: bool,
        peer_id: PeerId,
        addr: ConnectionAddr,
        stream_id: StreamId,
        protocol: BroadcastAlgorithm,
    },

    /// Process incoming raw data from a peer.
    ///
    /// **Fields:**
    /// - `data`: The raw data payload received.
    /// - `seen_limit`: The limit for tracking seen messages to prevent duplication.
    IncomingData {
        peer_id: PeerId,
        addr: ConnectionAddr,
        stream_id: StreamId,
        data: Data,
        seen_limit: usize,
    },

    /// Validate a batch of decoded incoming messages.
    ValidateIncomingMessages {
        peer_id: PeerId,
        seen_limit: usize,
        addr: ConnectionAddr,
    },

    /// Handle a fully decoded and validated message received from a peer.
    ///
    /// **Fields:**
    /// - `message`: The decoded protobuf message.
    /// - `seen_limit`: The limit for tracking seen messages to prevent duplication.
    IncomingMessage {
        peer_id: PeerId,
        message: pb::Message,
        seen_limit: usize,
    },

    /// Clean up temporary states after processing an incoming message.
    IncomingMessageCleanup {
        peer_id: PeerId,
    },

    /// Add a peer to the mesh network for a specific topic.
    Graft {
        peer_id: PeerId,
        topic_id: String,
    },

    /// Remove a peer from the mesh network for a specific topic.
    Prune {
        peer_id: PeerId,
        topic_id: String,
    },

    /// Rebroadcast message received from WebRTC connection.
    WebRtcRebroadcast {
        message: GossipNetMessageV2,
    },

    /// Initiate the broadcasting of a message to all subscribed peers.
    ///
    /// **Fields:**
    /// - `message`: The gossip network message to broadcast.
    Broadcast {
        message: GossipNetMessageV2,
    },

    /// Prepare a message for signing before broadcasting.
    ///
    /// **Fields:**
    /// - `seqno`: The sequence number of the message.
    /// - `author`: The identifier of the peer authoring the message.
    /// - `data`: The data payload of the message.
    /// - `topic`: The topic under which the message is published.
    Sign {
        seqno: u64,
        author: PeerId,
        data: Data,
        topic: String,
    },

    /// An error occured during the signing process.
    #[action_event(level = warn, fields(display(author), display(topic)))]
    SignError {
        author: PeerId,
        topic: String,
    },

    /// Finalize the broadcasting of a signed message by attaching the signature.
    ///
    /// **Fields:**
    /// - `signature`: The cryptographic signature of the message.
    BroadcastSigned {
        signature: Data,
    },

    /// Prepare an outgoing message to send to a specific peer.
    OutgoingMessage {
        peer_id: PeerId,
    },

    /// Clear the outgoing message state for a specific peer after sending.
    OutgoingMessageClear {
        peer_id: PeerId,
    },

    /// An error occured during the sending of an outgoing message.
    ///
    /// **Fields:**
    /// - `msg`: The protobuf message that failed to send.
    #[action_event(level = warn, fields(display(peer_id), debug(msg)))]
    OutgoingMessageError {
        msg: pb::Rpc,
        peer_id: PeerId,
    },

    /// Send encoded data over an outgoing stream to a specific peer.
    ///
    /// **Fields:**
    /// - `data`: The encoded data to be sent.
    OutgoingData {
        data: Data,
        peer_id: PeerId,
    },

    HandleIncomingMessage {
        message: pb::Message,
        message_content: GossipNetMessageV2,
        peer_id: PeerId,
    },

    ValidateIncomingMessage {
        message_id: P2pNetworkPubsubMessageCacheId,
    },

    /// Delete expired messages from state
    PruneMessages {},

    RejectMessage {
        message_id: Option<BroadcastMessageId>,
        peer_id: Option<PeerId>,
        reason: String,
    },
    IgnoreMessage {
        message_id: Option<BroadcastMessageId>,
        reason: String,
    },

    // After message is fully validated, broadcast it to other peers
    BroadcastValidatedMessage {
        message_id: BroadcastMessageId,
    },
}

impl From<P2pNetworkPubsubAction> for crate::P2pAction {
    fn from(value: P2pNetworkPubsubAction) -> Self {
        crate::P2pAction::Network(value.into())
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkPubsubAction {
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
        let pubsub = &state.network.scheduler.broadcast_state;
        match self {
            P2pNetworkPubsubAction::OutgoingMessage { peer_id } => pubsub
                .clients
                .get(peer_id)
                .is_some_and(|s| !s.message_is_empty()),
            P2pNetworkPubsubAction::Prune { peer_id, topic_id } => pubsub
                .topics
                .get(topic_id)
                .is_some_and(|topics| topics.contains_key(peer_id)),
            P2pNetworkPubsubAction::WebRtcRebroadcast { message } => {
                let source = super::webrtc_source_sk(message)
                    .public_key()
                    .peer_id()
                    .try_into()
                    .unwrap();
                pubsub
                    .mcache
                    .get_message(&P2pNetworkPubsubMessageCacheId { source, seqno: 0 })
                    .is_none()
            }
            P2pNetworkPubsubAction::BroadcastValidatedMessage { message_id }
            | P2pNetworkPubsubAction::RejectMessage {
                message_id: Some(message_id),
                ..
            } => pubsub.mcache.contains_broadcast_id(message_id),
            _ => true,
        }
    }
}
