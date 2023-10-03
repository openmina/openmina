use crate::action::ConsensusAction;
use crate::p2p::connection::outgoing::P2pConnectionOutgoingAction;
use crate::p2p::connection::P2pConnectionAction;
use crate::p2p::disconnection::P2pDisconnectionAction;
use crate::p2p::pubsub::{GossipNetMessageV2, P2pPubsubAction};
use crate::p2p::rpc::outgoing::P2pRpcOutgoingAction;
use crate::p2p::rpc::P2pRpcAction;
use crate::p2p::P2pAction;
use crate::snark::block_verify::SnarkBlockVerifyAction;
use crate::snark::SnarkAction;
use crate::{Action, ActionWithMetaRef, Service, Store, WatchedAccountsAction};

fn gossipnet_message_summary(msg: &GossipNetMessageV2) -> String {
    match msg {
        GossipNetMessageV2::NewState(transition) => {
            let height = transition
                .header
                .protocol_state
                .body
                .consensus_state
                .blockchain_length
                .0
                 .0;
            format!("NewState) height: {}", height)
        }
        GossipNetMessageV2::SnarkPoolDiff { .. } => {
            format!("Gossipsub::SnarkPoolDiff")
        }
        GossipNetMessageV2::TransactionPoolDiff { .. } => {
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
                    P2pConnectionOutgoingAction::Init(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = "PeerConnectionOutgoingInit",
                            summary = format!("peer_id: {}", action.opts.peer_id),
                            peer_id = action.opts.peer_id.to_string(),
                            addrs = format!("{:?}", action.opts.addrs),
                        );
                    }
                    P2pConnectionOutgoingAction::Reconnect(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = "PeerReconnect",
                            summary = format!("peer_id: {}", action.opts.peer_id),
                            peer_id = action.opts.peer_id.to_string(),
                            addrs = format!("{:?}", action.opts.addrs),
                        );
                    }
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
            P2pAction::Disconnection(action) => match action {
                _ => {}
            },
            P2pAction::PeerReady(_) => {}
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
            P2pAction::Rpc(action) => match action {
                P2pRpcAction::Outgoing(action) => match action {
                    P2pRpcOutgoingAction::Init(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = "P2pRpcOutgoingInit",
                            summary = format!("peer_id: {}, rpc_id: {}, kind: {:?}", action.peer_id, action.rpc_id, action.request.kind()),
                            peer_id = action.peer_id.to_string(),
                            rpc_id = action.rpc_id.to_string(),
                            request = serde_json::to_string(&action.request).ok()
                        );
                    }
                    P2pRpcOutgoingAction::Received(action) => {
                        let requestor = None.or_else(|| {
                            let p = store.state().p2p.get_ready_peer(&action.peer_id)?;
                            Some(p.rpc.outgoing.get(action.rpc_id)?.requestor())
                        });
                        let Some(requestor) = requestor else { return };
                        shared::log::info!(
                            meta.time();
                            kind = "P2pRpcOutgoingReceived",
                            summary = format!("peer_id: {}, rpc_id: {}, kind: {:?}", action.peer_id, action.rpc_id, action.response.kind()),
                            peer_id = action.peer_id.to_string(),
                            rpc_id = action.rpc_id.to_string(),
                            requestor = format!("{:?}", requestor),
                            response = serde_json::to_string(&action.response).ok()
                        );
                    }
                    P2pRpcOutgoingAction::Error(action) => {
                        let req = None.or_else(|| {
                            let p = store.state().p2p.get_ready_peer(&action.peer_id)?;
                            Some(p.rpc.outgoing.get(action.rpc_id)?.request())
                        });
                        shared::log::warn!(
                            meta.time();
                            kind = "P2pRpcOutgoingError",
                            summary = format!("peer_id: {}, rpc_id: {}", action.peer_id, action.rpc_id),
                            peer_id = action.peer_id.to_string(),
                            rpc_id = action.rpc_id.to_string(),
                            request = serde_json::to_string(&req).ok(),
                            error = format!("{:?}", action.error)
                        );
                    }
                    P2pRpcOutgoingAction::Success(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = "P2pRpcOutgoingSuccess",
                            summary = format!("peer_id: {}, rpc_id: {}", action.peer_id, action.rpc_id),
                            peer_id = action.peer_id.to_string(),
                            rpc_id = action.rpc_id.to_string(),
                        );
                    }
                    _ => {}
                },
            },
        },
        Action::Snark(action) => match action {
            SnarkAction::BlockVerify(action) => match action {
                SnarkBlockVerifyAction::Init(action) => {
                    let height = action
                        .block
                        .header_ref()
                        .protocol_state
                        .body
                        .consensus_state
                        .blockchain_length
                        .0
                         .0;
                    shared::log::info!(
                        meta.time();
                        kind = "SnarkBlockVerifyInit",
                        summary = format!("height: {}, req_id: {}", height, action.req_id),
                        req_id = action.req_id.to_string(),
                        block = serde_json::to_string(&action.block).ok()
                    );
                }
                SnarkBlockVerifyAction::Error(action) => {
                    let jobs = &store.state().snark.block_verify.jobs;
                    let Some(job) = jobs.get(action.req_id) else {
                        return;
                    };
                    let block = job.block();

                    let height = block
                        .header_ref()
                        .protocol_state
                        .body
                        .consensus_state
                        .blockchain_length
                        .0
                         .0;
                    shared::log::info!(
                        meta.time();
                        kind = "SnarkBlockVerifyError",
                        summary = format!("height: {}, req_id: {}", height, action.req_id),
                        req_id = action.req_id.to_string(),
                        block = serde_json::to_string(&block).ok(),
                        error = format!("{:?}", action.error)
                    );
                }
                SnarkBlockVerifyAction::Success(action) => {
                    let jobs = &store.state().snark.block_verify.jobs;
                    let Some(job) = jobs.get(action.req_id) else {
                        return;
                    };
                    let block = job.block();

                    let height = block
                        .header_ref()
                        .protocol_state
                        .body
                        .consensus_state
                        .blockchain_length
                        .0
                         .0;
                    shared::log::info!(
                        meta.time();
                        kind = "SnarkBlockVerifySuccess",
                        summary = format!("height: {}, req_id: {}", height, action.req_id),
                        req_id = action.req_id.to_string(),
                        block = serde_json::to_string(&block).ok()
                    );
                }
                _ => {}
            },
        },
        Action::Consensus(a) => match a {
            ConsensusAction::BestTipUpdate(_) => {
                let prev = store.state().consensus.previous_best_tip();
                let tip = store.state().consensus.best_tip();
                shared::log::info!(
                    meta.time();
                    kind = "ConsensusBestTipTransition",
                    summary = format!("old_height: {:?}, new_height: {:?}", prev.map(|b| b.height()), tip.map(|b| b.height())),
                    old_hash = prev.map(|b| b.hash.to_string()),
                    new_hash = tip.map(|b| b.hash.to_string()),
                    status = serde_json::to_string(&tip.map(|b| b.status)).ok(),
                );
            }
            _ => {}
        },
        Action::WatchedAccounts(a) => match a {
            WatchedAccountsAction::LedgerInitialStateGetInit(a) => {
                shared::log::info!(
                    meta.time();
                    kind = "WatchedAccountInitialLedgerGetInit",
                    summary = format!("pub_key: {}", a.pub_key),
                );
            }
            WatchedAccountsAction::LedgerInitialStateGetError(a) => {
                shared::log::info!(
                    meta.time();
                    kind = "WatchedAccountInitialLedgerGetError",
                    summary = format!("pub_key: {}", a.pub_key),
                    error = format!("{:?}", a.error)
                );
            }
            WatchedAccountsAction::LedgerInitialStateGetRetry(a) => {
                shared::log::info!(
                    meta.time();
                    kind = "WatchedAccountInitialLedgerGetRetry",
                    summary = format!("pub_key: {}", a.pub_key),
                );
            }
            WatchedAccountsAction::LedgerInitialStateGetSuccess(a) => {
                shared::log::info!(
                    meta.time();
                    kind = "WatchedAccountInitialLedgerGetSuccess",
                    summary = format!("pub_key: {}", a.pub_key),
                    data = serde_json::to_string(&a.data).ok(),
                );
            }
            _ => {}
        },
        _ => {}
    }
}
