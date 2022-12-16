use mina_p2p_messages::gossip::GossipNetMessageV2;
use snark::hash::state_hash;

use crate::consensus::ConsensusBlockReceivedAction;
use crate::p2p::rpc::P2pRpcResponse;
use crate::rpc::{RpcP2pConnectionOutgoingErrorAction, RpcP2pConnectionOutgoingSuccessAction};
use crate::{Service, Store};

use super::connection::outgoing::P2pConnectionOutgoingAction;
use super::connection::P2pConnectionAction;
use super::pubsub::P2pPubsubAction;
use super::rpc::outgoing::P2pRpcOutgoingAction;
use super::rpc::P2pRpcAction;
use super::{P2pAction, P2pActionWithMeta};

pub fn p2p_effects<S: Service>(store: &mut Store<S>, action: P2pActionWithMeta) {
    let (action, meta) = action.split();

    match action {
        P2pAction::Connection(action) => match action {
            P2pConnectionAction::Outgoing(action) => match action {
                P2pConnectionOutgoingAction::Init(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::Pending(_) => {
                    // action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::Error(action) => {
                    let p2p = &store.state().p2p;
                    if let Some(rpc_id) = p2p.peer_connection_outgoing_rpc_id(&action.peer_id) {
                        store.dispatch(RpcP2pConnectionOutgoingErrorAction {
                            rpc_id,
                            error: action.error.clone(),
                        });
                    }

                    // action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::Success(action) => {
                    let p2p = &store.state().p2p;
                    if let Some(rpc_id) = p2p.peer_connection_outgoing_rpc_id(&action.peer_id) {
                        store.dispatch(RpcP2pConnectionOutgoingSuccessAction { rpc_id });
                    }
                    action.effects(&meta, store);
                }
            },
        },
        P2pAction::PeerReady(action) => {
            action.effects(&meta, store);
        }
        P2pAction::Pubsub(action) => match action {
            P2pPubsubAction::MessagePublish(action) => {
                action.effects(&meta, store);
            }
            P2pPubsubAction::BytesPublish(action) => {
                let rpc_id = action.rpc_id;
                action.effects(&meta, store);
                if let Some(rpc_id) = rpc_id {
                    // TODO(binier)
                    let _ = store.service.respond_p2p_publish(rpc_id, Ok(()));
                }
            }
            P2pPubsubAction::BytesReceived(action) => {
                action.effects(&meta, store);
            }
            P2pPubsubAction::MessageReceived(action) => match action.message {
                GossipNetMessageV2::NewState(block) => {
                    store.dispatch(ConsensusBlockReceivedAction {
                        hash: state_hash(&block),
                        header: block.header,
                    });
                }
                _ => {}
            },
        },
        P2pAction::Rpc(action) => match action {
            P2pRpcAction::Outgoing(action) => match action {
                P2pRpcOutgoingAction::Init(action) => {
                    action.effects(&meta, store);
                }
                P2pRpcOutgoingAction::Pending(_) => {}
                P2pRpcOutgoingAction::Error(action) => {
                    action.effects(&meta, store);
                }
                P2pRpcOutgoingAction::Success(action) => {
                    if let P2pRpcResponse::BestTipGet(Some(resp)) = &action.response {
                        // TODO(binier): maybe we need to validate best_tip proof (`resp.proof`)?
                        let header = resp.data.header.clone();
                        store.dispatch(ConsensusBlockReceivedAction {
                            hash: state_hash(&header),
                            header,
                        });
                    }
                    action.effects(&meta, store);
                }
                P2pRpcOutgoingAction::Finish(_) => {}
            },
        },
    }
}
