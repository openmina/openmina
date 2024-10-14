use openmina_core::{bug_condition, Substate};
use redux::ActionWithMeta;

use crate::{
    channels::signaling::{
        discovery_effectful::P2pChannelsSignalingDiscoveryEffectfulAction,
        exchange::P2pChannelsSignalingExchangeAction,
    },
    connection::{
        outgoing::{P2pConnectionOutgoingAction, P2pConnectionOutgoingInitOpts},
        P2pConnectionErrorResponse, P2pConnectionResponse,
    },
    webrtc::SignalingMethod,
    P2pState,
};

use super::{
    P2pChannelsSignalingDiscoveryAction, P2pChannelsSignalingDiscoveryState,
    SignalingDiscoveryChannelMsg, SignalingDiscoveryState,
};

impl P2pChannelsSignalingDiscoveryState {
    /// Substate is accessed
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, P2pState>,
        action: ActionWithMeta<&P2pChannelsSignalingDiscoveryAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();
        let p2p_state = state_context.get_substate_mut()?;
        let peer_id = *action.peer_id();
        let state = &mut p2p_state
            .get_ready_peer_mut(&peer_id)
            .ok_or_else(|| format!("Peer state not found for: {action:?}"))?
            .channels
            .signaling
            .discovery;

        match action {
            P2pChannelsSignalingDiscoveryAction::Init { .. } => {
                *state = Self::Init { time: meta.time() };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsSignalingDiscoveryEffectfulAction::Init { peer_id });
                Ok(())
            }
            P2pChannelsSignalingDiscoveryAction::Pending { .. } => {
                *state = Self::Pending { time: meta.time() };
                Ok(())
            }
            P2pChannelsSignalingDiscoveryAction::Ready { .. } => {
                *state = Self::Ready {
                    time: meta.time(),
                    local: SignalingDiscoveryState::WaitingForRequest { time: meta.time() },
                    remote: SignalingDiscoveryState::WaitingForRequest { time: meta.time() },
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsSignalingDiscoveryAction::RequestSend { peer_id });

                Ok(())
            }
            P2pChannelsSignalingDiscoveryAction::RequestSend { .. } => {
                let Self::Ready { local, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSignalingDiscoveryAction::RequestSend`, state: {state:?}",
                    );
                    return Ok(());
                };
                *local = SignalingDiscoveryState::Requested { time: meta.time() };

                let dispatcher = state_context.into_dispatcher();
                let message = SignalingDiscoveryChannelMsg::GetNext;
                dispatcher.push(P2pChannelsSignalingDiscoveryEffectfulAction::MessageSend {
                    peer_id,
                    message,
                });
                Ok(())
            }
            P2pChannelsSignalingDiscoveryAction::DiscoveryRequestReceived { .. } => {
                let Self::Ready { local, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSignalingDiscoveryAction::OfferDecryptError`, state: {state:?}",
                    );
                    return Ok(());
                };

                *local = SignalingDiscoveryState::DiscoveryRequested { time: meta.time() };

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let state: &P2pState = state.substate()?;
                state.webrtc_discovery_respond_with_availble_peers(dispatcher);

