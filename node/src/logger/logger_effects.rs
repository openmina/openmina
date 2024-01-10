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
use crate::transition_frontier::sync::TransitionFrontierSyncAction;
use crate::transition_frontier::TransitionFrontierAction;
use crate::{Action, ActionWithMetaRef, BlockProducerAction, Service, Store};

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
                    P2pChannelsBestTipAction::Init { peer_id } => {
                        openmina_core::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}"),
                            peer_id = peer_id.to_string()
                        );
                    }
                    P2pChannelsBestTipAction::Ready { peer_id } => {
                        openmina_core::log::debug!(
                            meta.time();
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
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}", ),
                            peer_id = peer_id.to_string()
                        );
                    }
                    P2pChannelsSnarkAction::Ready { peer_id } => {
                        openmina_core::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}", ),
                            peer_id = peer_id.to_string()
                        );
                    }
                    _ => {}
                },
                P2pChannelsAction::SnarkJobCommitment(action) => match action {
                    P2pChannelsSnarkJobCommitmentAction::Init { peer_id } => {
                        openmina_core::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {}", peer_id),
                            peer_id = peer_id.to_string()
                        );
                    }
                    P2pChannelsSnarkJobCommitmentAction::Ready { peer_id } => {
                        openmina_core::log::debug!(
                            meta.time();
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
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}", ),
                            peer_id = peer_id.to_string()
                        );
                    }
                    P2pChannelsRpcAction::Ready { peer_id } => {
                        openmina_core::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("peer_id: {peer_id}", ),
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
                    P2pNetworkSchedulerAction::InterfaceDetected(action) => {
                        openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("ip: {}", action.ip),
                        );
                    }
                    P2pNetworkSchedulerAction::InterfaceExpired(action) => {
                        openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("ip: {}", action.ip),
                        );
                    }
                    P2pNetworkSchedulerAction::IncomingDidAccept(action) => match &action.result {
                        Ok(()) => openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            addr = action.addr.as_ref().unwrap().to_string(),
                        ),
                        Err(err) => openmina_core::log::error!(
                            meta.time();
                            kind = kind.to_string(),
                            err = err,
                        ),
                    },
                    P2pNetworkSchedulerAction::OutgoingDidConnect(action) => match &action.result {
                        Ok(()) => openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            addr = action.addr.to_string(),
                        ),
                        Err(err) => openmina_core::log::error!(
                            meta.time();
                            kind = kind.to_string(),
                            err = err,
                        ),
                    },
                    P2pNetworkSchedulerAction::SelectDone(action) => match action.kind {
                        SelectKind::Authentication => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("authentication on addr: {}", action.addr),
                                negotiated = format!("{:?}", action.protocol),
                                incoming = action.incoming,
                            )
                        }
                        SelectKind::Multiplexing(peer_id) => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("multiplexing on addr: {}", action.addr),
                                peer_id = peer_id.to_string(),
                                negotiated = format!("{:?}", action.protocol),
                                incoming = action.incoming,
                            )
                        }
                        SelectKind::Stream(peer_id, stream_id) => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("stream on addr: {}", action.addr),
                                peer_id = peer_id.to_string(),
                                stream_id = stream_id,
                                negotiated = format!("{:?}", action.protocol),
                                incoming = action.incoming,
                            )
                        }
                    },
                    P2pNetworkSchedulerAction::SelectError(action) => match action.kind {
                        SelectKind::Authentication => {
                            openmina_core::log::error!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("failed select authentication on addr: {}", action.addr),
                            )
                        }
                        SelectKind::Multiplexing(peer_id) => {
                            openmina_core::log::error!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("failed select multiplexing on addr: {}", action.addr),
                                peer_id = peer_id.to_string(),
                            )
                        }
                        SelectKind::Stream(peer_id, stream_id) => {
                            openmina_core::log::error!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("failed select stream on addr: {}", action.addr),
                                peer_id = peer_id.to_string(),
                                stream_id = stream_id,
                            )
                        }
                    },
                    _ => {}
                },
                P2pNetworkAction::Pnet(action) => match action {
                    P2pNetworkPnetAction::SetupNonce(action) => {
                        openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            addr = action.addr.to_string(),
                            incoming = action.incoming,
                        )
                    }
                    _ => {}
                },
                P2pNetworkAction::Select(action) => match action {
                    P2pNetworkSelectAction::Init(action) => match action.kind {
                        SelectKind::Authentication => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("authentication on addr: {}", action.addr),
                                incoming = action.incoming,
                            )
                        }
                        SelectKind::Multiplexing(peer_id) => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("multiplexing on addr: {}", action.addr),
                                peer_id = peer_id.to_string(),
                                incoming = action.incoming,
                            )
                        }
                        SelectKind::Stream(peer_id, stream_id) => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("stream on addr: {}", action.addr),
                                peer_id = peer_id.to_string(),
                                stream_id = stream_id,
                                incoming = action.incoming,
                            )
                        }
                    },
                    P2pNetworkSelectAction::IncomingToken(action) => match action.kind {
                        SelectKind::Authentication => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("authentication on addr: {}", action.addr),
                                token = format!("{:?}", action.token),
                            )
                        }
                        SelectKind::Multiplexing(peer_id) => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("multiplexing on addr: {}", action.addr),
                                peer_id = peer_id.to_string(),
                                token = format!("{:?}", action.token),
                            )
                        }
                        SelectKind::Stream(peer_id, stream_id) => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("stream on addr: {}", action.addr),
                                peer_id = peer_id.to_string(),
                                stream_id = stream_id,
                                token = format!("{:?}", action.token),
                            )
                        }
                    },
                    P2pNetworkSelectAction::OutgoingTokens(action) => match action.kind {
                        SelectKind::Authentication => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("authentication on addr: {}", action.addr),
                                tokens = format!("{:?}", action.tokens),
                            )
                        }
                        SelectKind::Multiplexing(peer_id) => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("multiplexing on addr: {}", action.addr),
                                peer_id = peer_id.to_string(),
                                tokens = format!("{:?}", action.tokens),
                            )
                        }
                        SelectKind::Stream(peer_id, stream_id) => {
                            openmina_core::log::info!(
                                meta.time();
                                kind = kind.to_string(),
                                summary = format!("stream on addr: {}", action.addr),
                                peer_id = peer_id.to_string(),
                                stream_id = stream_id,
                                tokens = format!("{:?}", action.tokens),
                            )
                        }
                    },
                    _ => {}
                },
                P2pNetworkAction::Noise(action) => match action {
                    P2pNetworkNoiseAction::Init(action) => {
                        openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            addr = action.addr.to_string(),
                            incoming = action.incoming,
                        )
                    }
                    P2pNetworkNoiseAction::HandshakeDone(action) => {
                        openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            addr = action.addr.to_string(),
                            incoming = action.incoming,
                            peer_id = action.peer_id.to_string(),
                        )
                    }
                    _ => {}
                },
                P2pNetworkAction::Yamux(action) => match action {
                    P2pNetworkYamuxAction::IncomingFrame(action) => {
                        openmina_core::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            addr = action.addr.to_string(),
                            frame = format!("{:?}", action.frame),
                        )
                    }
                    P2pNetworkYamuxAction::OutgoingFrame(action) => {
                        openmina_core::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            addr = action.addr.to_string(),
                            frame = format!("{:?}", action.frame),
                        )
                    }
                    _ => {}
                },
                P2pNetworkAction::Rpc(action) => match action {
                    P2pNetworkRpcAction::Init(action) => {
                        openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            addr = action.addr.to_string(),
                            peer_id = action.peer_id.to_string(),
                            stream_id = action.stream_id,
                            incoming = action.incoming,
                        )
                    }
                    P2pNetworkRpcAction::IncomingMessage(action) => {
                        let msg = match &action.message {
                            p2p::RpcMessage::Handshake => "handshake".to_owned(),
                            p2p::RpcMessage::Heartbeat => "heartbeat".to_owned(),
                            p2p::RpcMessage::Query { header, .. } => format!("{header:?}"),
                            p2p::RpcMessage::Response { header, .. } => format!("{header:?}"),
                        };
                        openmina_core::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            addr = action.addr.to_string(),
                            peer_id = action.peer_id.to_string(),
                            stream_id = action.stream_id,
                            msg = msg,
                        )
                    }
                    _ => {}
                },
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
                        kind = kind.to_string(),
                        trace_action = serde_json::to_string(&a).ok()
                    )
                }
                ExternalSnarkWorkerAction::SubmitWork { job_id, .. } => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        work_id = job_id.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::WorkResult { .. } => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::CancelWork => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::WorkError { error, .. } => {
                    openmina_core::log::warn!(
                        meta.time();
                        kind = kind.to_string(),
                        error = error.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::Error { error, .. } => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        error = error.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::StartTimeout { .. } => {
                    openmina_core::log::warn!(
                        meta.time();
                        kind = kind.to_string(),
                    )
                }
                ExternalSnarkWorkerAction::WorkTimeout { .. } => {
                    openmina_core::log::warn!(
                        meta.time();
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
        Action::BlockProducer(a) => match a {
            BlockProducerAction::VrfEvaluator(a) => match a {
                BlockProducerVrfEvaluatorAction::EpochDataUpdate(a) => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("seed: {}, ledger: {}", a.epoch_data.seed.to_string(), a.epoch_data.ledger.hash.to_string()),
                    );
                }
                BlockProducerVrfEvaluatorAction::UpdateProducerAndDelegates(_) => {}
                BlockProducerVrfEvaluatorAction::UpdateProducerAndDelegatesSuccess(a) => {
                    openmina_core::log::info!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("Current epoch accounts: {:?}, Next epoch accounts: {:?}",
                            a.current_epoch_producer_and_delegators.values().map(| a | a.0.clone()).collect::<Vec<_>>(),
                            a.next_epoch_producer_and_delegators.values().map(| a | a.0.clone()).collect::<Vec<_>>()
                        ),
                    );
                }
                BlockProducerVrfEvaluatorAction::EvaluationSuccess(a) => match a.vrf_output {
                    vrf::VrfEvaluationOutput::SlotWon(_) => {
                        openmina_core::log::info!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("Slot evaluation result - won slot: {:?}", a.vrf_output),
                        )
                    }
                    vrf::VrfEvaluationOutput::SlotLost(_) => {
                        openmina_core::log::debug!(
                            meta.time();
                            kind = kind.to_string(),
                            summary = format!("Slot evaluation result - lost slot: {:?}", a.vrf_output),
                        )
                    }
                },
                BlockProducerVrfEvaluatorAction::EvaluateVrf(a) => {
                    openmina_core::log::debug!(
                        meta.time();
                        kind = kind.to_string(),
                        summary = format!("Vrf Evaluation requested: {:?}", a.vrf_input),
                    )
                }
            },
            BlockProducerAction::BestTipUpdate(_) => {}
            _ => {}
        },
        _ => {}
    }
}
