use crate::p2p::connection::outgoing::P2pConnectionOutgoingAction;
use crate::p2p::connection::P2pConnectionAction;
use crate::p2p::pubsub::{GossipNetMessageV1, P2pPubsubAction};
use crate::p2p::P2pAction;
use crate::{Action, ActionWithMetaRef, Service, Store};

fn gossipnet_message_summary(msg: &GossipNetMessageV1) -> String {
    match msg {
        GossipNetMessageV1::NewState(transition) => {
            let height = transition
                .inner()
                .protocol_state
                .inner()
                .0
                .inner()
                .body
                .inner()
                .0
                .inner()
                .consensus_state
                .inner()
                .0
                .inner()
                .blockchain_length
                .inner()
                .0
                .inner()
                .0
                 .0;
            format!("NewState) height: {}", height)
        }
        GossipNetMessageV1::SnarkPoolDiff(_) => {
            format!("Gossipsub::SnarkPoolDiff")
        }
        GossipNetMessageV1::TransactionPoolDiff(_) => {
            format!("Gossipsub::TransactionPoolDiff")
        }
    }
}

pub fn logger_effects<S: Service>(store: &Store<S>, action: ActionWithMetaRef<'_>) {
    let (action, meta) = action.split();

    match action {
        Action::P2p(action) => match action {
            P2pAction::Connection(action) => match action {
                P2pConnectionAction::Outgoing(action) => match action {
                    P2pConnectionOutgoingAction::Error(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = "PeerConnectionOutgoingError",
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            error = action.error
                        );
                    }
                    _ => {}
                },
            },

            P2pAction::Pubsub(action) => match action {
                P2pPubsubAction::MessagePublish(action) => {
                    shared::log::info!(
                        meta.time();
                        kind = "P2pPubsubMessagePublish",
                        summary = gossipnet_message_summary(&action.message),
                        rpc_id = action.rpc_id.map(|v| v.to_string()),
                        message = serde_json::to_string(&action.message).ok()
                    );
                }
                P2pPubsubAction::MessageReceived(action) => {
                    shared::log::info!(
                        meta.time();
                        kind = "P2pPubsubMessageReceive",
                        summary = gossipnet_message_summary(&action.message),
                        author = action.author.to_string(),
                        sender = action.sender.to_string(),
                        message = serde_json::to_string(&action.message).ok()
                    );
                }
                _ => {}
            },
        },
        _ => {}
    }
}
