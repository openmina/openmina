use openmina_core::{bug_condition, Substate};
use redux::ActionWithMeta;

use crate::{
    channels::signaling::{
        discovery::P2pChannelsSignalingDiscoveryAction,
        exchange_effectful::P2pChannelsSignalingExchangeEffectfulAction,
    },
    connection::{
        incoming::{
            IncomingSignalingMethod, P2pConnectionIncomingAction, P2pConnectionIncomingInitOpts,
        },
        P2pConnectionResponse,
    },
    P2pState,
};

use super::{
    P2pChannelsSignalingExchangeAction, P2pChannelsSignalingExchangeState,
    SignalingExchangeChannelMsg, SignalingExchangeState,
};

impl P2pChannelsSignalingExchangeState {
    /// Substate is accessed
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, P2pState>,
        action: ActionWithMeta<&P2pChannelsSignalingExchangeAction>,
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
            .exchange;

        match action {
            P2pChannelsSignalingExchangeAction::Init { .. } => {
                *state = Self::Init { time: meta.time() };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsSignalingExchangeEffectfulAction::Init { peer_id });
                Ok(())
            }
            P2pChannelsSignalingExchangeAction::Pending { .. } => {
                *state = Self::Pending { time: meta.time() };
                Ok(())
            }
            P2pChannelsSignalingExchangeAction::Ready { .. } => {
                *state = Self::Ready {
                    time: meta.time(),
                    local: SignalingExchangeState::WaitingForRequest { time: meta.time() },
                    remote: SignalingExchangeState::WaitingForRequest { time: meta.time() },
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsSignalingExchangeAction::RequestSend { peer_id });

                Ok(())
            }
            P2pChannelsSignalingExchangeAction::RequestSend { .. } => {
                let Self::Ready { local, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSignalingExchangeAction::RequestSend`, state: {state:?}",
                    );
                    return Ok(());
                };
                *local = SignalingExchangeState::Requested { time: meta.time() };

                let dispatcher = state_context.into_dispatcher();
                let message = SignalingExchangeChannelMsg::GetNext;
                dispatcher.push(P2pChannelsSignalingExchangeEffectfulAction::MessageSend {
                    peer_id,
                    message,
                });
                Ok(())
            }
            P2pChannelsSignalingExchangeAction::OfferReceived {
                offer,
                offerer_pub_key,
                ..
            } => {
                let Self::Ready { local, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSignalingExchangeAction::OfferDecryptError`, state: {state:?}",
                    );
                    return Ok(());
                };

                *local = SignalingExchangeState::Offered {
                    time: meta.time(),
                    offerer_pub_key: offerer_pub_key.clone(),
                };

                let dispatcher = state_context.into_dispatcher();
                let offer = offer.clone();
                dispatcher.push(P2pChannelsSignalingExchangeEffectfulAction::OfferDecrypt {
                    peer_id,
                    pub_key: offerer_pub_key.clone(),
                    offer,
                });
                Ok(())
            }
            P2pChannelsSignalingExchangeAction::OfferDecryptError { .. } => {
                let dispatcher = state_context.into_dispatcher();
                let answer = P2pConnectionResponse::SignalDecryptionFailed;
                dispatcher.push(P2pChannelsSignalingExchangeAction::AnswerSend { peer_id, answer });
                Ok(())
            }
            P2pChannelsSignalingExchangeAction::OfferDecryptSuccess { offer, .. } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let state: &P2pState = state.substate()?;
                let opts = P2pConnectionIncomingInitOpts {
                    peer_id: offer.identity_pub_key.peer_id(),
                    signaling: IncomingSignalingMethod::P2p {
                        relay_peer_id: peer_id,
                    },
                    offer: offer.clone().into(),
                };
                match state.incoming_accept(opts.peer_id, &opts.offer) {
                    Ok(_) => {
                        dispatcher.push(P2pConnectionIncomingAction::Init { opts, rpc_id: None });
                    }
                    Err(reason) => {
                        let answer = P2pConnectionResponse::Rejected(reason);
                        dispatcher.push(P2pChannelsSignalingExchangeAction::AnswerSend {
                            peer_id,
                            answer,
                        });
                    }
                }
                Ok(())
            }
            P2pChannelsSignalingExchangeAction::AnswerSend { answer, .. } => {
                let Self::Ready { local, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSignalingExchangeAction::AnswerSend`, state: {state:?}",
                    );
                    return Ok(());
                };

                let offerer_pub_key = match local {
                    SignalingExchangeState::Offered {
                        offerer_pub_key, ..
                    } => offerer_pub_key.clone(),
                    state => {
                        bug_condition!(
                        "Invalid state for `P2pChannelsSignalingExchangeAction::AnswerSend`, local state: {state:?}",
                    );
                        return Ok(());
                    }
                };

                *local = SignalingExchangeState::Answered { time: meta.time() };

                let answer = answer.clone();
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(
                    P2pChannelsSignalingExchangeEffectfulAction::AnswerEncryptAndSend {
                        peer_id,
                        pub_key: offerer_pub_key.clone(),
                        answer: Some(answer),
                    },
                );
                dispatcher.push(P2pConnectionIncomingAction::AnswerSendSuccess {
                    peer_id: offerer_pub_key.peer_id(),
                });
                Ok(())
            }
            P2pChannelsSignalingExchangeAction::RequestReceived { .. } => {
                let Self::Ready { remote, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSignalingExchangeAction::RequestReceived`, state: {state:?}",
                    );
                    return Ok(());
                };

                *remote = SignalingExchangeState::Requested { time: meta.time() };

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let state: &P2pState = state.substate()?;
                state.webrtc_discovery_respond_with_availble_peers(dispatcher);
                Ok(())
            }
            P2pChannelsSignalingExchangeAction::OfferSend {
                offer,
                offerer_pub_key,
                ..
            } => {
                let Self::Ready { remote, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSignalingExchangeAction::OfferSend`, state: {state:?}",
                    );
                    return Ok(());
                };

                *remote = SignalingExchangeState::Offered {
                    time: meta.time(),
                    offerer_pub_key: offerer_pub_key.clone(),
                };
                let dispatcher = state_context.into_dispatcher();
                let message = SignalingExchangeChannelMsg::OfferToYou {
                    offerer_pub_key: offerer_pub_key.clone(),
                    offer: offer.clone(),
                };
                dispatcher.push(P2pChannelsSignalingExchangeEffectfulAction::MessageSend {
                    peer_id,
                    message,
                });
                Ok(())
            }
            P2pChannelsSignalingExchangeAction::AnswerReceived { answer, .. } => {
                let Self::Ready { remote, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSignalingExchangeAction::AnswerReceived`, state: {state:?}",
                    );
                    return Ok(());
                };

                let offerer_pub_key = match remote {
                    SignalingExchangeState::Offered {
                        offerer_pub_key, ..
                    } => offerer_pub_key.clone(),
                    state => {
                        bug_condition!(
                            "Invalid state for `P2pChannelsSignalingExchangeAction::AnswerReceived`, state: {state:?}",
                        );
                        return Ok(());
                    }
                };
                *remote = SignalingExchangeState::Answered { time: meta.time() };
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsSignalingDiscoveryAction::AnswerSend {
                    peer_id: offerer_pub_key.peer_id(),
                    answer: answer.clone(),
                });
                Ok(())
            }
        }
    }
}
