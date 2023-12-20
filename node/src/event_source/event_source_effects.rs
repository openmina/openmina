use crate::action::CheckTimeoutsAction;
use crate::external_snark_worker::{
    ExternalSnarkWorkerErrorAction, ExternalSnarkWorkerEvent, ExternalSnarkWorkerKilledAction,
    ExternalSnarkWorkerStartedAction, ExternalSnarkWorkerWorkCancelledAction,
    ExternalSnarkWorkerWorkErrorAction, ExternalSnarkWorkerWorkResultAction,
};
use crate::p2p::channels::best_tip::P2pChannelsBestTipReadyAction;
use crate::p2p::channels::rpc::P2pChannelsRpcReadyAction;
use crate::p2p::channels::snark::{
    P2pChannelsSnarkLibp2pReceivedAction, P2pChannelsSnarkReadyAction,
};
use crate::p2p::channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentReadyAction;
use crate::p2p::channels::{ChannelId, P2pChannelsMessageReceivedAction};
use crate::p2p::connection::incoming::{
    P2pConnectionIncomingAnswerSdpCreateErrorAction,
    P2pConnectionIncomingAnswerSdpCreateSuccessAction, P2pConnectionIncomingFinalizeErrorAction,
    P2pConnectionIncomingFinalizeSuccessAction, P2pConnectionIncomingLibp2pReceivedAction,
};
use crate::p2p::connection::outgoing::{
    P2pConnectionOutgoingAnswerRecvErrorAction, P2pConnectionOutgoingAnswerRecvSuccessAction,
    P2pConnectionOutgoingFinalizeErrorAction, P2pConnectionOutgoingFinalizeSuccessAction,
    P2pConnectionOutgoingOfferSdpCreateErrorAction,
    P2pConnectionOutgoingOfferSdpCreateSuccessAction,
};
use crate::p2p::connection::{P2pConnectionErrorResponse, P2pConnectionResponse};
use crate::p2p::disconnection::{
    P2pDisconnectionFinishAction, P2pDisconnectionInitAction, P2pDisconnectionReason,
};
use crate::p2p::discovery::{
    P2pDiscoveryKademliaAddRouteAction, P2pDiscoveryKademliaFailureAction,
    P2pDiscoveryKademliaSuccessAction,
};
use crate::p2p::P2pChannelEvent;
use crate::rpc::{
    RpcActionStatsGetAction, RpcGlobalStateGetAction, RpcHealthCheckAction,
    RpcP2pConnectionIncomingInitAction, RpcP2pConnectionOutgoingInitAction, RpcPeersGetAction,
    RpcReadinessCheckAction, RpcRequest, RpcScanStateSummaryGetAction,
    RpcSnarkPoolAvailableJobsGetAction, RpcSnarkPoolJobGetAction, RpcSnarkerConfigGetAction,
    RpcSnarkerJobCommitAction, RpcSnarkerJobSpecAction, RpcSnarkersWorkersGetAction,
    RpcSyncStatsGetAction,
};
use crate::snark::block_verify::{SnarkBlockVerifyErrorAction, SnarkBlockVerifySuccessAction};
use crate::snark::work_verify::{SnarkWorkVerifyErrorAction, SnarkWorkVerifySuccessAction};
use crate::snark::SnarkEvent;
use crate::{Service, Store};

use super::{
    Event, EventSourceAction, EventSourceActionWithMeta, EventSourceNewEventAction,
    P2pConnectionEvent, P2pEvent,
};

