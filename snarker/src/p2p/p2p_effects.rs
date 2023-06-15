use p2p::channels::rpc::P2pRpcResponse;
use snark::hash::state_hash;

use crate::consensus::ConsensusBlockReceivedAction;
use crate::job_commitment::JobCommitmentAddAction;
use crate::p2p::channels::rpc::{P2pChannelsRpcRequestSendAction, P2pRpcRequest};
use crate::rpc::{
    RpcP2pConnectionIncomingErrorAction, RpcP2pConnectionIncomingRespondAction,
    RpcP2pConnectionIncomingSuccessAction, RpcP2pConnectionOutgoingErrorAction,
    RpcP2pConnectionOutgoingSuccessAction,
};
use crate::watched_accounts::{
    WatchedAccountLedgerInitialState, WatchedAccountsLedgerInitialStateGetError,
    WatchedAccountsLedgerInitialStateGetErrorAction,
};
use crate::{Service, Store};

use super::channels::best_tip::{P2pChannelsBestTipAction, P2pChannelsBestTipResponseSendAction};
use super::channels::rpc::P2pChannelsRpcAction;
use super::channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentAction;
use super::channels::P2pChannelsAction;
use super::connection::incoming::{
    P2pConnectionIncomingAction, P2pConnectionIncomingAnswerSendSuccessAction,
};
use super::connection::outgoing::P2pConnectionOutgoingAction;
use super::connection::{P2pConnectionAction, P2pConnectionResponse};
use super::disconnection::P2pDisconnectionAction;
use super::{P2pAction, P2pActionWithMeta};

