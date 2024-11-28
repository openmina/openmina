use std::net::{IpAddr, SocketAddr};

use openmina_core::requests::{RequestId, RpcIdType};
use p2p::channels::signaling::discovery::P2pChannelsSignalingDiscoveryAction;
use p2p::channels::signaling::exchange::P2pChannelsSignalingExchangeAction;
use p2p::channels::snark::P2pChannelsSnarkAction;
use p2p::channels::streaming_rpc::P2pChannelsStreamingRpcAction;
use p2p::channels::transaction::P2pChannelsTransactionAction;
use p2p::ConnectionAddr;
use redux::ActionMeta;
use snark::user_command_verify::{SnarkUserCommandVerifyAction, SnarkUserCommandVerifyError};

use crate::action::CheckTimeoutsAction;
use crate::block_producer::vrf_evaluator::BlockProducerVrfEvaluatorAction;
use crate::block_producer::{BlockProducerEvent, BlockProducerVrfEvaluatorEvent};
use crate::external_snark_worker_effectful::ExternalSnarkWorkerEvent;
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
#[cfg(feature = "p2p-libp2p")]
use crate::p2p::{MioEvent, P2pNetworkSchedulerAction};
use crate::rpc::{RpcAction, RpcRequest};
use crate::snark::block_verify::SnarkBlockVerifyAction;
use crate::snark::work_verify::SnarkWorkVerifyAction;
use crate::snark::SnarkEvent;
use crate::transition_frontier::genesis::{GenesisConfigLoaded, TransitionFrontierGenesisAction};
use crate::{BlockProducerAction, ExternalSnarkWorkerAction, Service, Store};

use super::{
    Event, EventSourceAction, EventSourceActionWithMeta, LedgerEvent, P2pConnectionEvent, P2pEvent,
};

#[inline(never)]
fn handle_p2p_event<S: Service>(store: &mut Store<S>, meta: ActionMeta, e: P2pEvent) {
    match e {
        #[cfg(not(feature = "p2p-libp2p"))]
        P2pEvent::MioEvent(_) => {}
        #[cfg(feature = "p2p-libp2p")]
        P2pEvent::MioEvent(e) => handle_mio_event(store, meta, e),
        P2pEvent::Connection(e) => handle_connection_event(store, e),
        P2pEvent::Channel(e) => handle_channel_event(store, meta, e),
    }
}

#[inline(never)]
fn handle_interface_detected<S: Service>(store: &mut Store<S>, ip: IpAddr) {
    store.dispatch(P2pNetworkSchedulerAction::InterfaceDetected { ip });
}

#[inline(never)]
fn handle_interface_expired<S: Service>(store: &mut Store<S>, ip: IpAddr) {
    store.dispatch(P2pNetworkSchedulerAction::InterfaceExpired { ip });
}

#[inline(never)]
fn handle_listener_ready<S: Service>(store: &mut Store<S>, listener: SocketAddr) {
    store.dispatch(P2pNetworkSchedulerAction::ListenerReady { listener });
}

#[inline(never)]
fn handle_listener_error<S: Service>(store: &mut Store<S>, listener: SocketAddr, error: String) {
    store.dispatch(P2pNetworkSchedulerAction::ListenerError { listener, error });
}

#[inline(never)]
fn handle_incoming_connection_ready<S: Service>(store: &mut Store<S>, listener: SocketAddr) {
    store.dispatch(P2pNetworkSchedulerAction::IncomingConnectionIsReady { listener });
}

#[inline(never)]
fn handle_incoming_connection_did_accept<S: Service>(
    store: &mut Store<S>,
    addr: Option<ConnectionAddr>,
    result: Result<(), String>,
) {
    store.dispatch(P2pNetworkSchedulerAction::IncomingDidAccept { addr, result });
}

