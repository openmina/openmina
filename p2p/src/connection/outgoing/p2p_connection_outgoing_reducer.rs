use std::net::SocketAddr;

use openmina_core::{bug_condition, warn, Substate};

use crate::{
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
    P2pConnectionOutgoingAction, P2pConnectionOutgoingActionWithMetaRef,
    P2pConnectionOutgoingError, P2pConnectionOutgoingInitOpts, P2pConnectionOutgoingState,
};

impl P2pConnectionOutgoingState {
    /// Substate is accessed
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, P2pState>,
        action: P2pConnectionOutgoingActionWithMetaRef<'_>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();
        let p2p_state = state_context.get_substate_mut()?;
        let peer_id = *action.peer_id();

        match action {
            P2pConnectionOutgoingAction::Init { opts, rpc_id } => {
                let peer_state = p2p_state
                    .peers
                    .entry(peer_id)
                    .or_insert_with(|| P2pPeerState {
                        is_libp2p: opts.is_libp2p(),
                        dial_opts: Some(opts.clone()),
                        status: P2pPeerStatus::Connecting(P2pConnectionState::outgoing_init(opts)),
                        identify: None,
                    });

                peer_state.status =
                    P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(Self::Init {
                        time: meta.time(),
                        opts: opts.clone(),
                        rpc_id: *rpc_id,
                    }));

                let dispatcher = state_context.into_dispatcher();

                #[cfg(feature = "p2p-libp2p")]
                if let P2pConnectionOutgoingInitOpts::LibP2P(libp2p_opts) = opts {
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
                    dispatcher.push(P2pConnectionOutgoingAction::FinalizePending { peer_id });
                    return Ok(());
                }

                dispatcher.push(P2pConnectionOutgoingEffectfulAction::Init {
                    opts: opts.clone(),
                    rpc_id: *rpc_id,
                });
                Ok(())
            }
            P2pConnectionOutgoingAction::Reconnect { opts, rpc_id } => {
                let peer_state = p2p_state
                    .peers
                    .get_mut(&peer_id)
                    .ok_or_else(|| format!("Invalid state: {:?}", action))?;

                peer_state.status =
                    P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(Self::Init {
                        time: meta.time(),
                        opts: opts.clone(),
                        rpc_id: *rpc_id,
                    }));

                let dispatcher = state_context.into_dispatcher();

                #[cfg(feature = "p2p-libp2p")]
                if let P2pConnectionOutgoingInitOpts::LibP2P(libp2p_opts) = opts {
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
                    dispatcher.push(P2pConnectionOutgoingAction::FinalizePending { peer_id });
                    return Ok(());
                }

