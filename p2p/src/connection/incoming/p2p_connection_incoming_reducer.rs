#[cfg(feature = "p2p-libp2p")]
use std::net::{IpAddr, SocketAddr};

use openmina_core::{bug_condition, debug, warn, Substate};
use redux::{ActionWithMeta, Dispatcher, Timestamp};

use crate::{
    connection::{
        incoming::P2pConnectionIncomingError,
        incoming_effectful::P2pConnectionIncomingEffectfulAction,
        outgoing::P2pConnectionOutgoingInitOpts, P2pConnectionResponse, P2pConnectionState,
    },
    disconnection::{P2pDisconnectionAction, P2pDisconnectionReason},
    webrtc::{HttpSignalingInfo, SignalingMethod},
    ConnectionAddr, P2pNetworkSchedulerAction, P2pPeerAction, P2pPeerState, P2pPeerStatus,
    P2pState, PeerId,
};

use super::{
    super::{incoming::P2pConnectionIncomingState, RejectionReason},
    IncomingSignalingMethod, P2pConnectionIncomingAction,
};

impl P2pConnectionIncomingState {
    /// Substate is accessed
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, P2pState>,
        action: ActionWithMeta<&P2pConnectionIncomingAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();
        let time = meta.time();
        let peer_id = *action.peer_id();
        let p2p_state = state_context.get_substate_mut()?;
        let my_id = p2p_state.my_id();