pub fn event_source_effects<S: Service>(store: &mut Store<S>, action: EventSourceActionWithMeta) {
    let (action, meta) = action.split();
    match action {
        EventSourceAction::ProcessEvents(_) => {
            // This action gets continously called until there are no more
            // events available.
            //
            // Retrieve and process max 1024 events at a time and dispatch
            // `CheckTimeoutsAction` in between `EventSourceProcessEventsAction`
            // calls so that we make sure, that action gets called even
            // if we are continously flooded with events.
            for _ in 0..1024 {
                match store.service.next_event() {
                    Some(event) => {
                        store.dispatch(EventSourceNewEventAction { event });
                    }
                    None => break,
                }
            }
            store.dispatch(CheckTimeoutsAction {});
        }
        // "Translate" event into the corresponding action and dispatch it.
        EventSourceAction::NewEvent(content) => match content.event {
            Event::P2p(e) => match e {
                #[cfg(all(not(target_arch = "wasm32"), not(feature = "p2p-libp2p")))]
                P2pEvent::MioEvent(e) => todo!("handle {e}"),
                P2pEvent::Connection(e) => match e {
                    P2pConnectionEvent::OfferSdpReady(peer_id, res) => match res {
                        Err(error) => {
                            store.dispatch(P2pConnectionOutgoingOfferSdpCreateErrorAction {
                                peer_id,
                                error,
                            });
                        }
                        Ok(sdp) => {
                            store.dispatch(P2pConnectionOutgoingOfferSdpCreateSuccessAction {
                                peer_id,
                                sdp,
                            });
                        }
                    },
                    P2pConnectionEvent::AnswerSdpReady(peer_id, res) => match res {
                        Err(error) => {
                            store.dispatch(P2pConnectionIncomingAnswerSdpCreateErrorAction {
                                peer_id,
                                error,
                            });
                        }
                        Ok(sdp) => {
                            store.dispatch(P2pConnectionIncomingAnswerSdpCreateSuccessAction {
                                peer_id,
                                sdp,
                            });
                        }
                    },
                    P2pConnectionEvent::AnswerReceived(peer_id, res) => match res {
                        P2pConnectionResponse::Accepted(answer) => {
                            store.dispatch(P2pConnectionOutgoingAnswerRecvSuccessAction {
                                peer_id,
                                answer,
                            });
                        }
                        P2pConnectionResponse::Rejected(reason) => {
                            store.dispatch(P2pConnectionOutgoingAnswerRecvErrorAction {
                                peer_id,
                                error: P2pConnectionErrorResponse::Rejected(reason),
                            });
                        }
                        P2pConnectionResponse::InternalError => {
                            store.dispatch(P2pConnectionOutgoingAnswerRecvErrorAction {
                                peer_id,
                                error: P2pConnectionErrorResponse::InternalError,
                            });
                        }
                    },
                    P2pConnectionEvent::Finalized(peer_id, res) => match res {
                        Err(error) => {
                            store.dispatch(P2pConnectionOutgoingFinalizeErrorAction {
                                peer_id,
                                error: error.clone(),
                            });
                            store.dispatch(P2pConnectionIncomingFinalizeErrorAction {
                                peer_id,
                                error,
                            });
                        }
                        Ok(_) => {
                            let _ = store
                                .dispatch(P2pConnectionOutgoingFinalizeSuccessAction { peer_id })
                                || store.dispatch(P2pConnectionIncomingFinalizeSuccessAction {
                                    peer_id,
                                })
                                || store.dispatch(P2pConnectionIncomingLibp2pReceivedAction {
                                    peer_id,
                                });
                        }
                    },
                    P2pConnectionEvent::Closed(peer_id) => {
                        store.dispatch(P2pDisconnectionFinishAction { peer_id });
                    }
                },
                P2pEvent::Channel(e) => match e {
                    P2pChannelEvent::Opened(peer_id, chan_id, res) => match res {
                        Err(err) => {
                            openmina_core::log::warn!(meta.time(); kind = "P2pChannelEvent::Opened", peer_id = peer_id.to_string(), error = err);
                            // TODO(binier): dispatch error action.
                        }
                        Ok(_) => match chan_id {
                            ChannelId::BestTipPropagation => {
                                // TODO(binier): maybe dispatch success and then ready.
                                store.dispatch(P2pChannelsBestTipReadyAction { peer_id });
                            }
                            ChannelId::SnarkPropagation => {
                                // TODO(binier): maybe dispatch success and then ready.
                                store.dispatch(P2pChannelsSnarkReadyAction { peer_id });
                            }
                            ChannelId::SnarkJobCommitmentPropagation => {
                                // TODO(binier): maybe dispatch success and then ready.
                                store
                                    .dispatch(P2pChannelsSnarkJobCommitmentReadyAction { peer_id });
                            }
                            ChannelId::Rpc => {
                                // TODO(binier): maybe dispatch success and then ready.
                                store.dispatch(P2pChannelsRpcReadyAction { peer_id });
                            }
                        },
                    },
                    P2pChannelEvent::Sent(peer_id, _, _, res) => {
                        if let Err(err) = res {
                            let reason = P2pDisconnectionReason::P2pChannelSendFailed(err);
                            store.dispatch(P2pDisconnectionInitAction { peer_id, reason });
                        }
                    }
                    P2pChannelEvent::Received(peer_id, res) => match res {
                        Err(err) => {
                            let reason = P2pDisconnectionReason::P2pChannelReceiveFailed(err);
                            store.dispatch(P2pDisconnectionInitAction { peer_id, reason });
                        }
                        Ok(message) => {
                            store.dispatch(P2pChannelsMessageReceivedAction { peer_id, message });
                        }
                    },
                    P2pChannelEvent::Libp2pSnarkReceived(peer_id, snark, nonce) => {
                        store.dispatch(P2pChannelsSnarkLibp2pReceivedAction {
                            peer_id,
                            snark,
                            nonce,
                        });
                    }
                    P2pChannelEvent::Closed(peer_id, chan_id) => {
                        let reason = P2pDisconnectionReason::P2pChannelClosed(chan_id);
                        store.dispatch(P2pDisconnectionInitAction { peer_id, reason });
                    }
                },
                #[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
                P2pEvent::Libp2pIdentify(..) => {}
                P2pEvent::Discovery(p2p::P2pDiscoveryEvent::Ready) => {}
                P2pEvent::Discovery(p2p::P2pDiscoveryEvent::DidFindPeers(peers)) => {
                    store.dispatch(P2pDiscoveryKademliaSuccessAction { peers });
                }
                P2pEvent::Discovery(p2p::P2pDiscoveryEvent::DidFindPeersError(description)) => {
                    store.dispatch(P2pDiscoveryKademliaFailureAction { description });
                }
                P2pEvent::Discovery(p2p::P2pDiscoveryEvent::AddRoute(peer_id, addresses)) => {
                    store.dispatch(P2pDiscoveryKademliaAddRouteAction { peer_id, addresses });
                }
            },
            Event::Snark(event) => match event {
                SnarkEvent::BlockVerify(req_id, result) => match result {
                    Err(error) => {
                        store.dispatch(SnarkBlockVerifyErrorAction { req_id, error });
                    }
                    Ok(()) => {
                        store.dispatch(SnarkBlockVerifySuccessAction { req_id });
                    }
                },
                SnarkEvent::WorkVerify(req_id, result) => match result {
                    Err(error) => {
                        store.dispatch(SnarkWorkVerifyErrorAction { req_id, error });
                    }
                    Ok(()) => {
                        store.dispatch(SnarkWorkVerifySuccessAction { req_id });
                    }
                },
            },
            Event::Rpc(rpc_id, e) => match e {
                RpcRequest::StateGet => {
                    store.dispatch(RpcGlobalStateGetAction { rpc_id });
                }
                RpcRequest::ActionStatsGet(query) => {
                    store.dispatch(RpcActionStatsGetAction { rpc_id, query });
                }
                RpcRequest::SyncStatsGet(query) => {
                    store.dispatch(RpcSyncStatsGetAction { rpc_id, query });
                }
                RpcRequest::PeersGet => {
                    store.dispatch(RpcPeersGetAction { rpc_id });
                }
                RpcRequest::P2pConnectionOutgoing(opts) => {
                    store.dispatch(RpcP2pConnectionOutgoingInitAction { rpc_id, opts });
                }
                RpcRequest::P2pConnectionIncoming(opts) => {
                    store.dispatch(RpcP2pConnectionIncomingInitAction {
                        rpc_id,
                        opts: opts.clone(),
                    });
                }
                RpcRequest::ScanStateSummaryGet(query) => {
                    store.dispatch(RpcScanStateSummaryGetAction { rpc_id, query });
                }
                RpcRequest::SnarkPoolGet => {
                    store.dispatch(RpcSnarkPoolAvailableJobsGetAction { rpc_id });
                }
                RpcRequest::SnarkPoolJobGet { job_id } => {
                    store.dispatch(RpcSnarkPoolJobGetAction { rpc_id, job_id });
                }
                RpcRequest::SnarkerConfig => {
                    store.dispatch(RpcSnarkerConfigGetAction { rpc_id });
                }
                RpcRequest::SnarkerJobCommit { job_id } => {
                    store.dispatch(RpcSnarkerJobCommitAction { rpc_id, job_id });
                }
                RpcRequest::SnarkerJobSpec { job_id } => {
                    store.dispatch(RpcSnarkerJobSpecAction { rpc_id, job_id });
                }
                RpcRequest::SnarkerWorkers => {
                    store.dispatch(RpcSnarkersWorkersGetAction { rpc_id });
                }
                RpcRequest::HealthCheck => {
                    store.dispatch(RpcHealthCheckAction { rpc_id });
                }
                RpcRequest::ReadinessCheck => {
                    store.dispatch(RpcReadinessCheckAction { rpc_id });
                }
            },
            Event::ExternalSnarkWorker(e) => match e {
                ExternalSnarkWorkerEvent::Started => {
                    store.dispatch(ExternalSnarkWorkerStartedAction {});
                }
                ExternalSnarkWorkerEvent::Killed => {
                    store.dispatch(ExternalSnarkWorkerKilledAction {});
                }
                ExternalSnarkWorkerEvent::WorkResult(result) => {
                    store.dispatch(ExternalSnarkWorkerWorkResultAction { result });
                }
                ExternalSnarkWorkerEvent::WorkError(error) => {
                    store.dispatch(ExternalSnarkWorkerWorkErrorAction { error });
                }
                ExternalSnarkWorkerEvent::WorkCancelled => {
                    store.dispatch(ExternalSnarkWorkerWorkCancelledAction {});
                }
                ExternalSnarkWorkerEvent::Error(error) => {
                    store.dispatch(ExternalSnarkWorkerErrorAction {
                        error,
                        permanent: false,
                    });
                }
            },
        },
        EventSourceAction::WaitTimeout(_) => {
            store.dispatch(CheckTimeoutsAction {});
        }
        EventSourceAction::WaitForEvents(_) => {}
    }
}
