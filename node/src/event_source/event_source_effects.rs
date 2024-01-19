use p2p::connection::webrtc::outgoing::P2pConnectionWebRTCOutgoingOfferSdpCreateErrorAction;
use p2p::listen::{
    P2pListenClosedAction, P2pListenErrorAction, P2pListenExpiredAction, P2pListenNewAction,
};
use p2p::P2pLibP2pEvent;

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
use crate::p2p::connection::libp2p::incoming::*;
use crate::p2p::connection::libp2p::outgoing::*;
use crate::p2p::connection::webrtc::incoming::*;
use crate::p2p::connection::webrtc::outgoing::*;
use crate::p2p::connection::webrtc::{
    P2pConnectionWebRTCErrorResponse, P2pConnectionWebRTCResponse,
};
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
    Event, EventSourceAction, EventSourceActionWithMeta, EventSourceNewEventAction, P2pEvent,
    P2pWebRTCEvent,
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
                P2pEvent::LibP2p(e) => match e {
                    P2pLibP2pEvent::IncomingConnection { connection_id: _ } => {
                        // TODO(akoptelov): track incoming connection initiation
                    }
                    P2pLibP2pEvent::Dialing { connection_id: _ } => {
                        // TODO(akoptelov): track outgoing connection initiated from libp2p
                    }
                    P2pLibP2pEvent::IncomingConnectionError {
                        connection_id: _,
                        error: _,
                    } => {
                        // TODO(akoptelov): track incoming connection error
                    }
                    P2pLibP2pEvent::OutgoingConnectionError {
                        connection_id: _,
                        peer_id,
                        error,
                    } => {
                        store.dispatch(P2pConnectionLibP2pOutgoingFinalizeErrorAction {
                            peer_id: peer_id.into(),
                            error: error.to_string(),
                        });
                    }
                    P2pLibP2pEvent::ConnectionEstablished {
                        peer_id,
                        connection_id,
                    } => {
                        let _ = store.dispatch(P2pConnectionLibP2pOutgoingFinalizeSuccessAction {
                            peer_id: peer_id.into(),
                        }) || store.dispatch(P2pConnectionLibP2pIncomingSuccessAction {
                            peer_id: peer_id.into(),
                            connection_id,
                        });
                    }
                    P2pLibP2pEvent::ConnectionClosed { peer_id, cause: _ } => {
                        store.dispatch(P2pDisconnectionFinishAction {
                            peer_id: peer_id.into(),
                        });
                    },
                    P2pLibP2pEvent::NewListenAddr { listener_id, addr } => {
                        store.dispatch(P2pListenNewAction { listener_id, addr });
                    }
                    P2pLibP2pEvent::ExpiredListenAddr { listener_id, addr } => {
                        store.dispatch(P2pListenExpiredAction { listener_id, addr });
                    }
                    P2pLibP2pEvent::ListenerError { listener_id, error } => {
                        store.dispatch(P2pListenErrorAction { listener_id, error });
                    }
                    P2pLibP2pEvent::ListenerClosed { listener_id, error } => {
                        store.dispatch(P2pListenClosedAction { listener_id, error });
                    }
                },
                P2pEvent::WebRTC(e) => match e {
                    P2pWebRTCEvent::OfferSdpReady(peer_id, res) => match res {
                        Err(error) => {
                            store.dispatch(P2pConnectionWebRTCOutgoingOfferSdpCreateErrorAction {
                                peer_id,
                                error,
                            });
                        }
                        Ok(sdp) => {
                            store.dispatch(
                                P2pConnectionWebRTCOutgoingOfferSdpCreateSuccessAction {
                                    peer_id,
                                    sdp,
                                },
                            );
                        }
                    },
                    P2pWebRTCEvent::AnswerSdpReady(peer_id, res) => match res {
                        Err(error) => {
                            store.dispatch(P2pConnectionWebRTCIncomingAnswerSdpCreateErrorAction {
                                peer_id,
                                error,
                            });
                        }
                        Ok(sdp) => {
                            store.dispatch(
                                P2pConnectionWebRTCIncomingAnswerSdpCreateSuccessAction {
                                    peer_id,
                                    sdp,
                                },
                            );
                        }
                    },
                    P2pWebRTCEvent::AnswerReceived(peer_id, res) => match res {
                        P2pConnectionWebRTCResponse::Accepted(answer) => {
                            store.dispatch(P2pConnectionWebRTCOutgoingAnswerRecvSuccessAction {
                                peer_id,
                                answer,
                            });
                        }
                        P2pConnectionWebRTCResponse::Rejected(reason) => {
                            store.dispatch(P2pConnectionWebRTCOutgoingAnswerRecvErrorAction {
                                peer_id,
                                error: P2pConnectionWebRTCErrorResponse::Rejected(reason),
                            });
                        }
                        P2pConnectionWebRTCResponse::InternalError => {
                            store.dispatch(P2pConnectionWebRTCOutgoingAnswerRecvErrorAction {
                                peer_id,
                                error: P2pConnectionWebRTCErrorResponse::InternalError,
                            });
                        }
                    },
                    P2pWebRTCEvent::Finalized(peer_id, res) => match res {
                        Err(error) => {
                            store.dispatch(P2pConnectionWebRTCOutgoingFinalizeErrorAction {
                                peer_id,
                                error: error.clone(),
                            });
                            store.dispatch(P2pConnectionWebRTCIncomingFinalizeErrorAction {
                                peer_id,
                                error,
                            });
                        }
                        Ok(_) => {
                            let _ =
                                store.dispatch(P2pConnectionWebRTCOutgoingFinalizeSuccessAction {
                                    peer_id,
                                }) || store.dispatch(
                                    P2pConnectionWebRTCIncomingFinalizeSuccessAction { peer_id },
                                ) || store.dispatch(
                                    P2pConnectionWebRTCIncomingLibp2pReceivedAction { peer_id },
                                );
                        }
                    },
                    P2pWebRTCEvent::Closed(peer_id) => {
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
                #[cfg(not(target_arch = "wasm32"))]
                P2pEvent::Libp2pIdentify(..) => {}
                P2pEvent::Discovery(p2p::P2pDiscoveryEvent::Ready) => {}
                P2pEvent::Discovery(p2p::P2pDiscoveryEvent::DidFindPeers(peers)) => {
                    store.dispatch(P2pDiscoveryKademliaSuccessAction { peers });
                }
                P2pEvent::Discovery(p2p::P2pDiscoveryEvent::DidFindPeersError(description)) => {
                    store.dispatch(P2pDiscoveryKademliaFailureAction { description });
                }
                P2pEvent::Discovery(p2p::P2pDiscoveryEvent::AddRoute(peer_id, addresses)) => {
                    store.dispatch(P2pDiscoveryKademliaAddRouteAction {
                        peer_id,
                        addresses: addresses.into(),
                    });
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
                RpcRequest::P2pConnectionOutgoing(peer_id, addrs) => {
                    store.dispatch(RpcP2pConnectionOutgoingInitAction {
                        rpc_id,
                        peer_id,
                        addrs,
                    });
                }
                RpcRequest::P2pConnectionWebRTCIncoming(opts) => {
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
