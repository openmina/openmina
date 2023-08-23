use crate::p2p::channels::best_tip::P2pChannelsBestTipAction;
use crate::p2p::channels::rpc::P2pChannelsRpcAction;
use crate::p2p::channels::snark::P2pChannelsSnarkAction;
use crate::p2p::channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentAction;
use crate::p2p::channels::P2pChannelsAction;
use crate::p2p::connection::incoming::P2pConnectionIncomingAction;
use crate::p2p::connection::outgoing::P2pConnectionOutgoingAction;
use crate::p2p::connection::P2pConnectionAction;
use crate::p2p::disconnection::P2pDisconnectionAction;
use crate::p2p::P2pAction;
use crate::snark::work_verify::SnarkWorkVerifyAction;
use crate::snark::SnarkAction;
use crate::{Action, ActionWithMetaRef, Service, Store};

pub fn logger_effects<S: Service>(store: &Store<S>, action: ActionWithMetaRef<'_>) {
    let (action, meta) = action.split();
    let kind = action.kind();

    match action {
        Action::P2p(action) => match action {
            P2pAction::Connection(action) => match action {
                P2pConnectionAction::Outgoing(action) => match action {
                    P2pConnectionOutgoingAction::RandomInit(_) => {}
                    P2pConnectionOutgoingAction::Init(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.opts.peer_id()),
                            peer_id = action.opts.peer_id().to_string(),
                            transport = action.opts.kind(),
                        );
                    }
                    P2pConnectionOutgoingAction::Reconnect(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.opts.peer_id()),
                            peer_id = action.opts.peer_id().to_string(),
                            transport = action.opts.kind(),
                        );
                    }
                    P2pConnectionOutgoingAction::OfferSdpCreatePending(_) => {}
                    P2pConnectionOutgoingAction::OfferSdpCreateError(action) => {
                        shared::log::warn!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            error = action.error.clone(),
                        );
                    }
                    P2pConnectionOutgoingAction::OfferSdpCreateSuccess(action) => {
                        shared::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            sdp = action.sdp.clone(),
                        );
                    }
                    P2pConnectionOutgoingAction::OfferReady(action) => {
                        shared::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            offer = serde_json::to_string(&action.offer).ok()
                        );
                    }
                    P2pConnectionOutgoingAction::OfferSendSuccess(action) => {
                        shared::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                        );
                    }
                    P2pConnectionOutgoingAction::AnswerRecvPending(_) => {}
                    P2pConnectionOutgoingAction::AnswerRecvError(action) => {
                        shared::log::warn!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            error = format!("{:?}", action.error),
                        );
                    }
                    P2pConnectionOutgoingAction::AnswerRecvSuccess(action) => {
                        shared::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            trace_answer = serde_json::to_string(&action.answer).ok()
                        );
                    }
                    P2pConnectionOutgoingAction::FinalizePending(_) => {}
                    P2pConnectionOutgoingAction::FinalizeError(action) => {
                        shared::log::warn!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            error = action.error.clone(),
                        );
                    }
                    P2pConnectionOutgoingAction::FinalizeSuccess(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pConnectionOutgoingAction::Timeout(action) => {
                        shared::log::warn!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pConnectionOutgoingAction::Error(action) => {
                        shared::log::warn!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            error = format!("{:?}", action.error),
                        );
                    }
                    P2pConnectionOutgoingAction::Success(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                },
                P2pConnectionAction::Incoming(action) => match action {
                    P2pConnectionIncomingAction::Init(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.opts.peer_id),
                            peer_id = action.opts.peer_id.to_string(),
                            trace_signaling = format!("{:?}", action.opts.signaling),
                        );
                    }
                    P2pConnectionIncomingAction::AnswerSdpCreatePending(_) => {}
                    P2pConnectionIncomingAction::AnswerSdpCreateError(action) => {
                        shared::log::warn!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            error = format!("{:?}", action.error),
                        );
                    }
                    P2pConnectionIncomingAction::AnswerSdpCreateSuccess(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            trace_sdp = action.sdp.clone(),
                        );
                    }
                    P2pConnectionIncomingAction::AnswerReady(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            trace_answer = serde_json::to_string(&action.answer).ok()
                        );
                    }
                    P2pConnectionIncomingAction::AnswerSendSuccess(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pConnectionIncomingAction::FinalizePending(_) => {}
                    P2pConnectionIncomingAction::FinalizeError(action) => {
                        shared::log::warn!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            error = format!("{:?}", action.error),
                        );
                    }
                    P2pConnectionIncomingAction::FinalizeSuccess(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pConnectionIncomingAction::Timeout(action) => {
                        shared::log::warn!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pConnectionIncomingAction::Error(action) => {
                        shared::log::warn!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            error = format!("{:?}", action.error),
                        );
                    }
                    P2pConnectionIncomingAction::Success(action) => {
                        shared::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                },
            },
            P2pAction::Disconnection(action) => match action {
                P2pDisconnectionAction::Init(action) => {
                    shared::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("peer_id: {}", action.peer_id),
                        peer_id = action.peer_id.to_string()
                    );
                }
                P2pDisconnectionAction::Finish(action) => {
                    shared::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("peer_id: {}", action.peer_id),
                        peer_id = action.peer_id.to_string()
                    );
                }
            },
            P2pAction::Channels(action) => match action {
                P2pChannelsAction::MessageReceived(_) => {}
                P2pChannelsAction::BestTip(action) => match action {
                    P2pChannelsBestTipAction::Init(action) => {
                        shared::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pChannelsBestTipAction::Ready(action) => {
                        shared::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    _ => {}
                },
                P2pChannelsAction::Snark(action) => match action {
                    P2pChannelsSnarkAction::Init(action) => {
                        shared::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pChannelsSnarkAction::Ready(action) => {
                        shared::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    _ => {}
                },
                P2pChannelsAction::SnarkJobCommitment(action) => match action {
                    P2pChannelsSnarkJobCommitmentAction::Init(action) => {
                        shared::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pChannelsSnarkJobCommitmentAction::Ready(action) => {
                        shared::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    _ => {}
                },
                P2pChannelsAction::Rpc(action) => match action {
                    P2pChannelsRpcAction::Init(action) => {
                        shared::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pChannelsRpcAction::Ready(action) => {
                        shared::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pChannelsRpcAction::RequestSend(action) => {
                        shared::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}, rpc_id: {}, kind: {:?}", action.peer_id, action.id, action.request.kind()),
                            peer_id = action.peer_id.to_string(),
                            rpc_id = action.id.to_string(),
                            trace_request = serde_json::to_string(&action.request).ok()
                        );
                    }
                    P2pChannelsRpcAction::ResponseReceived(action) => {
                        shared::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}, rpc_id: {}", action.peer_id, action.id),
                            peer_id = action.peer_id.to_string(),
                            rpc_id = action.id.to_string(),
                            trace_response = serde_json::to_string(&action.response).ok()
                        );
                    }
                    _ => {}
                },
            },
            P2pAction::Peer(_) => {}
        },
        Action::ExternalSnarkWorker(a) => {
            use crate::external_snark_worker::ExternalSnarkWorkerAction;
            match a {
                ExternalSnarkWorkerAction::Start(_)
                | ExternalSnarkWorkerAction::Started(_)
                | ExternalSnarkWorkerAction::Kill(_)
                | ExternalSnarkWorkerAction::Killed(_)
                | ExternalSnarkWorkerAction::WorkCancelled(_)
                | ExternalSnarkWorkerAction::PruneWork(_) => {
                    shared::log::debug!(
                        meta.time();
                        kind = kind.to_string(),
                        trace_action = serde_json::to_string(&a).ok()
                    )
                }
                ExternalSnarkWorkerAction::SubmitWork(a) => {
                    shared::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        work_id = a.job_id.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::WorkResult(_) => {
                    shared::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::CancelWork(_) => {
                    shared::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::WorkError(a) => {
                    shared::log::warn!(
                        meta.time();
                        kind = kind.to_string(),
                        error = a.error.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::Error(a) => {
                    shared::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        error = a.error.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::StartTimeout(_) => {
                    shared::log::warn!(
                        meta.time();
                        kind = kind.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::WorkTimeout(_) => {
                    shared::log::warn!(
                        meta.time();
                        kind = kind.to_string(),
                    )
                }
            }
        }
        Action::Snark(a) => match a {
            SnarkAction::WorkVerify(a) => match a {
                SnarkWorkVerifyAction::Init(a) => {
                    shared::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("id: {}, batch size: {}", a.req_id, a.batch.len()),
                        peer_id = a.sender,
                        rpc_id = a.req_id.to_string(),
                        trace_batch = serde_json::to_string(&a.batch.iter().map(|v| v.job_id()).collect::<Vec<_>>()).ok()
                    );
                }
                SnarkWorkVerifyAction::Error(a) => {
                    let Some(req) = store.state().snark.work_verify.jobs.get(a.req_id) else { return };
                    shared::log::warn!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("id: {}, batch size: {}", a.req_id, req.batch().len()),
                        peer_id = req.sender(),
                        rpc_id = a.req_id.to_string(),
                        trace_batch = serde_json::to_string(&req.batch().iter().map(|v| v.job_id()).collect::<Vec<_>>()).ok()
                    );
                }
                SnarkWorkVerifyAction::Success(a) => {
                    let Some(req) = store.state().snark.work_verify.jobs.get(a.req_id) else { return };
                    shared::log::warn!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("id: {}, batch size: {}", a.req_id, req.batch().len()),
                        peer_id = req.sender(),
                        rpc_id = a.req_id.to_string(),
                        trace_batch = serde_json::to_string(&req.batch().iter().map(|v| v.job_id()).collect::<Vec<_>>()).ok()
                    );
                }
                _ => {}
            },
            _ => {}
        },
        _ => {}
    }
}
