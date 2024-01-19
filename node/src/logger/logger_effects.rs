use p2p::connection::P2pConnectionAction;
use p2p::connection::libp2p::P2pConnectionLibP2pAction;
use p2p::connection::libp2p::incoming::P2pConnectionLibP2pIncomingAction;
use p2p::connection::libp2p::outgoing::P2pConnectionLibP2pOutgoingAction;
use p2p::connection::webrtc::P2pConnectionWebRTCAction;
use p2p::connection::webrtc::incoming::P2pConnectionWebRTCIncomingAction;
use p2p::connection::webrtc::outgoing::P2pConnectionWebRTCOutgoingAction;

use crate::p2p::channels::best_tip::P2pChannelsBestTipAction;
use crate::p2p::channels::rpc::P2pChannelsRpcAction;
use crate::p2p::channels::snark::P2pChannelsSnarkAction;
use crate::p2p::channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentAction;
use crate::p2p::channels::P2pChannelsAction;
use crate::p2p::disconnection::P2pDisconnectionAction;
use crate::p2p::discovery::P2pDiscoveryAction;
use crate::p2p::P2pAction;
use crate::snark::work_verify::SnarkWorkVerifyAction;
use crate::snark::SnarkAction;
use crate::transition_frontier::sync::TransitionFrontierSyncAction;
use crate::transition_frontier::TransitionFrontierAction;
use crate::{Action, ActionWithMetaRef, Service, Store};

