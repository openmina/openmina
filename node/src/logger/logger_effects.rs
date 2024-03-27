use p2p::P2pNetworkConnectionCloseReason;

use crate::block_producer::vrf_evaluator::BlockProducerVrfEvaluatorAction;
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
use crate::p2p::network::{
    noise::P2pNetworkNoiseAction, pnet::P2pNetworkPnetAction, rpc::P2pNetworkRpcAction,
    scheduler::P2pNetworkSchedulerAction, select::P2pNetworkSelectAction,
    yamux::P2pNetworkYamuxAction, P2pNetworkAction, SelectKind,
};
use crate::p2p::P2pAction;
use crate::snark::work_verify::SnarkWorkVerifyAction;
use crate::snark::SnarkAction;
use crate::transition_frontier::genesis::TransitionFrontierGenesisAction;
use crate::transition_frontier::sync::TransitionFrontierSyncAction;
use crate::transition_frontier::TransitionFrontierAction;
use crate::{Action, ActionWithMetaRef, BlockProducerAction, Service, Store};

pub fn logger_effects<S: Service>(store: &Store<S>, action: ActionWithMetaRef<'_>) {
    let (action, meta) = action.split();
    let kind = action.kind();

    let node_id = store.state().p2p.my_id().to_string();
    // let _guard = openmina_core::log::create_span(&peer_id).entered();

    match action {
        Action::P2p(action) => match action {
            P2pAction::Listen(action) => match action {
                p2p::listen::P2pListenAction::New { listener_id, addr } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("addr: {addr}"),
                        addr = addr.to_string(),
                        listener_id = listener_id.to_string(),
                    );
                }
                p2p::listen::P2pListenAction::Expired { listener_id, addr } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("addr: {addr}"),
                        addr = addr.to_string(),
                        listener_id = listener_id.to_string(),
                    );
                }
                p2p::listen::P2pListenAction::Error { listener_id, error } => {
                    openmina_core::log::warn!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("id: {listener_id}, error: {error}"),
                        error = error,
                        listener_id = listener_id.to_string(),
                    );
                }
                p2p::listen::P2pListenAction::Closed { listener_id, error } => {
                    if let Some(error) = error {
                        openmina_core::log::warn!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("id: {listener_id}, error: {error}"),
                            error = error,
                            listener_id = listener_id.to_string(),
                        );
                    } else {
                        openmina_core::log::info!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("id: {listener_id},"),
                            listener_id = listener_id.to_string(),
                        );
                    }
                }
            },
            P2pAction::Connection(action) => match action {
                P2pConnectionAction::Outgoing(action) => match action {
                    P2pConnectionOutgoingAction::RandomInit => {}
                    P2pConnectionOutgoingAction::Init { opts, .. } => {
                        let peer_id = opts.peer_id();
                        openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string(),
                            transport = opts.kind(),
                        );
                    }
                    P2pConnectionOutgoingAction::Reconnect { opts, .. } => {
                        let peer_id = opts.peer_id();
                        openmina_core::log::info!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string(),
                            transport = opts.kind(),
                        );
                    }
                    P2pConnectionOutgoingAction::OfferSdpCreatePending { .. } => {}
                    P2pConnectionOutgoingAction::OfferSdpCreateError { peer_id, error } => {
                        openmina_core::log::warn!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string(),
                            error = error.clone(),
                        );
                    }
                    P2pConnectionOutgoingAction::OfferSdpCreateSuccess { peer_id, sdp } => {
                        openmina_core::log::debug!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string(),
                            sdp = sdp.clone(),
                        );
                    }
                    P2pConnectionOutgoingAction::OfferReady { peer_id, offer } => {
                        openmina_core::log::debug!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string(),
                            offer = serde_json::to_string(offer).ok()
                        );
                    }
                    P2pConnectionOutgoingAction::OfferSendSuccess { peer_id } => {
                        openmina_core::log::debug!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string(),
                        );
                    }
                    P2pConnectionOutgoingAction::AnswerRecvPending { .. } => {}
                    P2pConnectionOutgoingAction::AnswerRecvError { peer_id, error } => {
                        openmina_core::log::warn!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string(),
                            error = format!("{:?}", error),
                        );
                    }
                    P2pConnectionOutgoingAction::AnswerRecvSuccess { peer_id, answer } => {
                        openmina_core::log::debug!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string(),
                            trace_answer = serde_json::to_string(answer).ok()
                        );
                    }
                    P2pConnectionOutgoingAction::FinalizePending { .. } => {}
                    P2pConnectionOutgoingAction::FinalizeError { peer_id, error } => {
                        openmina_core::log::warn!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string(),
                            error = error.clone(),
                        );
                    }
                    P2pConnectionOutgoingAction::FinalizeSuccess { peer_id } => {
                        openmina_core::log::info!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string()
                        );
                    }
                    P2pConnectionOutgoingAction::Timeout { peer_id } => {
                        openmina_core::log::warn!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string()
                        );
                    }
                    P2pConnectionOutgoingAction::Error { peer_id, error } => {
                        openmina_core::log::warn!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string(),
                            error = format!("{:?}", error),
                        );
                    }
                    P2pConnectionOutgoingAction::Success { peer_id } => {
                        openmina_core::log::info!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string()
                        );
                    }
                },
                P2pConnectionAction::Incoming(action) => match action {
                    P2pConnectionIncomingAction::Init { opts, .. } => {
                        let peer_id = opts.peer_id;
                        openmina_core::log::info!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string(),
                            trace_signaling = format!("{:?}", opts.signaling),
                        );
                    }
                    P2pConnectionIncomingAction::AnswerSdpCreatePending { .. } => {}
                    P2pConnectionIncomingAction::AnswerSdpCreateError { peer_id, error } => {
                        openmina_core::log::warn!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string(),
                            error = format!("{:?}", error),
                        );
                    }
                    P2pConnectionIncomingAction::AnswerSdpCreateSuccess { peer_id, sdp } => {
                        openmina_core::log::info!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string(),
                            trace_sdp = sdp.clone(),
                        );
                    }
                    P2pConnectionIncomingAction::AnswerReady { peer_id, answer } => {
                        openmina_core::log::info!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string(),
                            trace_answer = serde_json::to_string(answer).ok()
                        );
                    }
                    P2pConnectionIncomingAction::AnswerSendSuccess { peer_id } => {
                        openmina_core::log::info!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string(),
                        );
                    }
                    P2pConnectionIncomingAction::FinalizePending { .. } => {}
                    P2pConnectionIncomingAction::FinalizeError { peer_id, error } => {
                        openmina_core::log::warn!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string(),
                            error = format!("{:?}", error),
                        );
                    }
                    P2pConnectionIncomingAction::FinalizeSuccess { peer_id } => {
                        openmina_core::log::info!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string(),
                        );
                    }
                    P2pConnectionIncomingAction::Timeout { peer_id } => {
                        openmina_core::log::warn!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string(),
                        );
                    }
                    P2pConnectionIncomingAction::Error { peer_id, error } => {
                        openmina_core::log::warn!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string(),
                            error = format!("{:?}", error),
                        );
                    }
                    P2pConnectionIncomingAction::Success { peer_id } => {
                        openmina_core::log::info!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string(),
                        );
                    }
                    P2pConnectionIncomingAction::Libp2pReceived { peer_id } => {
                        openmina_core::log::info!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string(),
                        );
                    }
                },
            },
            P2pAction::Disconnection(action) => match action {
                P2pDisconnectionAction::Init { peer_id, reason } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("peer_id: {peer_id}"),
                        peer_id = peer_id.to_string(),
                        reason = format!("{:?}", reason)
                    );
                }
                P2pDisconnectionAction::Finish { peer_id } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("peer_id: {peer_id}"),
                        peer_id = peer_id.to_string()
                    );
                }
            },
            P2pAction::Discovery(action) => match action {
                P2pDiscoveryAction::Init { peer_id } => {
                    openmina_core::log::debug!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("peer_id: {peer_id}"),
                        peer_id = peer_id.to_string()
                    );
                }
                P2pDiscoveryAction::Success { peer_id, .. } => {
                    openmina_core::log::debug!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("peer_id: {peer_id}"),
                        peer_id = peer_id.to_string()
                    );
                }
                P2pDiscoveryAction::KademliaBootstrap => {
                    openmina_core::log::debug!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("bootstrap kademlia"),
                    );
                }
                P2pDiscoveryAction::KademliaInit => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("find node"),
                    );
                }
                P2pDiscoveryAction::KademliaAddRoute { peer_id, addresses } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("add route {peer_id} {:?}", addresses.first()),
                    );
                }
                P2pDiscoveryAction::KademliaSuccess { peers } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("peers: {:?}", peers),
                    );
                }
                P2pDiscoveryAction::KademliaFailure { description } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("{:?}", description),
                    );
                }
            },
            P2pAction::Channels(action) => match action {
                P2pChannelsAction::MessageReceived(_) => {}
                P2pChannelsAction::BestTip(action) => match action {
                    P2pChannelsBestTipAction::Init { peer_id } => {
                        openmina_core::log::debug!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string()
                        );
                    }
                    P2pChannelsBestTipAction::Ready { peer_id } => {
                        openmina_core::log::debug!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string()
                        );
                    }
                    _ => {}
                },
                P2pChannelsAction::Snark(action) => match action {
                    P2pChannelsSnarkAction::Init { peer_id } => {
                        openmina_core::log::debug!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string()
                        );
                    }
                    P2pChannelsSnarkAction::Ready { peer_id } => {
                        openmina_core::log::debug!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string()
                        );
                    }
                    _ => {}
                },
                P2pChannelsAction::SnarkJobCommitment(action) => match action {
                    P2pChannelsSnarkJobCommitmentAction::Init { peer_id } => {
                        openmina_core::log::debug!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", peer_id),
                            peer_id = peer_id.to_string()
                        );
                    }
                    P2pChannelsSnarkJobCommitmentAction::Ready { peer_id } => {
                        openmina_core::log::debug!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", peer_id),
                            peer_id = peer_id.to_string()
                        );
                    }
                    _ => {}
                },
                P2pChannelsAction::Rpc(action) => match action {
                    P2pChannelsRpcAction::Init { peer_id } => {
                        openmina_core::log::debug!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string()
                        );
                    }
                    P2pChannelsRpcAction::Ready { peer_id } => {
                        openmina_core::log::debug!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string()
                        );
                    }
                    P2pChannelsRpcAction::RequestSend {
                        peer_id,
                        id,
                        request,
                    } => {
                        openmina_core::log::debug!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}, rpc_id: {id}, kind: {:?}", request.kind()),
                            peer_id = peer_id.to_string(),
                            rpc_id = id.to_string(),
                            trace_request = serde_json::to_string(request).ok()
                        );
                    }
                    P2pChannelsRpcAction::ResponseReceived {
                        peer_id,
                        id,
                        response,
                    } => {
                        openmina_core::log::debug!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}, rpc_id: {id}"),
                            peer_id = peer_id.to_string(),
                            rpc_id = id.to_string(),
                            trace_response = serde_json::to_string(response).ok()
                        );
                    }
                    _ => {}
                },
            },
            P2pAction::Peer(_) => {}
            // TODO:
            P2pAction::Network(action) => match action {
                P2pNetworkAction::Scheduler(action) => match action {
                    P2pNetworkSchedulerAction::InterfaceDetected { ip } => {
                        openmina_core::log::info!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("ip: {}", ip),
                        );
                    }
                    P2pNetworkSchedulerAction::InterfaceExpired { ip } => {
                        openmina_core::log::info!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("ip: {}", ip),
                        );
                    }
                    P2pNetworkSchedulerAction::IncomingDidAccept { addr, result } => match result {
                        Ok(()) => openmina_core::log::info!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            addr = addr.as_ref().unwrap().to_string(),
                        ),
                        Err(err) => openmina_core::log::error!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            err = err,
                        ),
                    },
                    P2pNetworkSchedulerAction::OutgoingDidConnect { addr, result } => {
                        match result {
                            Ok(()) => openmina_core::log::info!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                addr = addr.to_string(),
                            ),
                            Err(err) => openmina_core::log::warn!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                err = err,
                            ),
                        }
                    }
                    P2pNetworkSchedulerAction::SelectDone {
                        addr,
                        kind: select_kind,
                        protocol,
                        incoming,
                    } if protocol.is_some() => match select_kind {
                        SelectKind::Authentication => {
                            openmina_core::log::info!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                summary = format!("authentication on addr: {}", addr),
                                negotiated = format!("{:?}", protocol),
                                incoming = incoming,
                            )
                        }
                        SelectKind::MultiplexingNoPeerId => {
                            openmina_core::log::info!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                summary = format!("multiplexing on addr: {}", addr),
                                negotiated = format!("{:?}", protocol),
                                incoming = incoming,
                            )
                        }
                        SelectKind::Multiplexing(peer_id) => {
                            openmina_core::log::info!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                summary = format!("multiplexing on addr: {}", addr),
                                peer_id = peer_id.to_string(),
                                negotiated = format!("{:?}", protocol),
                                incoming = incoming,
                            )
                        }
                        SelectKind::Stream(peer_id, stream_id) => {
                            openmina_core::log::info!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                summary = format!("stream on addr: {}", addr),
                                peer_id = peer_id.to_string(),
                                stream_id = stream_id,
                                negotiated = format!("{:?}", protocol),
                                incoming = incoming,
                            )
                        }
                    },
                    P2pNetworkSchedulerAction::SelectError {
                        addr,
                        kind: select_kind,
                        error,
                    } => match select_kind {
                        SelectKind::Authentication => {
                            openmina_core::log::warn!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                error = error,
                                summary = format!("failed select authentication on addr {}: {}", addr, error),
                            )
                        }
                        SelectKind::MultiplexingNoPeerId => {
                            openmina_core::log::warn!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                error = error,
                                summary = format!("failed select multiplexing on addr {}: {}", addr, error),
                            )
                        }
                        SelectKind::Multiplexing(peer_id) => {
                            openmina_core::log::warn!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                error = error,
                                summary = format!("failed select multiplexing on addr {}: {}", addr, error),
                                peer_id = peer_id.to_string(),
                            )
                        }
                        SelectKind::Stream(peer_id, stream_id) => {
                            openmina_core::log::warn!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                error = error,
                                summary = format!("failed select stream on addr {}: {}", addr, error),
                                peer_id = peer_id.to_string(),
                                stream_id = stream_id,
                            )
                        }
                    },
                    P2pNetworkSchedulerAction::Disconnect { addr, reason } => {
                        openmina_core::log::warn!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("disconnecting peer {addr}: {reason}"),
                            addr = openmina_core::log::inner::field::display(addr),
                            reason = openmina_core::log::inner::field::display(reason),
                        )
                    }
                    P2pNetworkSchedulerAction::Error { addr, error } => {
                        openmina_core::log::warn!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("disconnecting peer {addr}: {error}"),
                            addr = openmina_core::log::inner::field::display(addr),
                            error = openmina_core::log::inner::field::display(error),
                        )
                    }
                    P2pNetworkSchedulerAction::Disconnected { addr, reason } => match reason {
                        P2pNetworkConnectionCloseReason::Disconnect(reason) => {
                            openmina_core::log::warn!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                summary = format!("disconnected peer {addr}: {reason}"),
                                addr = openmina_core::log::inner::field::display(addr),
                                reason = openmina_core::log::inner::field::display(reason),
                            )
                        }
                        P2pNetworkConnectionCloseReason::Error(error) => {
                            openmina_core::log::warn!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                summary = format!("disconnected peer {addr}: {error}"),
                                addr = openmina_core::log::inner::field::display(addr),
                                error = openmina_core::log::inner::field::display(error),
                            )
                        }
                    },
                    _ => {}
                },
                P2pNetworkAction::Pnet(action) => match action {
                    P2pNetworkPnetAction::SetupNonce { addr, incoming, .. } => {
                        openmina_core::log::info!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            addr = addr.to_string(),
                            incoming = incoming,
                        )
                    }
                    _ => {}
                },
                P2pNetworkAction::Select(action) => match action {
                    P2pNetworkSelectAction::Init {
                        addr,
                        kind: select_kind,
                        incoming,
                        ..
                    } => match select_kind {
                        SelectKind::Authentication => {
                            openmina_core::log::debug!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                summary = format!("authentication on addr: {}", addr),
                                incoming = incoming,
                            )
                        }
                        SelectKind::MultiplexingNoPeerId => {
                            openmina_core::log::debug!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                summary = format!("multiplexing on addr: {}", addr),
                                incoming = incoming,
                            )
                        }
                        SelectKind::Multiplexing(peer_id) => {
                            openmina_core::log::debug!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                summary = format!("multiplexing on addr: {}", addr),
                                peer_id = peer_id.to_string(),
                                incoming = incoming,
                            )
                        }
                        SelectKind::Stream(peer_id, stream_id) => {
                            openmina_core::log::debug!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                summary = format!("stream on addr: {}", addr),
                                peer_id = peer_id.to_string(),
                                stream_id = stream_id,
                                incoming = incoming,
                            )
                        }
                    },
                    P2pNetworkSelectAction::IncomingToken {
                        addr,
                        kind: stream_kind,
                        token,
                    } => match stream_kind {
                        SelectKind::Authentication => {
                            openmina_core::log::debug!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                summary = format!("authentication on addr: {}", addr),
                                token = format!("{:?}", token),
                            )
                        }
                        SelectKind::MultiplexingNoPeerId => {
                            openmina_core::log::debug!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                summary = format!("multiplexing on addr: {}", addr),
                                token = format!("{:?}", token),
                            )
                        }
                        SelectKind::Multiplexing(peer_id) => {
                            openmina_core::log::debug!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                summary = format!("multiplexing on addr: {}", addr),
                                peer_id = peer_id.to_string(),
                                token = format!("{:?}", token),
                            )
                        }
                        SelectKind::Stream(peer_id, stream_id) => {
                            openmina_core::log::debug!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                summary = format!("stream on addr: {}", addr),
                                peer_id = peer_id.to_string(),
                                stream_id = stream_id,
                                token = format!("{:?}", token),
                            )
                        }
                    },
                    P2pNetworkSelectAction::OutgoingTokens {
                        addr,
                        kind: select_kind,
                        tokens,
                    } => match select_kind {
                        SelectKind::Authentication => {
                            openmina_core::log::debug!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                summary = format!("authentication on addr: {}", addr),
                                tokens = format!("{:?}", tokens),
                            )
                        }
                        SelectKind::MultiplexingNoPeerId => {
                            openmina_core::log::debug!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                summary = format!("multiplexing on addr: {}", addr),
                                tokens = format!("{:?}", tokens),
                            )
                        }
                        SelectKind::Multiplexing(peer_id) => {
                            openmina_core::log::debug!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                summary = format!("multiplexing on addr: {}", addr),
                                peer_id = peer_id.to_string(),
                                tokens = format!("{:?}", tokens),
                            )
                        }
                        SelectKind::Stream(peer_id, stream_id) => {
                            openmina_core::log::debug!(
                                meta.time();
                                node_id = node_id,
                                kind = kind.to_string(),
                                summary = format!("stream on addr: {}", addr),
                                peer_id = peer_id.to_string(),
                                stream_id = stream_id,
                                tokens = format!("{:?}", tokens),
                            )
                        }
                    },
                    _ => {}
                },
                P2pNetworkAction::Noise(action) => match action {
                    P2pNetworkNoiseAction::Init { addr, incoming, .. } => {
                        openmina_core::log::info!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            addr = addr.to_string(),
                            incoming = incoming,
                        )
                    }
                    P2pNetworkNoiseAction::HandshakeDone {
                        addr,
                        peer_id,
                        incoming,
                    } => {
                        openmina_core::log::info!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            addr = addr.to_string(),
                            incoming = incoming,
                            peer_id = peer_id.to_string(),
                        )
                    }
                    P2pNetworkNoiseAction::IncomingChunk { addr, data } => {
                        openmina_core::log::debug!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            addr = addr.to_string(),
                            data = format!("{:?}", data),
                        )
                    }
                    P2pNetworkNoiseAction::OutgoingChunk { addr, data } => {
                        openmina_core::log::debug!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            addr = addr.to_string(),
                            data = format!("{:?}", data),
                        )
                    }
                    P2pNetworkNoiseAction::DecryptedData { addr, data, .. } => {
                        openmina_core::log::debug!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            addr = addr.to_string(),
                            data = format!("{:?}", data),
                        )
                    }
                    _ => {}
                },
                P2pNetworkAction::Yamux(action) => match action {
                    P2pNetworkYamuxAction::IncomingData { addr, data } => {
                        openmina_core::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            addr = addr.to_string(),
                            data = format!("{:?}", data),
                        )
                    }
                    P2pNetworkYamuxAction::IncomingFrame { addr, frame } => {
                        openmina_core::log::debug!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            addr = addr.to_string(),
                            frame = format!("{:?}", frame),
                        )
                    }
                    P2pNetworkYamuxAction::OutgoingFrame { addr, frame } => {
                        openmina_core::log::debug!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            addr = addr.to_string(),
                            frame = format!("{:?}", frame),
                        )
                    }
                    _ => {}
                },
                P2pNetworkAction::Rpc(action) => match action {
                    P2pNetworkRpcAction::Init {
                        addr,
                        peer_id,
                        stream_id,
                        incoming,
                    } => {
                        openmina_core::log::info!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            addr = addr.to_string(),
                            peer_id = peer_id.to_string(),
                            stream_id = stream_id,
                            incoming = incoming,
                        )
                    }
                    P2pNetworkRpcAction::IncomingMessage {
                        addr,
                        peer_id,
                        stream_id,
                        message,
                    } => {
                        let msg = match &message {
                            p2p::RpcMessage::Handshake => "handshake".to_owned(),
                            p2p::RpcMessage::Heartbeat => "heartbeat".to_owned(),
                            p2p::RpcMessage::Query { header, .. } => format!("{header:?}"),
                            p2p::RpcMessage::Response { header, .. } => format!("{header:?}"),
                        };
                        openmina_core::log::debug!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            addr = addr.to_string(),
                            peer_id = peer_id.to_string(),
                            stream_id = stream_id,
                            msg = msg,
                        )
                    }
                    _ => {}
                },
                P2pNetworkAction::Kad(_) => {}
            },
        },
        Action::ExternalSnarkWorker(a) => {
            use crate::external_snark_worker::ExternalSnarkWorkerAction;
            match a {
                ExternalSnarkWorkerAction::Start
                | ExternalSnarkWorkerAction::Started
                | ExternalSnarkWorkerAction::Kill
                | ExternalSnarkWorkerAction::Killed
                | ExternalSnarkWorkerAction::WorkCancelled
                | ExternalSnarkWorkerAction::PruneWork => {
                    openmina_core::log::debug!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        trace_action = serde_json::to_string(&a).ok()
                    )
                }
                ExternalSnarkWorkerAction::SubmitWork { job_id, .. } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        work_id = job_id.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::WorkResult { .. } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::CancelWork => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::WorkError { error, .. } => {
                    openmina_core::log::warn!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        error = error.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::Error { error, .. } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        error = error.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::StartTimeout { .. } => {
                    openmina_core::log::warn!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::WorkTimeout { .. } => {
                    openmina_core::log::warn!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                    )
                }
            }
        }
        Action::Snark(a) => match a {
            SnarkAction::WorkVerify(a) => match a {
                SnarkWorkVerifyAction::Init {
                    req_id,
                    batch,
                    sender,
                } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("id: {}, batch size: {}", req_id, batch.len()),
                        peer_id = sender,
                        rpc_id = req_id.to_string(),
                        trace_batch = serde_json::to_string(&batch.iter().map(|v| v.job_id()).collect::<Vec<_>>()).ok()
                    );
                }
                SnarkWorkVerifyAction::Error { req_id, .. } => {
                    let Some(req) = store.state().snark.work_verify.jobs.get(*req_id) else {
                        return;
                    };
                    openmina_core::log::warn!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("id: {}, batch size: {}", req_id, req.batch().len()),
                        peer_id = req.sender(),
                        rpc_id = req_id.to_string(),
                        trace_batch = serde_json::to_string(&req.batch().iter().map(|v| v.job_id()).collect::<Vec<_>>()).ok()
                    );
                }
                SnarkWorkVerifyAction::Success { req_id } => {
                    let Some(req) = store.state().snark.work_verify.jobs.get(*req_id) else {
                        return;
                    };
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("id: {}, batch size: {}", req_id, req.batch().len()),
                        peer_id = req.sender(),
                        rpc_id = req_id.to_string(),
                        trace_batch = serde_json::to_string(&req.batch().iter().map(|v| v.job_id()).collect::<Vec<_>>()).ok()
                    );
                }
                _ => {}
            },
            _ => {}
        },
        Action::TransitionFrontier(a) => match a {
            TransitionFrontierAction::Genesis(a) => match a {
                TransitionFrontierGenesisAction::ProvePending => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("Proving genesis block"),
                    );
                }
                TransitionFrontierGenesisAction::ProveSuccess { .. } => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("Genesis block proved"),
                    );
                }
                _ => {}
            },
            TransitionFrontierAction::GenesisInject => {
                let Some(genesis) = store
                    .state()
                    .transition_frontier
                    .genesis
                    .block_with_real_or_dummy_proof()
                else {
                    return;
                };
                openmina_core::log::info!(
                    meta.time();
                    kind = kind.to_string(),
                    summary = format!("Transition frontier reconstructed genesis ledger({}) and block({})", genesis.snarked_ledger_hash(), genesis.hash()),
                    genesis_block_hash = genesis.hash().to_string(),
                    genesis_ledger_hash = genesis.snarked_ledger_hash().to_string(),
                );
            }
            TransitionFrontierAction::Sync(action) => match action {
                TransitionFrontierSyncAction::Init {
                    best_tip,
                    root_block,
                    ..
                } => openmina_core::log::info!(
                    meta.time();
                    node_id = node_id,
                    kind = kind.to_string(),
                    summary = "Transition frontier sync init".to_string(),
                    block_hash = best_tip.hash.to_string(),
                    root_block_hash = root_block.hash.to_string(),
                ),
                TransitionFrontierSyncAction::BestTipUpdate {
                    best_tip,
                    root_block,
                    ..
                } => openmina_core::log::info!(
                    meta.time();
                    node_id = node_id,
                    kind = kind.to_string(),
                    summary = "New best tip received".to_string(),
                    block_hash = best_tip.hash.to_string(),
                    root_block_hash = root_block.hash.to_string(),
                ),
                TransitionFrontierSyncAction::LedgerStakingPending => openmina_core::log::info!(
                    meta.time();
                    node_id = node_id,
                    kind = kind.to_string(),
                    summary = "Staking ledger sync pending".to_string(),
                ),
                TransitionFrontierSyncAction::LedgerStakingSuccess => openmina_core::log::info!(
                    meta.time();
                    node_id = node_id,
                    kind = kind.to_string(),
                    summary = "Staking ledger sync success".to_string(),
                ),
                TransitionFrontierSyncAction::LedgerNextEpochPending => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = "Next epoch ledger sync pending".to_string(),
                    )
                }
                TransitionFrontierSyncAction::LedgerNextEpochSuccess => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = "Next epoch ledger sync pending".to_string(),
                    )
                }
                TransitionFrontierSyncAction::LedgerRootPending => openmina_core::log::info!(
                    meta.time();
                    node_id = node_id,
                    kind = kind.to_string(),
                    summary = "Transition frontier root ledger sync pending".to_string(),
                ),
                TransitionFrontierSyncAction::LedgerRootSuccess => openmina_core::log::info!(
                    meta.time();
                    node_id = node_id,
                    kind = kind.to_string(),
                    summary = "Transition frontier root ledger sync success".to_string(),
                ),
                _other => openmina_core::log::debug!(
                    meta.time();
                    node_id = node_id,
                    kind = kind.to_string(),
                ),
            },
            TransitionFrontierAction::Synced { .. } => openmina_core::log::info!(
                meta.time();
                node_id = node_id,
                kind = kind.to_string(),
                summary = "Transition frontier synced".to_string(),
            ),
        },
        Action::BlockProducer(a) => match a {
            BlockProducerAction::VrfEvaluator(a) => match a {
                BlockProducerVrfEvaluatorAction::ProcessSlotEvaluationSuccess {
                    vrf_output,
                    ..
                } => match vrf_output {
                    vrf::VrfEvaluationOutput::SlotWon(won_slot) => {
                        openmina_core::log::info!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("Slot evaluation result - won slot"),
                            global_slot = won_slot.global_slot,
                            vrf_output = won_slot.vrf_output.to_string(),
                        )
                    }
                    vrf::VrfEvaluationOutput::SlotLost(_) => {
                        openmina_core::log::debug!(
                            meta.time();
                            node_id = node_id,
                            kind = kind.to_string(),
                            summary = format!("Slot evaluation result - lost slot: {:?}", vrf_output),
                        )
                    }
                },
                BlockProducerVrfEvaluatorAction::EvaluateSlot { vrf_input } => {
                    openmina_core::log::debug!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("Vrf Evaluation requested: {:?}", vrf_input),
                    )
                }
                BlockProducerVrfEvaluatorAction::CheckEpochEvaluability {
                    current_epoch_number,
                    current_best_tip_global_slot,
                    current_best_tip_slot,
                    ..
                } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("Checking possible Vrf evaluations"),
                        status = store.state().block_producer.vrf_evaluator().unwrap().status.to_string(), // TODO: keep? if yes, no unwrap
                        status_details = format!("{:?}", store.state().block_producer.vrf_evaluator().unwrap().status),
                        current_epoch = current_epoch_number,
                        current_best_tip_global_slot = current_best_tip_global_slot,
                        current_best_tip_slot = current_best_tip_slot,
                    )
                }
                BlockProducerVrfEvaluatorAction::InitializeEpochEvaluation {
                    current_epoch_number,
                    current_best_tip_global_slot,
                    current_best_tip_slot,
                    ..
                } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("Constructing delegator table"),
                        status = store.state().block_producer.vrf_evaluator().unwrap().status.to_string(), // TODO: keep? if yes, no unwrap
                        current_epoch = current_epoch_number,
                        current_best_tip_global_slot = current_best_tip_global_slot,
                        current_best_tip_slot = current_best_tip_slot,
                    )
                }
                BlockProducerVrfEvaluatorAction::BeginDelegatorTableConstruction {
                    current_epoch_number,
                    current_best_tip_global_slot,
                    current_best_tip_slot,
                    ..
                } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("Constructing delegator table"),
                        status = store.state().block_producer.vrf_evaluator().unwrap().status.to_string(), // TODO: keep? if yes, no unwrap
                        current_epoch = current_epoch_number,
                        current_best_tip_global_slot = current_best_tip_global_slot,
                        current_best_tip_slot = current_best_tip_slot,
                    )
                }
                BlockProducerVrfEvaluatorAction::BeginEpochEvaluation {
                    current_epoch_number,
                    current_best_tip_global_slot,
                    current_best_tip_slot,
                    ..
                } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("Starting epoch evaluation"),
                        status = store.state().block_producer.vrf_evaluator().unwrap().status.to_string(), // TODO: keep? if yes, no unwrap
                        current_epoch = current_epoch_number,
                        current_best_tip_global_slot = current_best_tip_global_slot,
                        current_best_tip_slot = current_best_tip_slot,
                    )
                }
                BlockProducerVrfEvaluatorAction::FinalizeDelegatorTableConstruction {
                    current_epoch_number,
                    current_best_tip_global_slot,
                    current_best_tip_slot,
                    ..
                } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("Delegator table constructed"),
                        status = store.state().block_producer.vrf_evaluator().unwrap().status.to_string(), // TODO: keep? if yes, no unwrap
                        current_epoch = current_epoch_number,
                        current_best_tip_global_slot = current_best_tip_global_slot,
                        current_best_tip_slot = current_best_tip_slot,
                    )
                }
                BlockProducerVrfEvaluatorAction::RecordLastBlockHeightInEpoch {
                    epoch_number,
                    last_block_height,
                    ..
                } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("Saving last block height in epoch"),
                        status = store.state().block_producer.vrf_evaluator().unwrap().status.to_string(), // TODO: keep? if yes, no unwrap
                        epoch = epoch_number,
                        last_block_height = last_block_height,
                    )
                }
                BlockProducerVrfEvaluatorAction::InitializeEvaluator { .. } => {}
                BlockProducerVrfEvaluatorAction::FinalizeEvaluatorInitialization { .. } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("Vrf evaluator initilaized"),
                    )
                }
                BlockProducerVrfEvaluatorAction::FinishEpochEvaluation {
                    epoch_number,
                    latest_evaluated_global_slot,
                } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("Epoch evaluation finished"),
                        epoch_number = epoch_number,
                        latest_evaluated_global_slot = latest_evaluated_global_slot,
                    )
                }
                BlockProducerVrfEvaluatorAction::WaitForNextEvaluation {
                    current_epoch_number,
                    current_best_tip_height,
                    ..
                } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("Waiting for epoch to evaluate"),
                        epoch_number = current_epoch_number,
                        current_best_tip_height = current_best_tip_height,
                    )
                }
                BlockProducerVrfEvaluatorAction::SelectInitialSlot {
                    current_epoch_number,
                    current_best_tip_height,
                    ..
                } => {
                    openmina_core::log::info!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("Selecting starting slot"),
                        epoch_number = current_epoch_number,
                        current_best_tip_height = current_best_tip_height,
                    )
                }
                BlockProducerVrfEvaluatorAction::CheckEpochBounds { .. } => {
                    openmina_core::log::trace!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("Checking epoch bounds"),
                        status = format!("{:?}", store.state().block_producer.vrf_evaluator().unwrap().status),
                    )
                }
                BlockProducerVrfEvaluatorAction::CleanupOldSlots { .. } => {
                    openmina_core::log::trace!(
                        meta.time();
                        node_id = node_id,
                        kind = kind.to_string(),
                        summary = format!("Cleaning up old won slots"),
                    )
                }
                _ => {}
            },
            BlockProducerAction::BestTipUpdate { .. } => {}
            BlockProducerAction::WonSlot { won_slot } => {
                openmina_core::log::info!(
                    meta.time();
                    node_id = node_id,
                    kind = kind.to_string(),
                    summary = format!("Won slot"),
                    slot = won_slot.global_slot.slot_number.as_u32(),
                    slot_time = openmina_core::log::to_rfc_3339(won_slot.slot_time).unwrap(),
                    current_time = openmina_core::log::to_rfc_3339(meta.time()).unwrap(),
                )
            }
            _ => {}
        },
        _ => {}
    }
}
