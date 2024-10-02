use openmina_core::{bug_condition, Substate};
use redux::ActionWithMeta;

use crate::{
    channels::best_tip_effectful::P2pChannelsBestTipEffectfulAction, P2pNetworkPubsubAction,
    P2pPeerAction, P2pState,
};

use super::{BestTipPropagationState, P2pChannelsBestTipAction, P2pChannelsBestTipState};

impl P2pChannelsBestTipState {
    /// Substate is accessed
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, P2pState>,
        action: ActionWithMeta<&P2pChannelsBestTipAction>,
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
                dispatcher.push(P2pChannelsBestTipEffectfulAction::Init { peer_id });
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
                dispatcher.push(P2pChannelsBestTipEffectfulAction::RequestSend { peer_id });
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
                dispatcher.push(P2pPeerAction::BestTipUpdate {
                    peer_id,
                    best_tip: best_tip.clone(),
                });
                dispatcher.push(P2pChannelsBestTipAction::RequestSend { peer_id });
                Ok(())
            }
            P2pChannelsBestTipAction::RequestReceived { .. } => {
                let Self::Ready { remote, .. } = best_tip_state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsBestTipAction::RequestReceived`, state: {:?}",
                        best_tip_state
                    );
                    return Ok(());
                };

                *remote = BestTipPropagationState::Requested { time: meta.time() };
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
                    dispatcher.push(P2pChannelsBestTipEffectfulAction::ResponseSend {
                        peer_id,
                        best_tip: best_tip.clone(),
                    });
                    return Ok(());
                }

                #[cfg(feature = "p2p-libp2p")]
                {
                    use mina_p2p_messages::gossip::GossipNetMessageV2;
                    let block = (*best_tip.block).clone();
                    let message = Box::new(GossipNetMessageV2::NewState(block));
                    // TODO(vlad): `P2pChannelsBestTipAction::ResponseSend`
                    // action is dispatched for each peer. So `P2pNetworkPubsubAction::Broadcast`
                    // will be called many times causing many duplicate
                    // broadcasts. Either in pubsub state machine, we
                    // need to filter out duplicate messages, or better,
                    // have a simple action to send pubsub message to a
                    // specific peer instead of sending to everyone.
                    // That way we can avoid duplicate state, since we
                    // already store last sent best tip here and we make
                    // sure we don't send same block to same peer again.
                    dispatcher.push(P2pNetworkPubsubAction::Broadcast { message });
                }
                Ok(())
            }
        }
    }
}