        match action {
            P2pConnectionIncomingAction::Init { opts, rpc_id } => {
                let state = p2p_state
                    .peers
                    .entry(peer_id)
                    .or_insert_with(|| P2pPeerState {
                        is_libp2p: false,
                        dial_opts: opts.offer.listen_port.map(|listen_port| {
                            let signaling = match opts.signaling {
                                IncomingSignalingMethod::Http => {
                                    SignalingMethod::Http(HttpSignalingInfo {
                                        host: opts.offer.host.clone(),
                                        port: listen_port,
                                    })
                                }
                            };
                            P2pConnectionOutgoingInitOpts::WebRTC { peer_id, signaling }
                        }),
                        status: P2pPeerStatus::Connecting(P2pConnectionState::incoming_init(opts)),
                        identify: None,
                    });

                state.status =
                    P2pPeerStatus::Connecting(P2pConnectionState::Incoming(Self::Init {
                        time: meta.time(),
                        signaling: opts.signaling.clone(),
                        offer: opts.offer.clone(),
                        rpc_id: *rpc_id,
                    }));

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pConnectionIncomingEffectfulAction::Init { opts: opts.clone() });
                Ok(())
            }
            P2pConnectionIncomingAction::AnswerSdpCreatePending { .. } => {
                let state = p2p_state
                    .incoming_peer_connection_mut(&peer_id)
                    .ok_or_else(|| format!("Invalid state for: {:?}", action))?;
                if let Self::Init {
                    signaling,
                    offer,
                    rpc_id,
                    ..
                } = state
                {
                    *state = Self::AnswerSdpCreatePending {
                        time: meta.time(),
                        signaling: signaling.clone(),
                        offer: offer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                } else {
                    bug_condition!(
                        "Invalid state for `P2pConnectionIncomingAction::AnswerSdpCreatePending`: {:?}",
                        state
                    );
                }
                Ok(())
            }
            P2pConnectionIncomingAction::AnswerSdpCreateError { peer_id, error } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pConnectionIncomingAction::Error {
                    peer_id: *peer_id,
                    error: P2pConnectionIncomingError::SdpCreateError(error.to_owned()),
                });
                Ok(())
            }
            P2pConnectionIncomingAction::AnswerSdpCreateSuccess { sdp, .. } => {
                let state = p2p_state
                    .incoming_peer_connection_mut(&peer_id)
                    .ok_or_else(|| format!("Invalid state for: {:?}", action))?;
                if let Self::AnswerSdpCreatePending {
                    signaling,
                    offer,
                    rpc_id,
                    ..
                } = state
                {
                    *state = Self::AnswerSdpCreateSuccess {
                        time: meta.time(),
                        signaling: signaling.clone(),
                        offer: offer.clone(),
                        sdp: sdp.clone(),
                        rpc_id: rpc_id.take(),
                    };
                } else {
                    bug_condition!(
                        "Invalid state for `P2pConnectionIncomingAction::AnswerSdpCreateSuccess`: {:?}",
                        state
                    );
                    return Ok(());
                }

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;
                let answer = Box::new(crate::webrtc::Answer {
                    sdp: sdp.to_owned(),
                    identity_pub_key: p2p_state.config.identity_pub_key.clone(),
                    target_peer_id: peer_id,
                });
                dispatcher.push(P2pConnectionIncomingAction::AnswerReady { peer_id, answer });
                Ok(())
            }
            P2pConnectionIncomingAction::AnswerReady { peer_id, answer } => {
                let state = p2p_state
                    .incoming_peer_connection_mut(peer_id)
                    .ok_or_else(|| format!("Invalid state for: {:?}", action))?;
                if let Self::AnswerSdpCreateSuccess {
                    signaling,
                    offer,
                    rpc_id,
                    ..
                } = state
                {
                    *state = Self::AnswerReady {
                        time: meta.time(),
                        signaling: signaling.clone(),
                        offer: offer.clone(),
                        answer: answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                } else {
                    bug_condition!(
                        "Invalid state for `P2pConnectionIncomingAction::AnswerReady`: {:?}",
                        state
                    );
                }
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;

                dispatcher.push(P2pConnectionIncomingEffectfulAction::AnswerSend {
                    peer_id: *peer_id,
                    answer: answer.clone(),
                });

                if let Some(rpc_id) = p2p_state.peer_connection_rpc_id(peer_id) {
                    if let Some(callback) =
                        &p2p_state.callbacks.on_p2p_connection_incoming_answer_ready
                    {
                        dispatcher.push_callback(
                            callback.clone(),
                            (
                                rpc_id,
                                *peer_id,
                                P2pConnectionResponse::Accepted(answer.clone()),
                            ),
                        );
                    }
                }

                Ok(())
            }
            P2pConnectionIncomingAction::AnswerSendSuccess { .. } => {
                let state = p2p_state
                    .incoming_peer_connection_mut(&peer_id)
                    .ok_or_else(|| format!("Invalid state for: {:?}", action))?;
                if let Self::AnswerReady {
                    signaling,
                    offer,
                    answer,
                    rpc_id,
                    ..
                } = state
                {
                    *state = Self::AnswerSendSuccess {
                        time: meta.time(),
                        signaling: signaling.clone(),
                        offer: offer.clone(),
                        answer: answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                } else {
                    bug_condition!(
                        "Invalid state for `P2pConnectionIncomingAction::AnswerSendSuccess`: {:?}",
                        state
                    );
                    return Ok(());
                }

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pConnectionIncomingAction::FinalizePending { peer_id });
                Ok(())
            }
            P2pConnectionIncomingAction::FinalizePending { .. } => {
                let state = p2p_state
                    .incoming_peer_connection_mut(&peer_id)
                    .ok_or_else(|| format!("Invalid state for: {:?}", action))?;
                if let Self::AnswerSendSuccess {
                    signaling,
                    offer,
                    answer,
                    rpc_id,
                    ..
                } = state
                {
                    *state = Self::FinalizePending {
                        time: meta.time(),
                        signaling: signaling.clone(),
                        offer: offer.clone(),
                        answer: answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                } else {
                    bug_condition!(
                        "Invalid state for `P2pConnectionIncomingAction::FinalizePending`: {:?}",
                        state
                    );
                }
                Ok(())
            }
            P2pConnectionIncomingAction::FinalizeError { error, .. } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pConnectionIncomingAction::Error {
                    peer_id,
                    error: P2pConnectionIncomingError::FinalizeError(error.to_owned()),
                });
                Ok(())
            }
            P2pConnectionIncomingAction::FinalizeSuccess { .. } => {
                let state = p2p_state
                    .incoming_peer_connection_mut(&peer_id)
                    .ok_or_else(|| format!("Invalid state for: {:?}", action))?;
                if let Self::FinalizePending {
                    signaling,
                    offer,
                    answer,
                    rpc_id,
                    ..
                } = state
                {
                    *state = Self::FinalizeSuccess {
                        time: meta.time(),
                        signaling: signaling.clone(),
                        offer: offer.clone(),
                        answer: answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                } else {
                    bug_condition!(
                        "Invalid state for `P2pConnectionIncomingAction::FinalizeSuccess`: {:?}",
                        state
                    );
                    return Ok(());
                }

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pConnectionIncomingAction::Success { peer_id });
                Ok(())
            }
            P2pConnectionIncomingAction::Timeout { .. } => {
                let (dispatcher, _state) = state_context.into_dispatcher_and_state();

                #[cfg(feature = "p2p-libp2p")]
                {
                    let p2p_state: &P2pState = _state.substate()?;
                    if let Some((addr, _)) = p2p_state
                        .network
                        .scheduler
                        .connections
                        .iter()
                        .find(|(_, state)| state.peer_id().is_some_and(|id| *id == peer_id))
                    {
                        dispatcher.push(P2pNetworkSchedulerAction::Disconnect {
                            addr: *addr,
                            reason: P2pDisconnectionReason::Timeout,
                        });
                    }
                }

                dispatcher.push(P2pConnectionIncomingAction::Error {
                    peer_id,
                    error: P2pConnectionIncomingError::Timeout,
                });

                Ok(())
            }
            P2pConnectionIncomingAction::Error { error, .. } => {
                let state = p2p_state
                    .incoming_peer_connection_mut(&peer_id)
                    .ok_or_else(|| format!("Invalid state for: {:?}", action))?;

                let rpc_id = state.rpc_id();
                *state = Self::Error {
                    time: meta.time(),
                    error: error.clone(),
                    rpc_id,
                };

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;

                if let Some(rpc_id) = p2p_state.peer_connection_rpc_id(&peer_id) {
                    if let Some(callback) = &p2p_state.callbacks.on_p2p_connection_incoming_error {
                        dispatcher
                            .push_callback(callback.clone(), (rpc_id, format!("{:?}", error)));
                    }
                }

                Ok(())
            }
            P2pConnectionIncomingAction::Success { .. } => {
                let state = p2p_state
                    .incoming_peer_connection_mut(&peer_id)
                    .ok_or_else(|| format!("Invalid state for: {:?}", action))?;

                if let Self::FinalizeSuccess {
                    signaling,
                    offer,
                    answer,
                    rpc_id,
                    ..
                } = state
                {
                    *state = Self::Success {
                        time: meta.time(),
                        signaling: signaling.clone(),
                        offer: offer.clone(),
                        answer: answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                } else {
                    bug_condition!(
                        "Invalid state for `P2pConnectionIncomingAction::Success`: {:?}",
                        state
                    );
                    return Ok(());
                }

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;

                dispatcher.push(P2pPeerAction::Ready {
                    peer_id,
                    incoming: true,
                });

                if let Some(rpc_id) = p2p_state.peer_connection_rpc_id(&peer_id) {
                    if let Some(callback) = &p2p_state.callbacks.on_p2p_connection_incoming_success
                    {
                        dispatcher.push_callback(callback.clone(), rpc_id);
                    }
                }
                Ok(())
            }
            P2pConnectionIncomingAction::FinalizePendingLibp2p { addr, .. } => {
                #[cfg(feature = "p2p-libp2p")]
                {
                    let state = p2p_state
                        .peers
                        .entry(peer_id)
                        .or_insert_with(|| P2pPeerState {
                            is_libp2p: true,
                            dial_opts: None,
                            status: P2pPeerStatus::Disconnected { time: meta.time() },
                            identify: None,
                        });

                    Self::reduce_finalize_libp2p_pending(state, *addr, time, my_id, peer_id);

                    let (dispatcher, state) = state_context.into_dispatcher_and_state();
                    let p2p_state: &P2pState = state.substate()?;
                    Self::dispatch_finalize_libp2p_pending(
                        dispatcher, p2p_state, my_id, peer_id, time, addr,
                    );
                }

                Ok(())
            }
            P2pConnectionIncomingAction::Libp2pReceived { .. } => {
                #[cfg(feature = "p2p-libp2p")]
                {
                    let state = p2p_state
                        .incoming_peer_connection_mut(&peer_id)
                        .ok_or_else(|| format!("Invalid state for: {:?}", action))?;

                    if let Self::FinalizePendingLibp2p { time, .. } = state {
                        *state = Self::Libp2pReceived { time: *time };
                    } else {
                        bug_condition!(
                            "Invalid state for `P2pConnectionIncomingAction::Libp2pReceived`: {:?}",
                            state
                        );
                        return Ok(());
                    }

                    let dispatcher = state_context.into_dispatcher();
                    dispatcher.push(P2pPeerAction::Ready {
                        peer_id,
                        incoming: true,
                    });
                }
                Ok(())
            }
        }
    }

    #[cfg(feature = "p2p-libp2p")]
    fn dispatch_finalize_libp2p_pending<Action, State>(
        dispatcher: &mut Dispatcher<Action, State>,
        p2p_state: &P2pState,
        my_id: PeerId,
        peer_id: PeerId,
        time: Timestamp,
        addr: &SocketAddr,
    ) where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let Some(peer_state) = p2p_state.peers.get(&peer_id) else {
            bug_condition!("Peer State not found for {}", peer_id);
            return;
        };

        if let Some(P2pConnectionIncomingState::FinalizePendingLibp2p {
            close_duplicates, ..
        }) = peer_state
            .status
            .as_connecting()
            .and_then(|connecting| connecting.as_incoming())
        {
            if let Err(reason) = p2p_state.libp2p_incoming_accept(peer_id) {
                warn!(time; node_id = display(my_id), summary = "rejecting incoming connection", peer_id = display(peer_id), reason = display(&reason));
                dispatcher.push(P2pDisconnectionAction::Init {
                    peer_id,
                    reason: P2pDisconnectionReason::Libp2pIncomingRejected(reason),
                });
            } else {
                debug!(time; "accepting incoming connection from {peer_id}");
                if !close_duplicates.is_empty() {
                    let duplicates = p2p_state
                        .network
                        .scheduler
                        .connections
                        .keys()
                        .filter(
                            |ConnectionAddr {
                                 sock_addr,
                                 incoming,
                             }| {
                                *incoming
                                    && sock_addr != addr
                                    && close_duplicates.contains(sock_addr)
                            },
                        )
                        .cloned()
                        .collect::<Vec<_>>();

                    for addr in duplicates {
                        warn!(time; node_id = display(my_id), summary = "closing duplicate connection", addr = display(addr));
                        dispatcher.push(P2pNetworkSchedulerAction::Disconnect {
                            addr,
                            reason: P2pDisconnectionReason::Libp2pIncomingRejected(
                                RejectionReason::AlreadyConnected,
                            ),
                        });
                    }
                }
            }
        } else {
            warn!(time; node_id = display(my_id), summary = "rejecting incoming connection as duplicate", peer_id = display(peer_id));
            dispatcher.push(P2pNetworkSchedulerAction::Disconnect {
                addr: ConnectionAddr {
                    sock_addr: *addr,
                    incoming: true,
                },
                reason: P2pDisconnectionReason::Libp2pIncomingRejected(
                    RejectionReason::AlreadyConnected,
                ),
            });
        }
    }

    #[cfg(feature = "p2p-libp2p")]
    fn reduce_finalize_libp2p_pending(
        state: &mut P2pPeerState,
        addr: SocketAddr,
        time: Timestamp,
        my_id: PeerId,
        peer_id: PeerId,
    ) {
        let incoming_state = match &state.status {
            // No duplicate connection
            // Timeout connections should be already closed at this point
            P2pPeerStatus::Disconnected { .. }
            | P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                P2pConnectionIncomingState::Error {
                    error: P2pConnectionIncomingError::Timeout,
                    ..
                },
            )) => Some(P2pConnectionIncomingState::FinalizePendingLibp2p {
                addr,
                close_duplicates: Vec::new(),
                time,
            }),
            P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(_)) if my_id < peer_id => {
                // connection from lesser peer_id to greater one is kept in favour of the opposite one (incoming in this case)
                None
            }
            P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(_)) => {
                let mut close_duplicates = Vec::new();
                if let Some(identify) = state.identify.as_ref() {
                    close_duplicates.extend(identify.listen_addrs.iter().filter_map(|maddr| {
                        let mut iter = maddr.iter();
                        let ip: IpAddr = match iter.next()? {
                            multiaddr::Protocol::Ip4(ip4) => ip4.into(),
                            multiaddr::Protocol::Ip6(ip6) => ip6.into(),
                            _ => return None,
                        };
                        let port = match iter.next()? {
                            multiaddr::Protocol::Tcp(port) => port,
                            _ => return None,
                        };
                        Some(SocketAddr::from((ip, port)))
                    }))
                }
                if let Some(P2pConnectionOutgoingInitOpts::LibP2P(libp2p)) =
                    state.dial_opts.as_ref()
                {
                    match libp2p.try_into() {
                        Ok(addr) if !close_duplicates.contains(&addr) => {
                            close_duplicates.push(addr)
                        }
                        _ => {}
                    }
                };
                Some(P2pConnectionIncomingState::FinalizePendingLibp2p {
                    addr,
                    close_duplicates,
                    time,
                })
            }
            _ => None,
        };
        if let Some(incoming_state) = incoming_state {
            state.status = P2pPeerStatus::Connecting(P2pConnectionState::Incoming(incoming_state));
        }
    }
}