pub fn p2p_effects<S: Service>(store: &mut Store<S>, action: P2pActionWithMeta) {
    let (action, meta) = action.split();

    match action {
        P2pAction::Connection(action) => match action {
            P2pConnectionAction::Outgoing(action) => match action {
                P2pConnectionOutgoingAction::RandomInit(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::Init(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::Reconnect(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::OfferSdpCreatePending(_) => {}
                P2pConnectionOutgoingAction::OfferSdpCreateError(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::OfferSdpCreateSuccess(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::OfferReady(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::OfferSendSuccess(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::AnswerRecvPending(_) => {}
                P2pConnectionOutgoingAction::AnswerRecvError(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::AnswerRecvSuccess(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::FinalizePending(_) => {}
                P2pConnectionOutgoingAction::FinalizeError(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::FinalizeSuccess(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::Error(action) => {
                    let p2p = &store.state().p2p;
                    if let Some(rpc_id) = p2p.peer_connection_rpc_id(&action.peer_id) {
                        store.dispatch(RpcP2pConnectionOutgoingErrorAction {
                            rpc_id,
                            error: action.error.clone(),
                        });
                    }
                    // action.effects(&meta, store);
                }
                P2pConnectionOutgoingAction::Success(action) => {
                    let p2p = &store.state().p2p;
                    if let Some(rpc_id) = p2p.peer_connection_rpc_id(&action.peer_id) {
                        store.dispatch(RpcP2pConnectionOutgoingSuccessAction { rpc_id });
                    }
                    action.effects(&meta, store);
                }
            },
            P2pConnectionAction::Incoming(action) => match action {
                P2pConnectionIncomingAction::Init(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionIncomingAction::AnswerSdpCreatePending(_) => {}
                P2pConnectionIncomingAction::AnswerSdpCreateError(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionIncomingAction::AnswerSdpCreateSuccess(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionIncomingAction::AnswerReady(action) => {
                    let p2p = &store.state().p2p;
                    if let Some(rpc_id) = p2p.peer_connection_rpc_id(&action.peer_id) {
                        store.dispatch(RpcP2pConnectionIncomingRespondAction {
                            rpc_id,
                            response: P2pConnectionResponse::Accepted(action.answer.clone()),
                        });
                        store.dispatch(P2pConnectionIncomingAnswerSendSuccessAction {
                            peer_id: action.peer_id,
                        });
                    }
                    action.effects(&meta, store);
                }
                P2pConnectionIncomingAction::AnswerSendSuccess(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionIncomingAction::FinalizePending(_) => {}
                P2pConnectionIncomingAction::FinalizeError(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionIncomingAction::FinalizeSuccess(action) => {
                    action.effects(&meta, store);
                }
                P2pConnectionIncomingAction::Error(action) => {
                    let p2p = &store.state().p2p;
                    if let Some(rpc_id) = p2p.peer_connection_rpc_id(&action.peer_id) {
                        store.dispatch(RpcP2pConnectionIncomingErrorAction {
                            rpc_id,
                            error: format!("{:?}", action.error),
                        });
                    }
                    // action.effects(&meta, store);
                }
                P2pConnectionIncomingAction::Success(action) => {
                    let p2p = &store.state().p2p;
                    if let Some(rpc_id) = p2p.peer_connection_rpc_id(&action.peer_id) {
                        store.dispatch(RpcP2pConnectionIncomingSuccessAction { rpc_id });
                    }
                    action.effects(&meta, store);
                }
            },
        },
        P2pAction::Disconnection(action) => {
            match action {
                P2pDisconnectionAction::Init(action) => action.effects(&meta, store),
                P2pDisconnectionAction::Finish(action) => {
                    let actions = store.state().watched_accounts.iter()
                    .filter_map(|(pub_key, a)| match &a.initial_state {
                        WatchedAccountLedgerInitialState::Pending { peer_id, .. } => {
                            if peer_id == &action.peer_id {
                                Some(WatchedAccountsLedgerInitialStateGetErrorAction {
                                    pub_key: pub_key.clone(),
                                    error: WatchedAccountsLedgerInitialStateGetError::PeerDisconnected,
                                })
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();

                    for action in actions {
                        store.dispatch(action);
                    }
                }
            }
        }
        P2pAction::PeerReady(action) => {
            action.effects(&meta, store);
        }
        P2pAction::Channels(action) => match action {
            P2pChannelsAction::MessageReceived(action) => {
                action.effects(&meta, store);
            }
            P2pChannelsAction::BestTip(action) => match action {
                P2pChannelsBestTipAction::Init(action) => {
                    action.effects(&meta, store);
                }
                P2pChannelsBestTipAction::Pending(_) => {}
                P2pChannelsBestTipAction::Ready(action) => {
                    action.effects(&meta, store);
                }
                P2pChannelsBestTipAction::RequestSend(action) => {
                    action.effects(&meta, store);
                }
                P2pChannelsBestTipAction::Received(action) => {
                    action.effects(&meta, store);
                    store.dispatch(ConsensusBlockReceivedAction {
                        hash: action.best_tip.hash,
                        block: action.best_tip.block,
                        history: None,
                    });
                }
                P2pChannelsBestTipAction::RequestReceived(action) => {
                    if let Some(best_tip) = store.state().consensus.best_tip_block_with_hash() {
                        store.dispatch(P2pChannelsBestTipResponseSendAction {
                            peer_id: action.peer_id,
                            best_tip,
                        });
                    }
                }
                P2pChannelsBestTipAction::ResponseSend(action) => {
                    action.effects(&meta, store);
                }
            },
            P2pChannelsAction::SnarkJobCommitment(action) => match action {
                P2pChannelsSnarkJobCommitmentAction::Init(action) => {
                    action.effects(&meta, store);
                }
                P2pChannelsSnarkJobCommitmentAction::Pending(_) => {}
                P2pChannelsSnarkJobCommitmentAction::Ready(action) => {
                    action.effects(&meta, store);
                }
                P2pChannelsSnarkJobCommitmentAction::RequestSend(action) => {
                    action.effects(&meta, store);
                }
                P2pChannelsSnarkJobCommitmentAction::PromiseReceived(_) => {}
                P2pChannelsSnarkJobCommitmentAction::Received(action) => {
                    action.effects(&meta, store);
                    store.dispatch(JobCommitmentAddAction {
                        commitment: action.commitment,
                        sender: action.peer_id,
                    });
                }
                P2pChannelsSnarkJobCommitmentAction::RequestReceived(_) => {}
                P2pChannelsSnarkJobCommitmentAction::ResponseSend(action) => {
                    action.effects(&meta, store);
                }
            },
            P2pChannelsAction::Rpc(action) => match action {
                P2pChannelsRpcAction::Init(action) => {
                    action.effects(&meta, store);
                }
                P2pChannelsRpcAction::Pending(_) => {}
                P2pChannelsRpcAction::Ready(a) => {
                    let request = P2pRpcRequest::BestTipWithProofGet;
                    store.dispatch(P2pChannelsRpcRequestSendAction {
                        peer_id: a.peer_id,
                        id: 0,
                        request,
                    });
                }
                P2pChannelsRpcAction::RequestSend(action) => {
                    action.effects(&meta, store);
                }
                P2pChannelsRpcAction::ResponseReceived(action) => match action.response {
                    Some(P2pRpcResponse::BestTipWithProofGet(resp)) => {
                        let hash = state_hash(&resp.best_tip.header);
                        store.dispatch(ConsensusBlockReceivedAction {
                            hash,
                            block: resp.best_tip,
                            history: None,
                        });
                    }
                    _ => {}
                },
                P2pChannelsRpcAction::RequestReceived(action) => {
                    // TODO(binier): handle incoming rpc requests.
                }
                P2pChannelsRpcAction::ResponseSend(action) => {
                    action.effects(&meta, store);
                }
            },
        },
    }
}
