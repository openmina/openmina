use std::net::SocketAddr;

use openmina_core::{bug_condition, warn, Substate};
use redux::ActionWithMeta;

use crate::{
    channels::signaling::discovery::P2pChannelsSignalingDiscoveryAction,
    connection::{
        outgoing_effectful::P2pConnectionOutgoingEffectfulAction, P2pConnectionErrorResponse,
        P2pConnectionState,
    },
    webrtc::Host,
    P2pNetworkKadRequestAction, P2pNetworkSchedulerAction, P2pPeerAction, P2pPeerState,
    P2pPeerStatus, P2pState,
};

use super::{
    libp2p_opts::P2pConnectionOutgoingInitLibp2pOptsTryToSocketAddrError,
    P2pConnectionOutgoingAction, P2pConnectionOutgoingError, P2pConnectionOutgoingInitOpts,
    P2pConnectionOutgoingState,
};

impl P2pConnectionOutgoingState {
    /// Substate is accessed
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, P2pState>,
        action: ActionWithMeta<P2pConnectionOutgoingAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();
        let time = meta.time();
        let p2p_state = state_context.get_substate_mut()?;

        match action {
            P2pConnectionOutgoingAction::RandomInit => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pConnectionOutgoingEffectfulAction::RandomInit);
                Ok(())
            }
            P2pConnectionOutgoingAction::Init { opts, rpc_id } => {
                let peer_state =
                    p2p_state
                        .peers
                        .entry(*opts.peer_id())
                        .or_insert_with(|| P2pPeerState {
                            is_libp2p: opts.is_libp2p(),
                            dial_opts: Some(opts.clone()).filter(|v| v.can_connect_directly()),
                            status: P2pPeerStatus::Connecting(P2pConnectionState::outgoing_init(
                                &opts,
                            )),
                            identify: None,
                        });

                peer_state.status =
                    P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(Self::Init {
                        time,
                        opts: opts.clone(),
                        rpc_id,
                    }));

                let dispatcher = state_context.into_dispatcher();

                #[cfg(feature = "p2p-libp2p")]
                if let P2pConnectionOutgoingInitOpts::LibP2P(libp2p_opts) = &opts {
                    match SocketAddr::try_from(libp2p_opts) {
                        Ok(addr) => {
                            dispatcher.push(P2pNetworkSchedulerAction::OutgoingConnect { addr });
                        }
                        Err(
                            P2pConnectionOutgoingInitLibp2pOptsTryToSocketAddrError::Unresolved(
                                _name,
                            ),
                        ) => {
                            // TODO: initiate name resolution
                            warn!(meta.time(); "name resolution needed to connect to {}", opts);
                        }
                    }
                    dispatcher.push(P2pConnectionOutgoingAction::FinalizePending {
                        peer_id: libp2p_opts.peer_id,
                    });
                    return Ok(());
                }

                dispatcher.push(P2pConnectionOutgoingEffectfulAction::Init { opts, rpc_id });
                Ok(())
            }
            P2pConnectionOutgoingAction::Reconnect { opts, rpc_id } => {
                let peer_state = p2p_state
                    .peers
                    .get_mut(opts.peer_id())
                    .ok_or("Missing peer state for: `P2pConnectionOutgoingAction::Reconnect`")?;

                peer_state.status =
                    P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(Self::Init {
                        time,
                        opts: opts.clone(),
                        rpc_id,
                    }));

                let dispatcher = state_context.into_dispatcher();

                #[cfg(feature = "p2p-libp2p")]
                if let P2pConnectionOutgoingInitOpts::LibP2P(libp2p_opts) = &opts {
                    match SocketAddr::try_from(libp2p_opts) {
                        Ok(addr) => {
                            dispatcher.push(P2pNetworkSchedulerAction::OutgoingConnect { addr });
                        }
                        Err(
                            P2pConnectionOutgoingInitLibp2pOptsTryToSocketAddrError::Unresolved(
                                _name,
                            ),
                        ) => {
                            // TODO: initiate name resolution
                            warn!(meta.time(); "name resolution needed to connect to {}", opts);
                        }
                    }
                    dispatcher.push(P2pConnectionOutgoingAction::FinalizePending {
                        peer_id: *opts.peer_id(),
                    });
                    return Ok(());
                }

                dispatcher.push(P2pConnectionOutgoingEffectfulAction::Init { opts, rpc_id });
                Ok(())
            }
            P2pConnectionOutgoingAction::OfferSdpCreatePending { peer_id, .. } => {
                let state = p2p_state
                    .outgoing_peer_connection_mut(&peer_id)
                    .ok_or("Missing connection state for: `P2pConnectionOutgoingAction::OfferSdpCreatePending`")?;

                if let Self::Init { opts, rpc_id, .. } = state {
                    *state = Self::OfferSdpCreatePending {
                        time,
                        opts: opts.clone(),
                        rpc_id: rpc_id.take(),
                    };
                } else {
                    bug_condition!("Invalid state for `P2pConnectionOutgoingAction::OfferSdpCreatePending`: {:?}", state);
                }

                Ok(())
            }
            P2pConnectionOutgoingAction::OfferSdpCreateError { error, peer_id, .. } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pConnectionOutgoingAction::Error {
                    peer_id,
                    error: P2pConnectionOutgoingError::SdpCreateError(error.to_owned()),
                });
                Ok(())
            }
            P2pConnectionOutgoingAction::OfferSdpCreateSuccess { sdp, peer_id } => {
                let state = p2p_state
                    .outgoing_peer_connection_mut(&peer_id)
                    .ok_or("Missing peer connection for `P2pConnectionOutgoingAction::OfferSdpCreateSuccess`")?;

                if let Self::OfferSdpCreatePending { opts, rpc_id, .. } = state {
                    *state = Self::OfferSdpCreateSuccess {
                        time,
                        opts: opts.clone(),
                        sdp: sdp.clone(),
                        rpc_id: rpc_id.take(),
                    };
                } else {
                    bug_condition!("Invalid state for `P2pConnectionOutgoingAction::OfferSdpCreateSuccess`: {:?}", state);
                    return Ok(());
                }

                let offer = Box::new(crate::webrtc::Offer {
                    sdp,
                    identity_pub_key: p2p_state.config.identity_pub_key.clone(),
                    target_peer_id: peer_id,
                    // TODO(vlad9486): put real address
                    host: Host::Ipv4([127, 0, 0, 1].into()),
                    listen_port: p2p_state.config.listen_port,
                });
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pConnectionOutgoingAction::OfferReady { peer_id, offer });
                Ok(())
            }
            P2pConnectionOutgoingAction::OfferReady { offer, peer_id } => {
                let state = p2p_state
                    .outgoing_peer_connection_mut(&peer_id)
                    .ok_or("Invalid state for `P2pConnectionOutgoingAction::OfferReady`")?;

                let Self::OfferSdpCreateSuccess { opts, rpc_id, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pConnectionOutgoingAction::OfferReady`: {:?}",
                        state
                    );
                    return Ok(());
                };
                let opts = opts.clone();
                *state = Self::OfferReady {
                    time: meta.time(),
                    opts: opts.clone(),
                    offer: offer.clone(),
                    rpc_id: rpc_id.take(),
                };

                let dispatcher = state_context.into_dispatcher();

                if let Some(relay_peer_id) = opts.webrtc_p2p_relay_peer_id() {
                    dispatcher.push(P2pChannelsSignalingDiscoveryAction::DiscoveredAccept {
                        peer_id: relay_peer_id,
                        offer: offer.clone(),
                    });
                } else {
                    dispatcher.push(P2pConnectionOutgoingEffectfulAction::OfferSend {
                        peer_id,
                        offer: offer.clone(),
                    });
                }
                Ok(())
            }
            P2pConnectionOutgoingAction::OfferSendSuccess { peer_id } => {
                let state = p2p_state
                    .outgoing_peer_connection_mut(&peer_id)
                    .ok_or_else(|| format!("Invalid state: {:?}", action))?;
                if let Self::OfferReady {
                    opts,
                    offer,
                    rpc_id,
                    ..
                } = state
                {
                    *state = Self::OfferSendSuccess {
                        time,
                        opts: opts.clone(),
                        offer: offer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                } else {
                    bug_condition!(
                        "Invalid state for `P2pConnectionOutgoingAction::OfferSendSuccess`: {:?}",
                        state
                    );
                    return Ok(());
                }

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pConnectionOutgoingAction::AnswerRecvPending { peer_id });
                Ok(())
            }
            P2pConnectionOutgoingAction::AnswerRecvPending { peer_id } => {
                let state = p2p_state
                    .outgoing_peer_connection_mut(&peer_id)
                    .ok_or_else(|| format!("Invalid state: {:?}", action))?;
                if let Self::OfferSendSuccess {
                    opts,
                    offer,
                    rpc_id,
                    ..
                } = state
                {
                    *state = Self::AnswerRecvPending {
                        time,
                        opts: opts.clone(),
                        offer: offer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                } else {
                    bug_condition!(
                        "Invalid state for `P2pConnectionOutgoingAction::AnswerRecvPending`: {:?}",
                        state
                    );
                }
                Ok(())
            }
            P2pConnectionOutgoingAction::AnswerRecvError { error, peer_id } => {
                let dispatcher = state_context.into_dispatcher();

                dispatcher.push(P2pConnectionOutgoingAction::Error {
                    peer_id,
                    error: match error {
                        P2pConnectionErrorResponse::Rejected(reason) => {
                            P2pConnectionOutgoingError::Rejected(reason)
                        }
                        P2pConnectionErrorResponse::SignalDecryptionFailed => {
                            P2pConnectionOutgoingError::RemoteSignalDecryptionFailed
                        }
                        P2pConnectionErrorResponse::InternalError => {
                            P2pConnectionOutgoingError::RemoteInternalError
                        }
                    },
                });
                Ok(())
            }
            P2pConnectionOutgoingAction::AnswerRecvSuccess { answer, peer_id } => {
                let state = p2p_state.outgoing_peer_connection_mut(&peer_id).ok_or(
                    "Missing peer connection for `P2pConnectionOutgoingAction::AnswerRecvSuccess`",
                )?;

                if let Self::AnswerRecvPending {
                    opts,
                    offer,
                    rpc_id,
                    ..
                } = state
                {
                    *state = Self::AnswerRecvSuccess {
                        time,
                        opts: opts.clone(),
                        offer: offer.clone(),
                        answer: answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                } else {
                    bug_condition!(
                        "Invalid state for `P2pConnectionOutgoingAction::AnswerRecvSuccess`: {:?}",
                        state
                    );
                }
                state_context
                    .into_dispatcher()
                    .push(P2pConnectionOutgoingEffectfulAction::AnswerSet { peer_id, answer });
                Ok(())
            }
            P2pConnectionOutgoingAction::FinalizePending { peer_id } => {
                let state = p2p_state
                    .outgoing_peer_connection_mut(&peer_id)
                    .ok_or_else(|| format!("Invalid state: {:?}", action))?;

                let (auth, other_pub_key) = match state {
                    Self::Init { opts, rpc_id, .. } => {
                        *state = Self::FinalizePending {
                            time,
                            opts: opts.clone(),
                            offer: None,
                            answer: None,
                            rpc_id: rpc_id.take(),
                        };
                        return Ok(());
                    }
                    Self::AnswerRecvSuccess {
                        opts,
                        offer,
                        answer,
                        rpc_id,
                        ..
                    } => {
                        let auth = offer.conn_auth(answer);
                        let other_pub_key = answer.identity_pub_key.clone();

                        *state = Self::FinalizePending {
                            time,
                            opts: opts.clone(),
                            offer: Some(offer.clone()),
                            answer: Some(answer.clone()),
                            rpc_id: rpc_id.take(),
                        };

                        (auth, other_pub_key)
                    }
                    _ => {
                        bug_condition!("Invalid state for `P2pConnectionOutgoingAction::FinalizePending`: {state:?}");
                        return Ok(());
                    }
                };

                state_context.into_dispatcher().push(
                    P2pConnectionOutgoingEffectfulAction::ConnectionAuthorizationEncryptAndSend {
                        peer_id,
                        other_pub_key,
                        auth,
                    },
                );
                Ok(())
            }
            P2pConnectionOutgoingAction::FinalizeError { error, peer_id } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pConnectionOutgoingAction::Error {
                    peer_id,
                    error: P2pConnectionOutgoingError::FinalizeError(error.to_owned()),
                });
                Ok(())
            }
            P2pConnectionOutgoingAction::FinalizeSuccess {
                peer_id,
                remote_auth: auth,
            } => {
                let state = p2p_state
                    .outgoing_peer_connection_mut(&peer_id)
                    .ok_or_else(|| {
                        "Invalid state for: P2pConnectionOutgoingAction::FinalizeSuccess".to_owned()
                    })?;

                let values = if let Self::FinalizePending {
                    opts,
                    offer,
                    answer,
                    rpc_id,
                    ..
                } = state
                {
                    let values = None.or_else(|| {
                        let answer = answer.as_ref()?;
                        Some((
                            auth?,
                            offer.as_ref()?.conn_auth(answer),
                            answer.identity_pub_key.clone(),
                        ))
                    });
                    *state = Self::FinalizeSuccess {
                        time,
                        opts: opts.clone(),
                        offer: offer.clone(),
                        answer: answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                    values
                } else {
                    bug_condition!(
                        "Invalid state for `P2pConnectionOutgoingAction::FinalizeSuccess`: {:?}",
                        state
                    );
                    return Ok(());
                };

                let dispatcher = state_context.into_dispatcher();
                if let Some((auth, expected_auth, other_pub_key)) = values {
                    dispatcher.push(
                        P2pConnectionOutgoingEffectfulAction::ConnectionAuthorizationDecryptAndCheck {
                            peer_id,
                            other_pub_key,
                            expected_auth,
                            auth,
                        },
                    );
                } else {
                    // libp2p
                    dispatcher.push(P2pConnectionOutgoingAction::Success { peer_id });
                }
                Ok(())
            }
            P2pConnectionOutgoingAction::Timeout { peer_id } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pConnectionOutgoingAction::Error {
                    peer_id,
                    error: P2pConnectionOutgoingError::Timeout,
                });
                Ok(())
            }
            P2pConnectionOutgoingAction::Error { error, peer_id } => {
                let state = p2p_state
                    .outgoing_peer_connection_mut(&peer_id)
                    .ok_or("Missing peer connection for `P2pConnectionOutgoingAction::Error`")?;

                let rpc_id = state.rpc_id();
                *state = Self::Error {
                    time,
                    error: error.clone(),
                    rpc_id,
                };

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;

                #[cfg(feature = "p2p-libp2p")]
                {
                    if p2p_state
                        .network
                        .scheduler
                        .discovery_state()
                        .and_then(|discovery_state| discovery_state.request(&peer_id))
                        .is_some()
                    {
                        dispatcher.push(P2pNetworkKadRequestAction::Error {
                            peer_id,
                            error: error.to_string(),
                        });
                    }
                }

                if let Some(rpc_id) = p2p_state.peer_connection_rpc_id(&peer_id) {
                    if let Some(callback) = &p2p_state.callbacks.on_p2p_connection_outgoing_error {
                        dispatcher.push_callback(callback.clone(), (rpc_id, error));
                    }
                }
                Ok(())
            }
            P2pConnectionOutgoingAction::Success { peer_id } => {
                let state = p2p_state
                    .outgoing_peer_connection_mut(&peer_id)
                    .ok_or_else(|| format!("Invalid state: {:?}", action))?;

                if let Self::FinalizeSuccess {
                    offer,
                    answer,
                    rpc_id,
                    ..
                } = state
                {
                    *state = Self::Success {
                        time,
                        offer: offer.clone(),
                        answer: answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                } else {
                    bug_condition!(
                        "Invalid state for `P2pConnectionOutgoingAction::Success`: {:?}",
                        state
                    );
                    return Ok(());
                }

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;
                dispatcher.push(P2pPeerAction::Ready {
                    peer_id,
                    incoming: false,
                });

                if let Some(rpc_id) = p2p_state.peer_connection_rpc_id(&peer_id) {
                    if let Some(callback) = &p2p_state.callbacks.on_p2p_connection_outgoing_success
                    {
                        dispatcher.push_callback(callback.clone(), rpc_id);
                    }
                }
                Ok(())
            }
        }
    }
}
