use p2p::channels::snark::P2pChannelsSnarkAction;
use snark::user_command_verify::{SnarkUserCommandVerifyAction, SnarkUserCommandVerifyError};

use crate::action::CheckTimeoutsAction;
use crate::block_producer::vrf_evaluator::BlockProducerVrfEvaluatorAction;
use crate::block_producer::{BlockProducerEvent, BlockProducerVrfEvaluatorEvent};
use crate::external_snark_worker::ExternalSnarkWorkerEvent;
use crate::ledger::read::LedgerReadAction;
use crate::ledger::write::LedgerWriteAction;
use crate::p2p::channels::best_tip::P2pChannelsBestTipAction;
use crate::p2p::channels::rpc::P2pChannelsRpcAction;
use crate::p2p::channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentAction;
use crate::p2p::channels::{ChannelId, P2pChannelsMessageReceivedAction};
use crate::p2p::connection::incoming::P2pConnectionIncomingAction;
use crate::p2p::connection::outgoing::P2pConnectionOutgoingAction;
use crate::p2p::connection::{P2pConnectionErrorResponse, P2pConnectionResponse};
use crate::p2p::disconnection::{P2pDisconnectionAction, P2pDisconnectionReason};
use crate::p2p::P2pChannelEvent;
#[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
use crate::p2p::{MioEvent, P2pNetworkSchedulerAction};
use crate::rpc::{RpcAction, RpcRequest};
use crate::snark::block_verify::SnarkBlockVerifyAction;
use crate::snark::work_verify::SnarkWorkVerifyAction;
use crate::snark::SnarkEvent;
use crate::transition_frontier::genesis::TransitionFrontierGenesisAction;
use crate::{BlockProducerAction, ExternalSnarkWorkerAction, Service, Store};

use super::{
    Event, EventSourceAction, EventSourceActionWithMeta, LedgerEvent, P2pConnectionEvent, P2pEvent,
};