                dispatcher.push(P2pConnectionOutgoingEffectfulAction::Init {
                    opts: opts.clone(),
                    rpc_id: *rpc_id,
                });
                Ok(())
            }
            P2pConnectionOutgoingAction::OfferSdpCreatePending { .. } => {
                let state = p2p_state
                    .outgoing_peer_connection_mut(&peer_id)
                    .ok_or_else(|| format!("Invalid state: {:?}", action))?;

                if let Self::Init { opts, rpc_id, .. } = state {
                    *state = Self::OfferSdpCreatePending {
                        time: meta.time(),
                        opts: opts.clone(),
                        rpc_id: rpc_id.take(),
                    };
                } else {
                    bug_condition!("Invalid state for `P2pConnectionOutgoingAction::OfferSdpCreatePending`: {:?}", state);
                }

                Ok(())
            }
            P2pConnectionOutgoingAction::OfferSdpCreateError { error, .. } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pConnectionOutgoingAction::Error {
                    peer_id,
                    error: P2pConnectionOutgoingError::SdpCreateError(error.to_owned()),
                });
                Ok(())
            }
            P2pConnectionOutgoingAction::OfferSdpCreateSuccess { sdp, .. } => {
                let state = p2p_state
                    .outgoing_peer_connection_mut(&peer_id)
                    .ok_or_else(|| format!("Invalid state: {:?}", action))?;

                if let Self::OfferSdpCreatePending { opts, rpc_id, .. } = state {
                    *state = Self::OfferSdpCreateSuccess {
                        time: meta.time(),
                        opts: opts.clone(),
                        sdp: sdp.clone(),
                        rpc_id: rpc_id.take(),
                    };
                } else {
                    bug_condition!("Invalid state for `P2pConnectionOutgoingAction::OfferSdpCreateSuccess`: {:?}", state);
                    return Ok(());
                }

                let offer = Box::new(crate::webrtc::Offer {
                    sdp: sdp.to_owned(),
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
            P2pConnectionOutgoingAction::OfferReady { offer, .. } => {
                let state = p2p_state
                    .outgoing_peer_connection_mut(&peer_id)
                    .ok_or_else(|| format!("Invalid state: {:?}", action))?;
                if let Self::OfferSdpCreateSuccess { opts, rpc_id, .. } = state {
                    *state = Self::OfferReady {
                        time: meta.time(),
                        opts: opts.clone(),
                        offer: offer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                } else {
                    bug_condition!(
                        "Invalid state for `P2pConnectionOutgoingAction::OfferReady`: {:?}",
                        state
                    );
                    return Ok(());
                }

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pConnectionOutgoingEffectfulAction::OfferReady {
                    peer_id,
                    offer: offer.clone(),
                });
                Ok(())
            }
            P2pConnectionOutgoingAction::OfferSendSuccess { .. } => {
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
                        time: meta.time(),
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
            P2pConnectionOutgoingAction::AnswerRecvPending { .. } => {
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
                        time: meta.time(),
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
            P2pConnectionOutgoingAction::AnswerRecvError { error, .. } => {
                let dispatcher = state_context.into_dispatcher();

                dispatcher.push(P2pConnectionOutgoingAction::Error {
                    peer_id,
                    error: match error {
                        P2pConnectionErrorResponse::Rejected(reason) => {
                            P2pConnectionOutgoingError::Rejected(*reason)
                        }
                        P2pConnectionErrorResponse::InternalError => {
                            P2pConnectionOutgoingError::RemoteInternalError
                        }
                    },
                });
                Ok(())
            }
            P2pConnectionOutgoingAction::AnswerRecvSuccess { answer, .. } => {
                let state = p2p_state
                    .outgoing_peer_connection_mut(&peer_id)
                    .ok_or_else(|| format!("Invalid state: {:?}", action))?;

                if let Self::AnswerRecvPending {
                    opts,
                    offer,
                    rpc_id,
                    ..
                } = state
                {
                    *state = Self::AnswerRecvSuccess {
                        time: meta.time(),
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
                Ok(())
            }
            P2pConnectionOutgoingAction::FinalizePending { .. } => {
                let state = p2p_state
                    .outgoing_peer_connection_mut(&peer_id)
                    .ok_or_else(|| format!("Invalid state: {:?}", action))?;

                match state {
                    Self::Init { opts, rpc_id, .. } => {
                        *state = Self::FinalizePending {
                            time: meta.time(),
                            opts: opts.clone(),
                            offer: None,
                            answer: None,
                            rpc_id: rpc_id.take(),
                        };
                        Ok(())
                    }
                    Self::AnswerRecvSuccess {
                        opts,
                        offer,
                        answer,
                        rpc_id,
                        ..
                    } => {
                        *state = Self::FinalizePending {
                            time: meta.time(),
                            opts: opts.clone(),
                            offer: Some(offer.clone()),
                            answer: Some(answer.clone()),
                            rpc_id: rpc_id.take(),
                        };
                        Ok(())
                    }
                    _ => {
                        bug_condition!(
                                "Invalid state for `P2pConnectionOutgoingAction::FinalizePending`: {:?}",
                                state
                            );
                        Ok(())
                    }
                }
            }
            P2pConnectionOutgoingAction::FinalizeError { error, .. } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pConnectionOutgoingAction::Error {
                    peer_id,
                    error: P2pConnectionOutgoingError::FinalizeError(error.to_owned()),
                });
                Ok(())
            }
            P2pConnectionOutgoingAction::FinalizeSuccess { .. } => {
                let state = p2p_state
                    .outgoing_peer_connection_mut(&peer_id)
                    .ok_or_else(|| format!("Invalid state: {:?}", action))?;

                if let Self::FinalizePending {
                    opts,
                    offer,
                    answer,
                    rpc_id,
                    ..
                } = state
                {
                    *state = Self::FinalizeSuccess {
                        time: meta.time(),
                        opts: opts.clone(),
                        offer: offer.clone(),
                        answer: answer.clone(),
                        rpc_id: rpc_id.take(),
                    };
                } else {
                    bug_condition!(
                        "Invalid state for `P2pConnectionOutgoingAction::FinalizeSuccess`: {:?}",
                        state
                    );
                    return Ok(());
                }

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pConnectionOutgoingAction::Success { peer_id });
                Ok(())
            }
            P2pConnectionOutgoingAction::Timeout { .. } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pConnectionOutgoingAction::Error {
                    peer_id,
                    error: P2pConnectionOutgoingError::Timeout,
                });
                Ok(())
            }
            P2pConnectionOutgoingAction::Error { error, .. } => {
                let state = p2p_state
                    .outgoing_peer_connection_mut(&peer_id)
                    .ok_or_else(|| format!("Invalid state: {:?}", action))?;

                let rpc_id = state.rpc_id();
                *state = Self::Error {
                    time: meta.time(),
                    error: error.clone(),
                    rpc_id,
                };

                #[cfg(feature = "p2p-libp2p")]
                {
                    let (dispatcher, state) = state_context.into_dispatcher_and_state();
                    let p2p_state: &P2pState = state.substate()?;

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
                Ok(())
            }
            P2pConnectionOutgoingAction::Success { .. } => {
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
                        time: meta.time(),
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

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pPeerAction::Ready {
                    peer_id,
                    incoming: false,
                });
                Ok(())
            }
        }
    }
}
