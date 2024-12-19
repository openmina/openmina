use openmina_core::{bug_condition, Substate};
use redux::ActionWithMeta;

use crate::{
    channels::{ChannelId, MsgId, P2pChannelsEffectfulAction},
    P2pNetworkPubsubAction, P2pState,
};

use super::{
    P2pChannelsSnarkAction, P2pChannelsSnarkState, SnarkPropagationChannelMsg,
    SnarkPropagationState,
};
use mina_p2p_messages::{gossip::GossipNetMessageV2, v2};

impl P2pChannelsSnarkState {
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, P2pState>,
        action: ActionWithMeta<P2pChannelsSnarkAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();
        let p2p_state = state_context.get_substate_mut()?;

        let state = action
            .peer_id()
            .and_then(|peer_id| p2p_state.get_ready_peer_mut(peer_id))
            .map(|peer_state| &mut peer_state.channels.snark)
            .ok_or_else(|| format!("Invalid state for: {action:?}"));

        match action {
            P2pChannelsSnarkAction::Init { peer_id } => {
                let state = state.inspect_err(|error| bug_condition!("{}", error))?;
                *state = Self::Init { time: meta.time() };

                let dispatcher = state_context.into_dispatcher();

                dispatcher.push(P2pChannelsEffectfulAction::InitChannel {
                    peer_id,
                    id: ChannelId::SnarkPropagation,
                    on_success: redux::callback!(
                        on_snark_channel_init(peer_id: crate::PeerId) -> crate::P2pAction {
                            P2pChannelsSnarkAction::Pending { peer_id }
                        }
                    ),
                });
                Ok(())
            }
            P2pChannelsSnarkAction::Pending { .. } => {
                let state = state.inspect_err(|error| bug_condition!("{}", error))?;
                *state = Self::Pending { time: meta.time() };
                Ok(())
            }
            P2pChannelsSnarkAction::Ready { .. } => {
                let state = state.inspect_err(|error| bug_condition!("{}", error))?;
                *state = Self::Ready {
                    time: meta.time(),
                    local: SnarkPropagationState::WaitingForRequest { time: meta.time() },
                    remote: SnarkPropagationState::WaitingForRequest { time: meta.time() },
                    next_send_index: 0,
                };
                Ok(())
            }
            P2pChannelsSnarkAction::RequestSend { limit, peer_id } => {
                let state = state.inspect_err(|error| bug_condition!("{}", error))?;
                let Self::Ready { local, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSnarkAction::RequestSend`, state: {:?}",
                        state
                    );
                    return Ok(());
                };
                *local = SnarkPropagationState::Requested {
                    time: meta.time(),
                    requested_limit: limit,
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsEffectfulAction::MessageSend {
                    peer_id,
                    msg_id: MsgId::first(),
                    msg: SnarkPropagationChannelMsg::GetNext { limit }.into(),
                });
                Ok(())
            }
            P2pChannelsSnarkAction::PromiseReceived { promised_count, .. } => {
                let state = state.inspect_err(|error| bug_condition!("{}", error))?;
                let Self::Ready { local, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSnarkAction::PromiseReceived`, state: {:?}",
                        state
                    );
                    return Ok(());
                };
                let SnarkPropagationState::Requested {
                    requested_limit, ..
                } = &local
                else {
                    return Ok(());
                };
                *local = SnarkPropagationState::Responding {
                    time: meta.time(),
                    requested_limit: *requested_limit,
                    promised_count,
                    current_count: 0,
                };
                Ok(())
            }
            P2pChannelsSnarkAction::Received { peer_id, snark } => {
                let state = state.inspect_err(|error| bug_condition!("{}", error))?;
                let Self::Ready { local, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSnarkAction::Received`, state: {:?}",
                        state
                    );
                    return Ok(());
                };
                let SnarkPropagationState::Responding {
                    promised_count,
                    current_count,
                    ..
                } = local
                else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSnarkAction::Received`, state: {:?}",
                        state
                    );
                    return Ok(());
                };

                *current_count += 1;

                if current_count >= promised_count {
                    *local = SnarkPropagationState::Responded {
                        time: meta.time(),
                        count: *current_count,
                    };
                }

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;

                if let Some(callback) = &p2p_state.callbacks.on_p2p_channels_snark_received {
                    dispatcher.push_callback(callback.clone(), (peer_id, snark));
                }

                Ok(())
            }
            P2pChannelsSnarkAction::RequestReceived { limit, .. } => {
                let state = state.inspect_err(|error| bug_condition!("{}", error))?;
                let Self::Ready { remote, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSnarkAction::RequestReceived`, state: {:?}",
                        state
                    );
                    return Ok(());
                };
                *remote = SnarkPropagationState::Requested {
                    time: meta.time(),
                    requested_limit: limit,
                };
                Ok(())
            }
            P2pChannelsSnarkAction::ResponseSend {
                snarks,
                last_index,
                peer_id,
                ..
            } => {
                let state = state.inspect_err(|error| bug_condition!("{}", error))?;
                let Self::Ready {
                    remote,
                    next_send_index,
                    ..
                } = state
                else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSnarkAction::ResponseSend`, state: {:?}",
                        state
                    );
                    return Ok(());
                };
                *next_send_index = last_index + 1;

                let count = snarks.len() as u8;
                if count == 0 {
                    return Ok(());
                }

                *remote = SnarkPropagationState::Responded {
                    time: meta.time(),
                    count,
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsEffectfulAction::MessageSend {
                    peer_id,
                    msg_id: MsgId::first(),
                    msg: SnarkPropagationChannelMsg::WillSend { count }.into(),
                });

                for snark in snarks {
                    dispatcher.push(P2pChannelsEffectfulAction::MessageSend {
                        peer_id,
                        msg_id: MsgId::first(),
                        msg: SnarkPropagationChannelMsg::Snark(snark).into(),
                    });
                }
                Ok(())
            }
            #[cfg(feature = "p2p-libp2p")]
            P2pChannelsSnarkAction::Libp2pBroadcast { snark, nonce } => {
                let dispatcher = state_context.into_dispatcher();
                let message = Box::new((snark.statement(), (&snark).into()));
                let message = v2::NetworkPoolSnarkPoolDiffVersionedStableV2::AddSolvedWork(message);
                let nonce = nonce.into();
                let message = GossipNetMessageV2::SnarkPoolDiff { message, nonce };
                dispatcher.push(P2pNetworkPubsubAction::Broadcast { message });
                Ok(())
            }
            #[cfg(not(feature = "p2p-libp2p"))]
            P2pChannelsSnarkAction::Libp2pBroadcast { .. } => Ok(()),
            P2pChannelsSnarkAction::Libp2pReceived { peer_id, snark, .. } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;

                if let Some(callback) = &p2p_state.callbacks.on_p2p_channels_snark_libp2p_received {
                    dispatcher.push_callback(callback.clone(), (peer_id, snark));
                }

                Ok(())
            }
        }
    }
}
