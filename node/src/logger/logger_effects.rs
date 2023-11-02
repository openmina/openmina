use crate::p2p::channels::best_tip::P2pChannelsBestTipAction;
use crate::p2p::channels::rpc::P2pChannelsRpcAction;
use crate::p2p::channels::snark::P2pChannelsSnarkAction;
use crate::p2p::channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentAction;
use crate::p2p::channels::P2pChannelsAction;
use crate::p2p::connection::incoming::P2pConnectionIncomingAction;
use crate::p2p::connection::outgoing::P2pConnectionOutgoingAction;
use crate::p2p::connection::P2pConnectionAction;
use crate::p2p::disconnection::P2pDisconnectionAction;
use crate::p2p::discovery::P2pDiscoveryAction;
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
                        openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.opts.peer_id()),
                            peer_id = action.opts.peer_id().to_string(),
                            transport = action.opts.kind(),
                        );
                    }
                    P2pConnectionOutgoingAction::Reconnect(action) => {
                        openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.opts.peer_id()),
                            peer_id = action.opts.peer_id().to_string(),
                            transport = action.opts.kind(),
                        );
                    }
                    P2pConnectionOutgoingAction::OfferSdpCreatePending(_) => {}
                    P2pConnectionOutgoingAction::OfferSdpCreateError(action) => {
                        openmina_core::log::warn!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            error = action.error.clone(),
                        );
                    }
                    P2pConnectionOutgoingAction::OfferSdpCreateSuccess(action) => {
                        openmina_core::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            sdp = action.sdp.clone(),
                        );
                    }
                    P2pConnectionOutgoingAction::OfferReady(action) => {
                        openmina_core::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            offer = serde_json::to_string(&action.offer).ok()
                        );
                    }
                    P2pConnectionOutgoingAction::OfferSendSuccess(action) => {
                        openmina_core::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                        );
                    }
                    P2pConnectionOutgoingAction::AnswerRecvPending(_) => {}
                    P2pConnectionOutgoingAction::AnswerRecvError(action) => {
                        openmina_core::log::warn!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            error = format!("{:?}", action.error),
                        );
                    }
                    P2pConnectionOutgoingAction::AnswerRecvSuccess(action) => {
                        openmina_core::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            trace_answer = serde_json::to_string(&action.answer).ok()
                        );
                    }
                    P2pConnectionOutgoingAction::FinalizePending(_) => {}
                    P2pConnectionOutgoingAction::FinalizeError(action) => {
                        openmina_core::log::warn!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            error = action.error.clone(),
                        );
                    }
                    P2pConnectionOutgoingAction::FinalizeSuccess(action) => {
                        openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pConnectionOutgoingAction::Timeout(action) => {
                        openmina_core::log::warn!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pConnectionOutgoingAction::Error(action) => {
                        openmina_core::log::warn!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            error = format!("{:?}", action.error),
                        );
                    }
                    P2pConnectionOutgoingAction::Success(action) => {
                        openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                },
                P2pConnectionAction::Incoming(action) => match action {
                    P2pConnectionIncomingAction::Init(action) => {
                        openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.opts.peer_id),
                            peer_id = action.opts.peer_id.to_string(),
                            trace_signaling = format!("{:?}", action.opts.signaling),
                        );
                    }
                    P2pConnectionIncomingAction::AnswerSdpCreatePending(_) => {}
                    P2pConnectionIncomingAction::AnswerSdpCreateError(action) => {
                        openmina_core::log::warn!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            error = format!("{:?}", action.error),
                        );
                    }
                    P2pConnectionIncomingAction::AnswerSdpCreateSuccess(action) => {
                        openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            trace_sdp = action.sdp.clone(),
                        );
                    }
                    P2pConnectionIncomingAction::AnswerReady(action) => {
                        openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            trace_answer = serde_json::to_string(&action.answer).ok()
                        );
                    }
                    P2pConnectionIncomingAction::AnswerSendSuccess(action) => {
                        openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pConnectionIncomingAction::FinalizePending(_) => {}
                    P2pConnectionIncomingAction::FinalizeError(action) => {
                        openmina_core::log::warn!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            error = format!("{:?}", action.error),
                        );
                    }
                    P2pConnectionIncomingAction::FinalizeSuccess(action) => {
                        openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pConnectionIncomingAction::Timeout(action) => {
                        openmina_core::log::warn!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pConnectionIncomingAction::Error(action) => {
                        openmina_core::log::warn!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                            error = format!("{:?}", action.error),
                        );
                    }
                    P2pConnectionIncomingAction::Success(action) => {
                        openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pConnectionIncomingAction::Libp2pReceived(action) => {
                        openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string(),
                        );
                    }
                },
            },
            P2pAction::Disconnection(action) => match action {
                P2pDisconnectionAction::Init(action) => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("peer_id: {}", action.peer_id),
                        peer_id = action.peer_id.to_string(),
                        reason = format!("{:?}", action.reason)
                    );
                }
                P2pDisconnectionAction::Finish(action) => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("peer_id: {}", action.peer_id),
                        peer_id = action.peer_id.to_string()
                    );
                }
            },
            P2pAction::Discovery(action) => match action {
                P2pDiscoveryAction::Init(action) => {
                    openmina_core::log::debug!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("peer_id: {}", action.peer_id),
                        peer_id = action.peer_id.to_string()
                    );
                }
                P2pDiscoveryAction::Success(action) => {
                    openmina_core::log::debug!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("peer_id: {}", action.peer_id),
                        peer_id = action.peer_id.to_string()
                    );
                }
                P2pDiscoveryAction::KademliaInit(..) => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("find node"),
                    );
                }
                P2pDiscoveryAction::KademliaAddRoute(action) => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("add route {} {:?}", action.peer_id, action.addresses.first()),
                    );
                }
                P2pDiscoveryAction::KademliaSuccess(action) => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("peers: {:?}", action.peers),
                    );
                }
                P2pDiscoveryAction::KademliaFailure(action) => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("{:?}", action.description),
                    );
                }
            },
            P2pAction::Channels(action) => match action {
                P2pChannelsAction::MessageReceived(_) => {}
                P2pChannelsAction::BestTip(action) => match action {
                    P2pChannelsBestTipAction::Init(action) => {
                        openmina_core::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pChannelsBestTipAction::Ready(action) => {
                        openmina_core::log::debug!(
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
                        openmina_core::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pChannelsSnarkAction::Ready(action) => {
                        openmina_core::log::debug!(
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
                        openmina_core::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pChannelsSnarkJobCommitmentAction::Ready(action) => {
                        openmina_core::log::debug!(
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
                        openmina_core::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pChannelsRpcAction::Ready(action) => {
                        openmina_core::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", action.peer_id),
                            peer_id = action.peer_id.to_string()
                        );
                    }
                    P2pChannelsRpcAction::RequestSend(action) => {
                        openmina_core::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}, rpc_id: {}, kind: {:?}", action.peer_id, action.id, action.request.kind()),
                            peer_id = action.peer_id.to_string(),
                            rpc_id = action.id.to_string(),
                            trace_request = serde_json::to_string(&action.request).ok()
                        );
                    }
                    P2pChannelsRpcAction::ResponseReceived(action) => {
                        openmina_core::log::debug!(
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
                    openmina_core::log::debug!(
                        meta.time();
                        kind = kind.to_string(),
                        trace_action = serde_json::to_string(&a).ok()
                    )
                }
                ExternalSnarkWorkerAction::SubmitWork(a) => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        work_id = a.job_id.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::WorkResult(_) => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::CancelWork(_) => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::WorkError(a) => {
                    openmina_core::log::warn!(
                        meta.time();
                        kind = kind.to_string(),
                        error = a.error.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::Error(a) => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        error = a.error.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::StartTimeout(_) => {
                    openmina_core::log::warn!(
                        meta.time();
                        kind = kind.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::WorkTimeout(_) => {
                    openmina_core::log::warn!(
                        meta.time();
                        kind = kind.to_string(),
                    )
                }
            }
        }
        Action::Snark(a) => match a {
            SnarkAction::WorkVerify(a) => match a {
                SnarkWorkVerifyAction::Init(a) => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("id: {}, batch size: {}", a.req_id, a.batch.len()),
                        peer_id = a.sender,
                        rpc_id = a.req_id.to_string(),
                        trace_batch = serde_json::to_string(&a.batch.iter().map(|v| v.job_id()).collect::<Vec<_>>()).ok()
                    );
                }
                SnarkWorkVerifyAction::Error(a) => {
                    let Some(req) = store.state().snark.work_verify.jobs.get(a.req_id) else {
                        return;
                    };
                    openmina_core::log::warn!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("id: {}, batch size: {}", a.req_id, req.batch().len()),
                        peer_id = req.sender(),
                        rpc_id = a.req_id.to_string(),
                        trace_batch = serde_json::to_string(&req.batch().iter().map(|v| v.job_id()).collect::<Vec<_>>()).ok()
                    );
                }
                SnarkWorkVerifyAction::Success(a) => {
                    let Some(req) = store.state().snark.work_verify.jobs.get(a.req_id) else {
                        return;
                    };
                    openmina_core::log::info!(
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
