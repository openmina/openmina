use mina_core::{bug_condition, Substate};
use redux::ActionWithMeta;

use crate::{
    channels::{ChannelId, ChannelMsg, MsgId, P2pChannelsEffectfulAction},
    P2pPeerAction, P2pState, PeerId,
};

use super::{
    BestTipPropagationChannelMsg, BestTipPropagationState, P2pChannelsBestTipAction,
    P2pChannelsBestTipState,
};

impl P2pChannelsBestTipState {
    /// Substate is accessed
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, P2pState>,
        action: ActionWithMeta<P2pChannelsBestTipAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();
        let p2p_state = state_context.get_substate_mut()?;
        let peer_id = *action.peer_id();
        let is_libp2p = p2p_state.is_libp2p_peer(&peer_id);
        let best_tip_state = &mut p2p_state
            .get_ready_peer_mut(&peer_id)
            .ok_or_else(|| format!("Peer state not found for: {action:?}"))?
            .channels
            .best_tip;

        match action {
            P2pChannelsBestTipAction::Init { .. } => {
                *best_tip_state = Self::Init { time: meta.time() };

                let dispatcher = state_context.into_dispatcher();

                dispatcher.push(P2pChannelsEffectfulAction::InitChannel {
                    peer_id,
                    id: ChannelId::BestTipPropagation,
                    on_success: redux::callback!(
                        on_best_tip_channel_init(peer_id: PeerId) -> crate::P2pAction {
                            P2pChannelsBestTipAction::Pending { peer_id }
                        }
                    ),
                });
                Ok(())
            }
            P2pChannelsBestTipAction::Pending { .. } => {
                *best_tip_state = Self::Pending { time: meta.time() };
                Ok(())
            }
            P2pChannelsBestTipAction::Ready { .. } => {
                *best_tip_state = Self::Ready {
                    time: meta.time(),
                    local: BestTipPropagationState::WaitingForRequest { time: meta.time() },
                    remote: BestTipPropagationState::WaitingForRequest { time: meta.time() },
                    last_sent: None,
                    last_received: None,
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsBestTipAction::RequestSend { peer_id });

                #[cfg(feature = "p2p-libp2p")]
                if is_libp2p {
                    dispatcher.push(P2pChannelsBestTipAction::RequestReceived { peer_id });
                }
                Ok(())
            }
            P2pChannelsBestTipAction::RequestSend { .. } => {
                let Self::Ready { local, .. } = best_tip_state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsBestTipAction::RequestSend`, state: {:?}",
                        best_tip_state
                    );
                    return Ok(());
                };
                *local = BestTipPropagationState::Requested { time: meta.time() };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsEffectfulAction::MessageSend {
                    peer_id,
                    msg_id: MsgId::first(),
                    msg: ChannelMsg::BestTipPropagation(BestTipPropagationChannelMsg::GetNext),
                });
                Ok(())
            }
            P2pChannelsBestTipAction::Received { best_tip, .. } => {
                let Self::Ready {
                    local,
                    last_received,
                    ..
                } = best_tip_state
                else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsBestTipAction::Received`, state: {:?}",
                        best_tip_state
                    );
                    return Ok(());
                };

                *local = BestTipPropagationState::Responded { time: meta.time() };
                *last_received = Some(best_tip.clone());

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pPeerAction::BestTipUpdate { peer_id, best_tip });
                dispatcher.push(P2pChannelsBestTipAction::RequestSend { peer_id });
                Ok(())
            }
            P2pChannelsBestTipAction::RequestReceived { peer_id, .. } => {
                let Self::Ready { remote, .. } = best_tip_state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsBestTipAction::RequestReceived`, state: {:?}",
                        best_tip_state
                    );
                    return Ok(());
                };

                *remote = BestTipPropagationState::Requested { time: meta.time() };

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;

                if let Some(callback) = &p2p_state
                    .callbacks
                    .on_p2p_channels_best_tip_request_received
                {
                    dispatcher.push_callback(callback.clone(), peer_id);
                }
                Ok(())
            }
            P2pChannelsBestTipAction::ResponseSend { best_tip, .. } => {
                let Self::Ready {
                    remote, last_sent, ..
                } = best_tip_state
                else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsBestTipAction::ResponseSend`, state: {:?}",
                        best_tip_state
                    );
                    return Ok(());
                };

                if !is_libp2p {
                    *remote = BestTipPropagationState::Responded { time: meta.time() };
                }
                *last_sent = Some(best_tip.clone());

                let dispatcher = state_context.into_dispatcher();

                if !is_libp2p {
                    dispatcher.push(P2pChannelsEffectfulAction::MessageSend {
                        peer_id,
                        msg_id: MsgId::first(),
                        msg: ChannelMsg::BestTipPropagation(BestTipPropagationChannelMsg::BestTip(
                            best_tip.block,
                        )),
                    });
                    return Ok(());
                }

                Ok(())
            }
        }
    }
}