                Ok(())
            }
            P2pChannelsSignalingDiscoveryAction::DiscoveredSend {
                target_public_key, ..
            } => {
                let Self::Ready { local, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSignalingDiscoveryAction::OfferDecryptError`, state: {state:?}",
                    );
                    return Ok(());
                };

                *local = SignalingDiscoveryState::Discovered {
                    time: meta.time(),
                    target_public_key: target_public_key.clone(),
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsSignalingDiscoveryEffectfulAction::MessageSend {
                    peer_id,
                    message: SignalingDiscoveryChannelMsg::Discovered {
                        target_public_key: target_public_key.clone(),
                    },
                });
                Ok(())
            }
            P2pChannelsSignalingDiscoveryAction::DiscoveredRejectReceived { .. } => {
                let Self::Ready { local, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSignalingDiscoveryAction::OfferDecryptError`, state: {state:?}",
                    );
                    return Ok(());
                };

                let target_public_key = match local {
                    SignalingDiscoveryState::Discovered {
                        target_public_key, ..
                    } => target_public_key.clone(),
                    state => {
                        bug_condition!(
                            "Invalid local state for `P2pChannelsSignalingDiscoveryAction::OfferDecryptError`, state: {state:?}",
                        );
                        return Ok(());
                    }
                };

                *local = SignalingDiscoveryState::DiscoveredRejected {
                    time: meta.time(),
                    target_public_key,
                };

                Ok(())
            }
            P2pChannelsSignalingDiscoveryAction::DiscoveredAcceptReceived { offer, .. } => {
                let Self::Ready { local, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSignalingDiscoveryAction::OfferDecryptError`, state: {state:?}",
                    );
                    return Ok(());
                };

                let target_public_key = match local {
                    SignalingDiscoveryState::Discovered {
                        target_public_key, ..
                    } => target_public_key.clone(),
                    state => {
                        bug_condition!(
                            "Invalid local state for `P2pChannelsSignalingDiscoveryAction::OfferDecryptError`, state: {state:?}",
                        );
                        return Ok(());
                    }
                };

                *local = SignalingDiscoveryState::DiscoveredAccepted {
                    time: meta.time(),
                    target_public_key: target_public_key.clone(),
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsSignalingExchangeAction::OfferSend {
                    peer_id: target_public_key.peer_id(),
                    offerer_pub_key: peer_id.to_public_key().unwrap(),
                    offer: offer.clone(),
                });
                Ok(())
            }
            P2pChannelsSignalingDiscoveryAction::AnswerSend { answer, .. } => {
                let Self::Ready { local, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSignalingDiscoveryAction::OfferDecryptError`, state: {state:?}",
                    );
                    return Ok(());
                };

                *local = SignalingDiscoveryState::Answered { time: meta.time() };

                let dispatcher = state_context.into_dispatcher();
                let message = SignalingDiscoveryChannelMsg::Answer(answer.clone());
                dispatcher.push(P2pChannelsSignalingDiscoveryEffectfulAction::MessageSend {
                    peer_id,
                    message,
                });
                Ok(())
            }
            P2pChannelsSignalingDiscoveryAction::RequestReceived { .. } => {
                let Self::Ready { remote, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSignalingDiscoveryAction::RequestReceived`, state: {state:?}",
                    );
                    return Ok(());
                };

                *remote = SignalingDiscoveryState::Requested { time: meta.time() };
                Ok(())
            }
            P2pChannelsSignalingDiscoveryAction::DiscoveryRequestSend { .. } => {
                let Self::Ready { remote, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSignalingDiscoveryAction::OfferSend`, state: {state:?}",
                    );
                    return Ok(());
                };

                *remote = SignalingDiscoveryState::DiscoveryRequested { time: meta.time() };
                let dispatcher = state_context.into_dispatcher();
                let message = SignalingDiscoveryChannelMsg::Discover;
                dispatcher.push(P2pChannelsSignalingDiscoveryEffectfulAction::MessageSend {
                    peer_id,
                    message,
                });
                Ok(())
            }
            P2pChannelsSignalingDiscoveryAction::DiscoveredReceived {
                target_public_key, ..
            } => {
                let Self::Ready { remote, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSignalingDiscoveryAction::OfferSend`, state: {state:?}",
                    );
                    return Ok(());
                };

                *remote = SignalingDiscoveryState::Discovered {
                    time: meta.time(),
                    target_public_key: target_public_key.clone(),
                };
                let dispatcher = state_context.into_dispatcher();
                // TODO(binier): this action might not be enabled, in
                // which case we sshould be rejecting discovered peer.
                dispatcher.push(P2pConnectionOutgoingAction::Init {
                    opts: P2pConnectionOutgoingInitOpts::WebRTC {
                        peer_id: target_public_key.peer_id(),
                        signaling: SignalingMethod::P2p {
                            relay_peer_id: peer_id,
                        },
                    },
                    rpc_id: None,
                });
                Ok(())
            }
            P2pChannelsSignalingDiscoveryAction::DiscoveredReject { .. } => {
                todo!("handle peer rejection")
            }
            P2pChannelsSignalingDiscoveryAction::DiscoveredAccept { offer, .. } => {
                let Self::Ready { remote, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSignalingDiscoveryAction::OfferSend`, state: {state:?}",
                    );
                    return Ok(());
                };

                let target_public_key = match remote {
                    SignalingDiscoveryState::Discovered {
                        target_public_key, ..
                    } => target_public_key.clone(),
                    state => {
                        bug_condition!(
                            "Invalid state for `P2pChannelsSignalingDiscoveryAction::OfferSend`, state: {state:?}",
                        );
                        return Ok(());
                    }
                };

                *remote = SignalingDiscoveryState::DiscoveredAccepted {
                    time: meta.time(),
                    target_public_key: target_public_key.clone(),
                };
                let dispatcher = state_context.into_dispatcher();
                // TODO(binier): this action might not be enabled, in
                // which case we sshould be rejecting discovered peer.
                dispatcher.push(P2pConnectionOutgoingAction::OfferSendSuccess {
                    peer_id: target_public_key.peer_id(),
                });
                dispatcher.push(
                    P2pChannelsSignalingDiscoveryEffectfulAction::OfferEncryptAndSend {
                        peer_id,
                        pub_key: target_public_key,
                        offer: offer.clone(),
                    },
                );
                Ok(())
            }
            P2pChannelsSignalingDiscoveryAction::AnswerReceived { answer, .. } => {
                let Self::Ready { remote, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSignalingDiscoveryAction::OfferSend`, state: {state:?}",
                    );
                    return Ok(());
                };

                let target_public_key = match remote {
                    SignalingDiscoveryState::DiscoveredAccepted {
                        target_public_key, ..
                    } => target_public_key.clone(),
                    state => {
                        bug_condition!(
                        "Invalid remote state for `P2pChannelsSignalingDiscoveryAction::OfferDecryptError`, state: {state:?}",
                    );
                        return Ok(());
                    }
                };

                let dispatcher = state_context.into_dispatcher();
                match answer {
                    // TODO(binier): custom error
                    None => dispatcher.push(P2pConnectionOutgoingAction::AnswerRecvError {
                        peer_id: target_public_key.peer_id(),
                        error: P2pConnectionErrorResponse::InternalError,
                    }),
                    Some(answer) => dispatcher.push(
                        P2pChannelsSignalingDiscoveryEffectfulAction::AnswerDecrypt {
                            peer_id,
                            pub_key: target_public_key,
                            answer: answer.clone(),
                        },
                    ),
                }
                Ok(())
            }
            P2pChannelsSignalingDiscoveryAction::AnswerDecrypted { answer, .. } => {
                let Self::Ready { remote, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSignalingDiscoveryAction::OfferSend`, state: {state:?}",
                    );
                    return Ok(());
                };

                let target_public_key = match remote {
                    SignalingDiscoveryState::DiscoveredAccepted {
                        target_public_key, ..
                    } => target_public_key.clone(),
                    state => {
                        bug_condition!(
                            "Invalid remote state for `P2pChannelsSignalingDiscoveryAction::OfferDecryptError`, state: {state:?}",
                        );
                        return Ok(());
                    }
                };

                *remote = SignalingDiscoveryState::Answered { time: meta.time() };

                let dispatcher = state_context.into_dispatcher();
                match answer {
                    P2pConnectionResponse::Accepted(answer) => {
                        dispatcher.push(P2pConnectionOutgoingAction::AnswerRecvSuccess {
                            peer_id: target_public_key.peer_id(),
                            answer: answer.clone(),
                        })
                    }
                    P2pConnectionResponse::Rejected(reason) => {
                        dispatcher.push(P2pConnectionOutgoingAction::AnswerRecvError {
                            peer_id: target_public_key.peer_id(),
                            error: P2pConnectionErrorResponse::Rejected(reason.clone()),
                        })
                    }
                    P2pConnectionResponse::SignalDecryptionFailed => {
                        dispatcher.push(P2pConnectionOutgoingAction::AnswerRecvError {
                            peer_id: target_public_key.peer_id(),
                            error: P2pConnectionErrorResponse::SignalDecryptionFailed,
                        })
                    }
                    P2pConnectionResponse::InternalError => {
                        dispatcher.push(P2pConnectionOutgoingAction::AnswerRecvError {
                            peer_id: target_public_key.peer_id(),
                            error: P2pConnectionErrorResponse::InternalError,
                        })
                    }
                }
                Ok(())
            }
        }
    }
}