#[inline(never)]
fn handle_outgoing_connection_did_connect<S: Service>(
    store: &mut Store<S>,
    addr: ConnectionAddr,
    result: Result<(), String>,
) {
    store.dispatch(P2pNetworkSchedulerAction::OutgoingDidConnect { addr, result });
}

#[inline(never)]
fn handle_incoming_data_is_ready<S: Service>(store: &mut Store<S>, addr: ConnectionAddr) {
    store.dispatch(P2pNetworkSchedulerAction::IncomingDataIsReady { addr });
}

#[inline(never)]
fn handle_incoming_data_did_receive<S: Service>(
    store: &mut Store<S>,
    addr: ConnectionAddr,
    result: Result<p2p::Data, String>,
) {
    store.dispatch(P2pNetworkSchedulerAction::IncomingDataDidReceive {
        addr,
        result: result.map(From::from),
    });
}

#[inline(never)]
fn handle_connection_did_close<S: Service>(
    store: &mut Store<S>,
    addr: ConnectionAddr,
    result: Result<(), String>,
) {
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

#[inline(never)]
fn handle_connection_did_close_on_demand<S: Service>(store: &mut Store<S>, addr: ConnectionAddr) {
    store.dispatch(P2pNetworkSchedulerAction::Prune { addr });
}

#[inline(never)]
fn handle_mio_event<S: Service>(store: &mut Store<S>, _meta: ActionMeta, e: MioEvent) {
    match e {
        MioEvent::InterfaceDetected(ip) => {
            handle_interface_detected(store, ip);
        }
        MioEvent::InterfaceExpired(ip) => {
            handle_interface_expired(store, ip);
        }
        MioEvent::ListenerReady { listener } => {
            handle_listener_ready(store, listener);
        }
        MioEvent::ListenerError { listener, error } => {
            handle_listener_error(store, listener, error);
        }
        MioEvent::IncomingConnectionIsReady { listener } => {
            handle_incoming_connection_ready(store, listener);
        }
        MioEvent::IncomingConnectionDidAccept(addr, result) => {
            handle_incoming_connection_did_accept(store, addr, result);
        }
        MioEvent::OutgoingConnectionDidConnect(addr, result) => {
            handle_outgoing_connection_did_connect(store, addr, result);
        }
        MioEvent::IncomingDataIsReady(addr) => {
            handle_incoming_data_is_ready(store, addr);
        }
        MioEvent::IncomingDataDidReceive(addr, result) => {
            handle_incoming_data_did_receive(store, addr, result);
        }
        MioEvent::OutgoingDataDidSend(_, _) => {}
        MioEvent::ConnectionDidClose(addr, result) => {
            handle_connection_did_close(store, addr, result);
        }
        MioEvent::ConnectionDidCloseOnDemand(addr) => {
            handle_connection_did_close_on_demand(store, addr);
        }
    }
}
#[inline(never)]
fn handle_connection_event<S: Service>(store: &mut Store<S>, e: P2pConnectionEvent) {
    match e {
        P2pConnectionEvent::OfferSdpReady(peer_id, res) => match res {
            Err(error) => {
                store.dispatch(P2pConnectionOutgoingAction::OfferSdpCreateError { peer_id, error });
            }
            Ok(sdp) => {
                store.dispatch(P2pConnectionOutgoingAction::OfferSdpCreateSuccess { peer_id, sdp });
            }
        },
        P2pConnectionEvent::AnswerSdpReady(peer_id, res) => match res {
            Err(error) => {
                store
                    .dispatch(P2pConnectionIncomingAction::AnswerSdpCreateError { peer_id, error });
            }
            Ok(sdp) => {
                store
                    .dispatch(P2pConnectionIncomingAction::AnswerSdpCreateSuccess { peer_id, sdp });
            }
        },
        P2pConnectionEvent::AnswerReceived(peer_id, res) => match res {
            P2pConnectionResponse::Accepted(answer) => {
                store.dispatch(P2pConnectionOutgoingAction::AnswerRecvSuccess { peer_id, answer });
            }
            P2pConnectionResponse::Rejected(reason) => {
                store.dispatch(P2pConnectionOutgoingAction::AnswerRecvError {
                    peer_id,
                    error: P2pConnectionErrorResponse::Rejected(reason),
                });
            }
            P2pConnectionResponse::SignalDecryptionFailed => {
                store.dispatch(P2pConnectionOutgoingAction::AnswerRecvError {
                    peer_id,
                    error: P2pConnectionErrorResponse::SignalDecryptionFailed,
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
                store.dispatch(P2pConnectionIncomingAction::FinalizeError { peer_id, error });
            }
            Ok(auth) => {
                let _ = store.dispatch(P2pConnectionOutgoingAction::FinalizeSuccess {
                    peer_id,
                    remote_auth: Some(auth.clone()),
                }) || store.dispatch(P2pConnectionIncomingAction::FinalizeSuccess {
                    peer_id,
                    remote_auth: auth.clone(),
                });
            }
        },
        P2pConnectionEvent::Closed(peer_id) => {
            store.dispatch(P2pDisconnectionAction::PeerClosed { peer_id });
        }
    }
}

#[inline(never)]
fn handle_channel_event<S: Service>(store: &mut Store<S>, meta: ActionMeta, e: P2pChannelEvent) {
    match e {
        P2pChannelEvent::Opened(peer_id, chan_id, res) => match res {
            Err(err) => {
                openmina_core::log::warn!(meta.time(); kind = "P2pChannelEvent::Opened", peer_id = peer_id.to_string(), error = err);
                // TODO(binier): dispatch error action.
            }
            Ok(_) => match chan_id {
                ChannelId::SignalingDiscovery => {
                    store.dispatch(P2pChannelsSignalingDiscoveryAction::Ready { peer_id });
                }
                ChannelId::SignalingExchange => {
                    store.dispatch(P2pChannelsSignalingExchangeAction::Ready { peer_id });
                }
                ChannelId::BestTipPropagation => {
                    store.dispatch(P2pChannelsBestTipAction::Ready { peer_id });
                }
                ChannelId::TransactionPropagation => {
                    store.dispatch(P2pChannelsTransactionAction::Ready { peer_id });
                }
                ChannelId::SnarkPropagation => {
                    store.dispatch(P2pChannelsSnarkAction::Ready { peer_id });
                }
                ChannelId::SnarkJobCommitmentPropagation => {
                    store.dispatch(P2pChannelsSnarkJobCommitmentAction::Ready { peer_id });
                }
                ChannelId::Rpc => {
                    store.dispatch(P2pChannelsRpcAction::Ready { peer_id });
                }
                ChannelId::StreamingRpc => {
                    store.dispatch(P2pChannelsStreamingRpcAction::Ready { peer_id });
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
                store.dispatch(P2pChannelsMessageReceivedAction {
                    peer_id,
                    message: Box::new(message),
                });
            }
        },
        P2pChannelEvent::Closed(peer_id, chan_id) => {
            let reason = P2pDisconnectionReason::P2pChannelClosed(chan_id);
            store.dispatch(P2pDisconnectionAction::Init { peer_id, reason });
        }
    }
}

#[inline(never)]
fn handle_ledger_event<S: Service>(store: &mut Store<S>, event: LedgerEvent) {
    match event {
        LedgerEvent::Write(response) => {
            store.dispatch(LedgerWriteAction::Success { response });
        }
        LedgerEvent::Read(id, response) => {
            store.dispatch(LedgerReadAction::Success { id, response });
        }
    }
}

#[inline(never)]
fn handle_snark_event<S: Service>(store: &mut Store<S>, event: SnarkEvent) {
    match event {
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
    }
}

#[inline(never)]
fn handle_rpc_event<S: Service>(
    store: &mut Store<S>,
    rpc_id: RequestId<RpcIdType>,
    e: Box<RpcRequest>,
) {
    match *e {
        RpcRequest::StateGet(filter) => {
            store.dispatch(RpcAction::GlobalStateGet { rpc_id, filter });
        }
        RpcRequest::StatusGet => {
            store.dispatch(RpcAction::StatusGet { rpc_id });
        }
        RpcRequest::ActionStatsGet(query) => {
            store.dispatch(RpcAction::ActionStatsGet { rpc_id, query });
        }
        RpcRequest::SyncStatsGet(query) => {
            store.dispatch(RpcAction::SyncStatsGet { rpc_id, query });
        }
        RpcRequest::BlockProducerStatsGet => {
            store.dispatch(RpcAction::BlockProducerStatsGet { rpc_id });
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
            store.dispatch(RpcAction::P2pConnectionIncomingInit { rpc_id, opts });
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
        RpcRequest::TransactionPoolGet => {
            store.dispatch(RpcAction::TransactionPool { rpc_id });
        }
        RpcRequest::LedgerAccountsGet(account_query) => {
            store.dispatch(RpcAction::LedgerAccountsGetInit {
                rpc_id,
                account_query,
            });
        }
        RpcRequest::TransactionInject(commands) => {
            store.dispatch(RpcAction::TransactionInjectInit { rpc_id, commands });
        }
        RpcRequest::TransitionFrontierUserCommandsGet => {
            store.dispatch(RpcAction::TransitionFrontierUserCommandsGet { rpc_id });
        }
        RpcRequest::BestChain(max_length) => {
            store.dispatch(RpcAction::BestChain { rpc_id, max_length });
        }
        RpcRequest::ConsensusConstantsGet => {
            store.dispatch(RpcAction::ConsensusConstantsGet { rpc_id });
        }
        RpcRequest::TransactionStatusGet(tx) => {
            store.dispatch(RpcAction::TransactionStatusGet { rpc_id, tx });
        }
    }
}

#[inline(never)]
fn handle_external_snark_worker_event<S: Service>(
    store: &mut Store<S>,
    e: ExternalSnarkWorkerEvent,
) {
    match e {
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
    }
}

#[inline(never)]
fn handle_block_producer_event<S: Service>(store: &mut Store<S>, e: BlockProducerEvent) {
    match e {
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
            Err(err) => {
                todo!("error while trying to produce block proof for block {block_hash} - {err}")
            }
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
    }
}

#[inline(never)]
fn handle_genesis_load_event<S: Service>(
    store: &mut Store<S>,
    res: Result<GenesisConfigLoaded, String>,
) {
    match res {
        Err(err) => todo!("error while trying to load genesis config/ledger. - {err}"),
        Ok(data) => {
            store.dispatch(TransitionFrontierGenesisAction::LedgerLoadSuccess { data });
        }
    }
}

#[inline(never)]
fn handle_new_event<S: Service>(store: &mut Store<S>, meta: ActionMeta, event: Event) {
    match event {
        Event::P2p(e) => handle_p2p_event(store, meta, e),
        Event::Ledger(event) => handle_ledger_event(store, event),
        Event::Snark(event) => handle_snark_event(store, event),
        Event::Rpc(rpc_id, e) => handle_rpc_event(store, rpc_id, e),
        Event::ExternalSnarkWorker(e) => handle_external_snark_worker_event(store, e),
        Event::BlockProducerEvent(e) => handle_block_producer_event(store, e),
        Event::GenesisLoad(res) => handle_genesis_load_event(store, res),
    }
}

pub fn event_source_effects<S: Service>(store: &mut Store<S>, action: EventSourceActionWithMeta) {
    let (action, meta) = action.split();
    match action {
        EventSourceAction::ProcessEvents => {
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
        EventSourceAction::NewEvent { event } => {
            handle_new_event(store, meta, event);
        }
        EventSourceAction::WaitTimeout => {
            store.dispatch(CheckTimeoutsAction {});
        }
        EventSourceAction::WaitForEvents => {}
    }
}