pub fn logger_effects<S: Service>(store: &Store<S>, action: ActionWithMetaRef<'_>) {
    let (action, meta) = action.split();
    let kind = action.kind();

    match action {
        Action::P2p(action) => match action {
            P2pAction::Listen(action) => match action {
                p2p::listen::P2pListenAction::New(action) => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("addr: {}", action.addr),
                        addr = action.addr.to_string(),
                        listener_id = action.listener_id.to_string(),
                    );
                }
                p2p::listen::P2pListenAction::Expired(action) => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("addr: {}", action.addr),
                        addr = action.addr.to_string(),
                        listener_id = action.listener_id.to_string(),
                    );
                }
                p2p::listen::P2pListenAction::Error(action) => {
                    openmina_core::log::warn!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("id: {}, error: {}", action.listener_id, action.error),
                        error = action.error,
                        listener_id = action.listener_id.to_string(),
                    );
                }
                p2p::listen::P2pListenAction::Closed(action) => {
                    if let Some(error) = &action.error {
                        openmina_core::log::warn!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("id: {}, error: {error}", action.listener_id),
                            error = error,
                            listener_id = action.listener_id.to_string(),
                        );
                    } else {
                        openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("id: {},", action.listener_id),
                            listener_id = action.listener_id.to_string(),
                        );
                    }
                }
            },
            P2pAction::Connection(action) => match action {
                P2pConnectionAction::WebRTC(action) => match action {
                    P2pConnectionWebRTCAction::Outgoing(action) => match action {
                        P2pConnectionWebRTCOutgoingAction::Init(action) => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                            );
                        }
                        P2pConnectionWebRTCOutgoingAction::OfferSdpCreatePending(_) => {}
                        P2pConnectionWebRTCOutgoingAction::OfferSdpCreateError(action) => {
                            openmina_core::log::warn!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                                error = action.error.clone(),
                            );
                        }
                        P2pConnectionWebRTCOutgoingAction::OfferSdpCreateSuccess(action) => {
                            openmina_core::log::debug!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                                sdp = action.sdp.clone(),
                            );
                        }
                        P2pConnectionWebRTCOutgoingAction::OfferReady(action) => {
                            openmina_core::log::debug!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                                offer = serde_json::to_string(&action.offer).ok()
                            );
                        }
                        P2pConnectionWebRTCOutgoingAction::OfferSendSuccess(action) => {
                            openmina_core::log::debug!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                            );
                        }
                        P2pConnectionWebRTCOutgoingAction::AnswerRecvPending(_) => {}
                        P2pConnectionWebRTCOutgoingAction::AnswerRecvError(action) => {
                            openmina_core::log::warn!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                                error = format!("{:?}", action.error),
                            );
                        }
                        P2pConnectionWebRTCOutgoingAction::AnswerRecvSuccess(action) => {
                            openmina_core::log::debug!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                                trace_answer = serde_json::to_string(&action.answer).ok()
                            );
                        }
                        P2pConnectionWebRTCOutgoingAction::FinalizePending(_) => {}
                        P2pConnectionWebRTCOutgoingAction::FinalizeError(action) => {
                            openmina_core::log::warn!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                                error = action.error.clone(),
                            );
                        }
                        P2pConnectionWebRTCOutgoingAction::FinalizeSuccess(action) => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string()
                            );
                        }
                        P2pConnectionWebRTCOutgoingAction::Timeout(action) => {
                            openmina_core::log::warn!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string()
                            );
                        }
                        P2pConnectionWebRTCOutgoingAction::Error(action) => {
                            openmina_core::log::warn!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                                error = format!("{:?}", action.error),
                            );
                        }
                        P2pConnectionWebRTCOutgoingAction::Success(action) => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string()
                            );
                        }
                    }
                    P2pConnectionWebRTCAction::Incoming(action) => match action {
                        P2pConnectionWebRTCIncomingAction::Init(action) => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.opts.peer_id),
                                peer_id = action.opts.peer_id.to_string(),
                                trace_signaling = format!("{:?}", action.opts.signaling),
                            );
                        }
                        P2pConnectionWebRTCIncomingAction::AnswerSdpCreatePending(_) => {}
                        P2pConnectionWebRTCIncomingAction::AnswerSdpCreateError(action) => {
                            openmina_core::log::warn!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                                error = format!("{:?}", action.error),
                            );
                        }
                        P2pConnectionWebRTCIncomingAction::AnswerSdpCreateSuccess(action) => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                                trace_sdp = action.sdp.clone(),
                            );
                        }
                        P2pConnectionWebRTCIncomingAction::AnswerReady(action) => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                                trace_answer = serde_json::to_string(&action.answer).ok()
                            );
                        }
                        P2pConnectionWebRTCIncomingAction::AnswerSendSuccess(action) => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string()
                            );
                        }
                        P2pConnectionWebRTCIncomingAction::FinalizePending(_) => {}
                        P2pConnectionWebRTCIncomingAction::FinalizeError(action) => {
                            openmina_core::log::warn!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                                error = format!("{:?}", action.error),
                            );
                        }
                        P2pConnectionWebRTCIncomingAction::FinalizeSuccess(action) => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string()
                            );
                        }
                        P2pConnectionWebRTCIncomingAction::Timeout(action) => {
                            openmina_core::log::warn!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string()
                            );
                        }
                        P2pConnectionWebRTCIncomingAction::Error(action) => {
                            openmina_core::log::warn!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                                error = format!("{:?}", action.error),
                            );
                        }
                        P2pConnectionWebRTCIncomingAction::Success(action) => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string()
                            );
                        }
                        P2pConnectionWebRTCIncomingAction::Libp2pReceived(action) => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                            );
                        }
                    }
                }
                P2pConnectionAction::LibP2p(action) => match action {
                    P2pConnectionLibP2pAction::Outgoing(action) => match action {
                        P2pConnectionLibP2pOutgoingAction::Init(action) => {
                            openmina_core::log::debug!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                            );
                        }
                        P2pConnectionLibP2pOutgoingAction::FinalizePending(action) => {
                            openmina_core::log::debug!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                            );
                        }
                        P2pConnectionLibP2pOutgoingAction::FinalizeSuccess(action) => {
                            openmina_core::log::debug!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                            );
                        }
                        P2pConnectionLibP2pOutgoingAction::FinalizeError(action) => {
                            openmina_core::log::debug!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                            );
                        }
                        P2pConnectionLibP2pOutgoingAction::FinalizeTimeout(action) => {
                            openmina_core::log::debug!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                            );
                        }
                        P2pConnectionLibP2pOutgoingAction::Success(action) => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                            );
                        }
                        P2pConnectionLibP2pOutgoingAction::Error(action) => {
                            openmina_core::log::warn!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}, error: {}", action.peer_id, action.error.to_string()),
                                peer_id = action.peer_id.to_string(),
                                error = action.error.to_string(),
                            );
                        }
                    },
                    P2pConnectionLibP2pAction::Incoming(action) => match action {
                        P2pConnectionLibP2pIncomingAction::Success(action) => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("peer_id: {}", action.peer_id),
                                peer_id = action.peer_id.to_string(),
                            );
                        }
                    },
                }
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
                P2pDiscoveryAction::KademliaBootstrap(..) => {
                    openmina_core::log::debug!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("bootstrap kademlia"),
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
                        summary = format!("add route {} {:?}", action.peer_id, action.addresses),
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
        Action::TransitionFrontier(a) => match a {
            TransitionFrontierAction::Sync(action) => match action {
                TransitionFrontierSyncAction::Init(action) => openmina_core::log::info!(
                    meta.time();
                    kind = kind.to_string(),
                    summary = "Transition frontier sync init".to_string(),
                    block_hash = action.best_tip.hash.to_string(),
                    root_block_hash = action.root_block.hash.to_string(),
                ),
                TransitionFrontierSyncAction::BestTipUpdate(action) => openmina_core::log::info!(
                    meta.time();
                    kind = kind.to_string(),
                    summary = "New best tip received".to_string(),
                    block_hash = action.best_tip.hash.to_string(),
                    root_block_hash = action.root_block.hash.to_string(),
                ),
                TransitionFrontierSyncAction::LedgerStakingPending(_) => openmina_core::log::info!(
                    meta.time();
                    kind = kind.to_string(),
                    summary = "Staking ledger sync pending".to_string(),
                ),
                TransitionFrontierSyncAction::LedgerStakingSuccess(_) => openmina_core::log::info!(
                    meta.time();
                    kind = kind.to_string(),
                    summary = "Staking ledger sync success".to_string(),
                ),
                TransitionFrontierSyncAction::LedgerNextEpochPending(_) => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = "Next epoch ledger sync pending".to_string(),
                    )
                }
                TransitionFrontierSyncAction::LedgerNextEpochSuccess(_) => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = "Next epoch ledger sync pending".to_string(),
                    )
                }
                TransitionFrontierSyncAction::LedgerRootPending(_) => openmina_core::log::info!(
                    meta.time();
                    kind = kind.to_string(),
                    summary = "Transition frontier root ledger sync pending".to_string(),
                ),
                TransitionFrontierSyncAction::LedgerRootSuccess(_) => openmina_core::log::info!(
                    meta.time();
                    kind = kind.to_string(),
                    summary = "Transition frontier root ledger sync success".to_string(),
                ),
                _other => openmina_core::log::debug!(
                    meta.time();
                    kind = kind.to_string(),
                ),
            },
            TransitionFrontierAction::Synced(_) => openmina_core::log::info!(
                meta.time();
                kind = kind.to_string(),
                summary = "Transition frontier synced".to_string(),
            ),
        },
        _ => {}
    }
}
