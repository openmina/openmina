use crate::action::CheckTimeoutsAction;
use crate::external_snark_worker::{
    ExternalSnarkWorkerErrorAction, ExternalSnarkWorkerEvent, ExternalSnarkWorkerKilledAction,
    ExternalSnarkWorkerStartedAction, ExternalSnarkWorkerWorkErrorAction,
    ExternalSnarkWorkerWorkResultAction,
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
    P2pConnectionIncomingFinalizeSuccessAction,
};
use crate::p2p::connection::outgoing::{
    P2pConnectionOutgoingAnswerRecvErrorAction, P2pConnectionOutgoingAnswerRecvSuccessAction,
    P2pConnectionOutgoingFinalizeErrorAction, P2pConnectionOutgoingFinalizeSuccessAction,
    P2pConnectionOutgoingOfferSdpCreateErrorAction,
    P2pConnectionOutgoingOfferSdpCreateSuccessAction,
};
use crate::p2p::connection::{P2pConnectionErrorResponse, P2pConnectionResponse};
use crate::p2p::disconnection::P2pDisconnectionFinishAction;
use crate::p2p::P2pChannelEvent;
use crate::rpc::{
    RpcActionStatsGetAction, RpcGlobalStateGetAction, RpcP2pConnectionIncomingInitAction,
    RpcP2pConnectionOutgoingInitAction, RpcRequest, RpcSnarkPoolAvailableJobsGetAction,
    RpcSnarkPoolJobGetAction, RpcSnarkerJobCommitAction, RpcSnarkerJobSpecAction,
    RpcSnarkersWorkersGetAction, RpcSyncStatsGetAction,
};
use crate::snark::block_verify::{SnarkBlockVerifyErrorAction, SnarkBlockVerifySuccessAction};
use crate::snark::SnarkEvent;
use crate::{Service, Store};

use super::{
    Event, EventSourceAction, EventSourceActionWithMeta, EventSourceNewEventAction,
    P2pConnectionEvent, P2pEvent,
};

pub fn event_source_effects<S: Service>(store: &mut Store<S>, action: EventSourceActionWithMeta) {
    let (action, _) = action.split();
    match action {
        EventSourceAction::ProcessEvents(_) => {
            // process max 1024 events at a time.
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
        EventSourceAction::NewEvent(content) => match content.event {
            Event::P2p(e) => match e {
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
                            store.dispatch(P2pConnectionOutgoingFinalizeSuccessAction { peer_id });
                            store.dispatch(P2pConnectionIncomingFinalizeSuccessAction { peer_id });
                        }
                    },
                    P2pConnectionEvent::Closed(peer_id) => {
                        store.dispatch(P2pDisconnectionFinishAction { peer_id });
                    }
                },
                P2pEvent::Channel(e) => match e {
                    P2pChannelEvent::Opened(peer_id, chan_id, res) => match res {
                        Err(_err) => {
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
                        if res.is_err() {
                            store.dispatch(P2pDisconnectionFinishAction { peer_id });
                        }
                    }
                    P2pChannelEvent::Received(peer_id, res) => match res {
                        Err(_) => {
                            store.dispatch(P2pDisconnectionFinishAction { peer_id });
                        }
                        Ok(message) => {
                            store.dispatch(P2pChannelsMessageReceivedAction { peer_id, message });
                        }
                    },
                    P2pChannelEvent::Libp2pSnarkReceived(peer_id, snark) => {
                        store.dispatch(P2pChannelsSnarkLibp2pReceivedAction { peer_id, snark });
                    }
                    P2pChannelEvent::Closed(peer_id, _) => {
                        store.dispatch(P2pDisconnectionFinishAction { peer_id });
                    }
                },
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
            },
            Event::Rpc(rpc_id, e) => match e {
                RpcRequest::GetState => {
                    store.dispatch(RpcGlobalStateGetAction { rpc_id });
                }
                RpcRequest::ActionStatsGet(query) => {
                    store.dispatch(RpcActionStatsGetAction { rpc_id, query });
                }
                RpcRequest::SyncStatsGet(query) => {
                    store.dispatch(RpcSyncStatsGetAction { rpc_id, query });
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
                RpcRequest::SnarkPoolGet => {
                    store.dispatch(RpcSnarkPoolAvailableJobsGetAction { rpc_id });
                }
                RpcRequest::SnarkPoolJobGet { job_id } => {
                    store.dispatch(RpcSnarkPoolJobGetAction { rpc_id, job_id });
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
                ExternalSnarkWorkerEvent::Error(error) => {
                    store.dispatch(ExternalSnarkWorkerErrorAction { error });
                }
            },
        },
        EventSourceAction::WaitTimeout(_) => {
            store.dispatch(CheckTimeoutsAction {});
        }
        EventSourceAction::WaitForEvents(_) => {}
    }
}
