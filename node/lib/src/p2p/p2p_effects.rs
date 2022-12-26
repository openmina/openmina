use mina_p2p_messages::gossip::GossipNetMessageV2;
use mina_p2p_messages::v2::MinaLedgerSyncLedgerAnswerStableV2;
use p2p::rpc::outgoing::P2pRpcRequestor;

use crate::consensus::ConsensusBlockReceivedAction;
use crate::p2p::rpc::outgoing::P2pRpcOutgoingStatus;
use crate::p2p::rpc::P2pRpcResponse;
use crate::rpc::{RpcP2pConnectionOutgoingErrorAction, RpcP2pConnectionOutgoingSuccessAction};
use crate::snark::hash::state_hash;
use crate::watched_accounts::WatchedAccountsBlockLedgerQuerySuccessAction;
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
                        block: block.into(),
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
                P2pRpcOutgoingAction::Received(action) => {
                    action.effects(&meta, store);
                }
                P2pRpcOutgoingAction::Error(action) => {
                    action.effects(&meta, store);
                }
                P2pRpcOutgoingAction::Success(action) => {
                    let Some(peer) = store.state().p2p.peers.get(&action.peer_id) else { return };
                    let Some(peer) = peer.status.as_ready() else { return };

                    let (_, resp, requestor) = match peer.rpc.outgoing.get(action.rpc_id) {
                        Some(P2pRpcOutgoingStatus::Success {
                            request,
                            response,
                            requestor,
                            ..
                        }) => (request, response, requestor),
                        _ => return,
                    };
                    match resp {
                        P2pRpcResponse::BestTipGet(Some(resp)) => {
                            // TODO(binier): maybe we need to validate best_tip proof (`resp.proof`)?
                            let block = resp.data.clone();
                            store.dispatch(ConsensusBlockReceivedAction {
                                hash: state_hash(&block),
                                block: block.into(),
                            });
                        }
                        P2pRpcResponse::LedgerQuery(resp) => match &resp.0 {
                            Ok(MinaLedgerSyncLedgerAnswerStableV2::AccountWithPath(
                                account,
                                path,
                            )) => match requestor.clone() {
                                P2pRpcRequestor::WatchedAccount(pub_key, block_hash) => {
                                    store.dispatch(WatchedAccountsBlockLedgerQuerySuccessAction {
                                        pub_key,
                                        block_hash,
                                        ledger_account: account.clone(),
                                    });
                                }
                                _ => {}
                            },
                            _ => {}
                        },
                        _ => {}
                    }
                    action.effects(&meta, store);
                }
                P2pRpcOutgoingAction::Finish(_) => {}
            },
        },
    }
}