pub fn event_source_effects<S: Service>(store: &mut Store<S>, action: EventSourceActionWithMeta) {
    let (action, meta) = action.split();
    match action {
        EventSourceAction::ProcessEvents => {
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
                        store.dispatch(EventSourceAction::NewEvent { event });
                    }
                    None => break,
                }
            }
            store.dispatch(CheckTimeoutsAction {});
        }
        // "Translate" event into the corresponding action and dispatch it.
        EventSourceAction::NewEvent { event } => match event {
            Event::P2p(e) => match e {
                #[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
                P2pEvent::MioEvent(e) => match e {
                    MioEvent::InterfaceDetected(ip) => {
                        store.dispatch(P2pNetworkSchedulerAction::InterfaceDetected { ip });
                    }
                    MioEvent::InterfaceExpired(ip) => {
                        store.dispatch(P2pNetworkSchedulerAction::InterfaceExpired { ip });
                    }
                    MioEvent::ListenerReady { listener } => {
                        store.dispatch(P2pNetworkSchedulerAction::ListenerReady { listener });
                    }
                    MioEvent::ListenerError { listener, error } => {
                        store
                            .dispatch(P2pNetworkSchedulerAction::ListenerError { listener, error });
                    }
                    MioEvent::IncomingConnectionIsReady { listener } => {
                        store.dispatch(P2pNetworkSchedulerAction::IncomingConnectionIsReady {
                            listener,
                        });
                    }
                    MioEvent::IncomingConnectionDidAccept(addr, result) => {
                        store.dispatch(P2pNetworkSchedulerAction::IncomingDidAccept {
                            addr,
                            result,
                        });
                    }
                    MioEvent::OutgoingConnectionDidConnect(addr, result) => {
                        store.dispatch(P2pNetworkSchedulerAction::OutgoingDidConnect {
                            addr,
                            result,
                        });
                    }
                    MioEvent::IncomingDataIsReady(addr) => {
                        store.dispatch(P2pNetworkSchedulerAction::IncomingDataIsReady { addr });
                    }
                    MioEvent::IncomingDataDidReceive(addr, result) => {
                        store.dispatch(P2pNetworkSchedulerAction::IncomingDataDidReceive {
                            addr,
                            result: result.map(From::from),
                        });
                    }
                    MioEvent::OutgoingDataDidSend(_, _result) => {}
                    MioEvent::ConnectionDidClose(addr, result) => {
                        if let Err(e) = result {
                            store.dispatch(P2pNetworkSchedulerAction::Error {
                                addr,
                                error: p2p::P2pNetworkConnectionError::MioError(e),
                            });
                        } else {
                            store.dispatch(P2pNetworkSchedulerAction::Error {
                                addr,
                                error: p2p::P2pNetworkConnectionError::RemoteClosed,
                            });
                        }
                    }
                },
                P2pEvent::Connection(e) => match e {
                    P2pConnectionEvent::OfferSdpReady(peer_id, res) => match res {
                        Err(error) => {
                            store.dispatch(P2pConnectionOutgoingAction::OfferSdpCreateError {
                                peer_id,
                                error,
                            });
                        }
                        Ok(sdp) => {
                            store.dispatch(P2pConnectionOutgoingAction::OfferSdpCreateSuccess {
                                peer_id,
                                sdp,
                            });
                        }
                    },
                    P2pConnectionEvent::AnswerSdpReady(peer_id, res) => match res {
                        Err(error) => {
                            store.dispatch(P2pConnectionIncomingAction::AnswerSdpCreateError {
                                peer_id,
                                error,
                            });
                        }
                        Ok(sdp) => {
                            store.dispatch(P2pConnectionIncomingAction::AnswerSdpCreateSuccess {
                                peer_id,
                                sdp,
                            });
                        }
                    },
                    P2pConnectionEvent::AnswerReceived(peer_id, res) => match res {
                        P2pConnectionResponse::Accepted(answer) => {
                            store.dispatch(P2pConnectionOutgoingAction::AnswerRecvSuccess {
                                peer_id,
                                answer: *answer,
                            });
                        }
                        P2pConnectionResponse::Rejected(reason) => {
                            store.dispatch(P2pConnectionOutgoingAction::AnswerRecvError {
                                peer_id,
                                error: P2pConnectionErrorResponse::Rejected(reason),
                            });
                        }
                        P2pConnectionResponse::InternalError => {
                            store.dispatch(P2pConnectionOutgoingAction::AnswerRecvError {
                                peer_id,
                                error: P2pConnectionErrorResponse::InternalError,
                            });
                        }
                    },
                    P2pConnectionEvent::Finalized(peer_id, res) => match res {
                        Err(error) => {
                            store.dispatch(P2pConnectionOutgoingAction::FinalizeError {
                                peer_id,
                                error: error.clone(),
                            });
                            store.dispatch(P2pConnectionIncomingAction::FinalizeError {
                                peer_id,
                                error,
                            });
                        }
                        Ok(_) => {
                            let _ = store
                                .dispatch(P2pConnectionOutgoingAction::FinalizeSuccess { peer_id })
                                || store.dispatch(P2pConnectionIncomingAction::FinalizeSuccess {
                                    peer_id,
                                })
                                || store.dispatch(P2pConnectionIncomingAction::Libp2pReceived {
                                    peer_id,
                                });
                        }
                    },
                    P2pConnectionEvent::Closed(peer_id) => {
                        store.dispatch(P2pDisconnectionAction::Finish { peer_id });
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
                                store.dispatch(P2pChannelsBestTipAction::Ready { peer_id });
                            }
                            ChannelId::SnarkPropagation => {
                                // TODO(binier): maybe dispatch success and then ready.
                                store.dispatch(P2pChannelsSnarkAction::Ready { peer_id });
                            }
                            ChannelId::SnarkJobCommitmentPropagation => {
                                // TODO(binier): maybe dispatch success and then ready.
                                store.dispatch(P2pChannelsSnarkJobCommitmentAction::Ready {
                                    peer_id,
                                });
                            }
                            ChannelId::Rpc => {
                                // TODO(binier): maybe dispatch success and then ready.
                                store.dispatch(P2pChannelsRpcAction::Ready { peer_id });
                            }
                        },
                    },
                    P2pChannelEvent::Sent(peer_id, _, _, res) => {
                        if let Err(err) = res {
                            let reason = P2pDisconnectionReason::P2pChannelSendFailed(err);
                            store.dispatch(P2pDisconnectionAction::Init { peer_id, reason });
                        }
                    }
                    P2pChannelEvent::Received(peer_id, res) => match res {
                        Err(err) => {
                            let reason = P2pDisconnectionReason::P2pChannelReceiveFailed(err);
                            store.dispatch(P2pDisconnectionAction::Init { peer_id, reason });
                        }
                        Ok(message) => {
                            store.dispatch(P2pChannelsMessageReceivedAction { peer_id, message });
                        }
                    },
                    P2pChannelEvent::Libp2pSnarkReceived(peer_id, snark, nonce) => {
                        store.dispatch(P2pChannelsSnarkAction::Libp2pReceived {
                            peer_id,
                            snark,
                            nonce,
                        });
                    }
                    P2pChannelEvent::Closed(peer_id, chan_id) => {
                        let reason = P2pDisconnectionReason::P2pChannelClosed(chan_id);
                        store.dispatch(P2pDisconnectionAction::Init { peer_id, reason });
                    }
                },
            },
            Event::Ledger(event) => match event {
                LedgerEvent::Write(response) => {
                    store.dispatch(LedgerWriteAction::Success { response });
                }
                LedgerEvent::Read(id, response) => {
                    store.dispatch(LedgerReadAction::Success { id, response });
                }
            },
            Event::Snark(event) => match event {
                SnarkEvent::BlockVerify(req_id, result) => match result {
                    Err(error) => {
                        store.dispatch(SnarkBlockVerifyAction::Error { req_id, error });
                    }
                    Ok(()) => {
                        store.dispatch(SnarkBlockVerifyAction::Success { req_id });
                    }
                },
                SnarkEvent::WorkVerify(req_id, result) => match result {
                    Err(error) => {
                        store.dispatch(SnarkWorkVerifyAction::Error { req_id, error });
                    }
                    Ok(()) => {
                        store.dispatch(SnarkWorkVerifyAction::Success { req_id });
                    }
                },
                SnarkEvent::UserCommandVerify(req_id, result) => {
                    if result.iter().any(|res| res.is_err()) {
                        store.dispatch(SnarkUserCommandVerifyAction::Error {
                            req_id,
                            error: SnarkUserCommandVerifyError::VerificationFailed,
                        });
                    } else {
                        store.dispatch(SnarkUserCommandVerifyAction::Success { req_id });
                    }
                }
            },
            Event::Rpc(rpc_id, e) => match e {
                RpcRequest::StateGet(filter) => {
                    store.dispatch(RpcAction::GlobalStateGet { rpc_id, filter });
                }
                RpcRequest::ActionStatsGet(query) => {
                    store.dispatch(RpcAction::ActionStatsGet { rpc_id, query });
                }
                RpcRequest::SyncStatsGet(query) => {
                    store.dispatch(RpcAction::SyncStatsGet { rpc_id, query });
                }
                RpcRequest::PeersGet => {
                    store.dispatch(RpcAction::PeersGet { rpc_id });
                }
                RpcRequest::MessageProgressGet => {
                    store.dispatch(RpcAction::MessageProgressGet { rpc_id });
                }
                RpcRequest::P2pConnectionOutgoing(opts) => {
                    store.dispatch(RpcAction::P2pConnectionOutgoingInit { rpc_id, opts });
                }
                RpcRequest::P2pConnectionIncoming(opts) => {
                    store.dispatch(RpcAction::P2pConnectionIncomingInit {
                        rpc_id,
                        opts: opts.clone(),
                    });
                }
                RpcRequest::ScanStateSummaryGet(query) => {
                    store.dispatch(RpcAction::ScanStateSummaryGetInit { rpc_id, query });
                }
                RpcRequest::SnarkPoolGet => {
                    store.dispatch(RpcAction::SnarkPoolAvailableJobsGet { rpc_id });
                }
                RpcRequest::SnarkPoolJobGet { job_id } => {
                    store.dispatch(RpcAction::SnarkPoolJobGet { rpc_id, job_id });
                }
                RpcRequest::SnarkerConfig => {
                    store.dispatch(RpcAction::SnarkerConfigGet { rpc_id });
                }
                RpcRequest::SnarkerJobCommit { job_id } => {
                    store.dispatch(RpcAction::SnarkerJobCommit { rpc_id, job_id });
                }
                RpcRequest::SnarkerJobSpec { job_id } => {
                    store.dispatch(RpcAction::SnarkerJobSpec { rpc_id, job_id });
                }
                RpcRequest::SnarkerWorkers => {
                    store.dispatch(RpcAction::SnarkerWorkersGet { rpc_id });
                }
                RpcRequest::HealthCheck => {
                    store.dispatch(RpcAction::HealthCheck { rpc_id });
                }
                RpcRequest::ReadinessCheck => {
                    store.dispatch(RpcAction::ReadinessCheck { rpc_id });
                }
                RpcRequest::DiscoveryRoutingTable => {
                    store.dispatch(RpcAction::DiscoveryRoutingTable { rpc_id });
                }
                RpcRequest::DiscoveryBoostrapStats => {
                    store.dispatch(RpcAction::DiscoveryBoostrapStats { rpc_id });
                }
            },
            Event::ExternalSnarkWorker(e) => match e {
                ExternalSnarkWorkerEvent::Started => {
                    store.dispatch(ExternalSnarkWorkerAction::Started);
                }
                ExternalSnarkWorkerEvent::Killed => {
                    store.dispatch(ExternalSnarkWorkerAction::Killed);
                }
                ExternalSnarkWorkerEvent::WorkResult(result) => {
                    store.dispatch(ExternalSnarkWorkerAction::WorkResult { result });
                }
                ExternalSnarkWorkerEvent::WorkError(error) => {
                    store.dispatch(ExternalSnarkWorkerAction::WorkError { error });
                }
                ExternalSnarkWorkerEvent::WorkCancelled => {
                    store.dispatch(ExternalSnarkWorkerAction::WorkCancelled);
                }
                ExternalSnarkWorkerEvent::Error(error) => {
                    store.dispatch(ExternalSnarkWorkerAction::Error {
                        error,
                        permanent: false,
                    });
                }
            },
            Event::BlockProducerEvent(e) => match e {
                BlockProducerEvent::VrfEvaluator(vrf_e) => match vrf_e {
                    BlockProducerVrfEvaluatorEvent::Evaluated(vrf_output_with_hash) => {
                        store.dispatch(
                            BlockProducerVrfEvaluatorAction::ProcessSlotEvaluationSuccess {
                                vrf_output: vrf_output_with_hash.evaluation_result,
                                staking_ledger_hash: vrf_output_with_hash.staking_ledger_hash,
                            },
                        );
                    }
                },
                BlockProducerEvent::BlockProve(block_hash, res) => match res {
                    Err(err) => todo!(
                        "error while trying to produce block proof for block {block_hash} - {err}"
                    ),
                    Ok(proof) => {
                        if store
                            .state()
                            .transition_frontier
                            .genesis
                            .prove_pending_block_hash()
                            .map_or(false, |hash| hash == block_hash)
                        {
                            store.dispatch(TransitionFrontierGenesisAction::ProveSuccess { proof });
                        } else {
                            store.dispatch(BlockProducerAction::BlockProveSuccess { proof });
                        }
                    }
                },
            },
            Event::GenesisLoad(res) => match res {
                Err(err) => todo!("error while trying to load genesis config/ledger. - {err}"),
                Ok(data) => {
                    store.dispatch(TransitionFrontierGenesisAction::LedgerLoadSuccess { data });
                }
            },
        },
        EventSourceAction::WaitTimeout => {
            store.dispatch(CheckTimeoutsAction {});
        }
        EventSourceAction::WaitForEvents => {}
    }
}
