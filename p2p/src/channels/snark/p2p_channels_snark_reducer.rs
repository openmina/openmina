use openmina_core::{bug_condition, Substate};
use redux::ActionWithMeta;

use crate::{
    channels::snark_effectful::P2pChannelsSnarkEffectfulAction, P2pNetworkPubsubAction, P2pState,
};

use super::{P2pChannelsSnarkAction, P2pChannelsSnarkState, SnarkPropagationState};
use mina_p2p_messages::{gossip::GossipNetMessageV2, v2};

impl P2pChannelsSnarkState {
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, P2pState>,
        action: ActionWithMeta<&P2pChannelsSnarkAction>,
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
            .ok_or_else(|| format!("Invalid state for: {action:?}"))
            .inspect_err(|error| bug_condition!("{}", error));

        match action {
            P2pChannelsSnarkAction::Init { peer_id } => {
                let state = state?;
                *state = Self::Init { time: meta.time() };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsSnarkEffectfulAction::Init { peer_id: *peer_id });
                Ok(())
            }
            P2pChannelsSnarkAction::Pending { .. } => {
                let state = state?;
                *state = Self::Pending { time: meta.time() };
                Ok(())
            }
            P2pChannelsSnarkAction::Ready { .. } => {
                let state = state?;
                *state = Self::Ready {
                    time: meta.time(),
                    local: SnarkPropagationState::WaitingForRequest { time: meta.time() },
                    remote: SnarkPropagationState::WaitingForRequest { time: meta.time() },
                    next_send_index: 0,
                };
                Ok(())
            }
            P2pChannelsSnarkAction::RequestSend { limit, peer_id } => {
                let state = state?;
                let Self::Ready { local, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSnarkAction::RequestSend`, state: {:?}",
                        state
                    );
                    return Ok(());
                };
                *local = SnarkPropagationState::Requested {
                    time: meta.time(),
                    requested_limit: *limit,
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsSnarkEffectfulAction::RequestSend {
                    peer_id: *peer_id,
                    limit: *limit,
                });
                Ok(())
            }
            P2pChannelsSnarkAction::PromiseReceived { promised_count, .. } => {
                let state = state?;
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
                    promised_count: *promised_count,
                    current_count: 0,
                };
                Ok(())
            }
            P2pChannelsSnarkAction::Received { .. } => {
                let state = state?;
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
                Ok(())
            }
            P2pChannelsSnarkAction::RequestReceived { limit, .. } => {
                let state = state?;
                let Self::Ready { remote, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsSnarkAction::RequestReceived`, state: {:?}",
                        state
                    );
                    return Ok(());
                };
                *remote = SnarkPropagationState::Requested {
                    time: meta.time(),
                    requested_limit: *limit,
                };
                Ok(())
            }
            P2pChannelsSnarkAction::ResponseSend {
                snarks,
                last_index,
                peer_id,
                ..
            } => {
                let state = state?;
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
                dispatcher.push(P2pChannelsSnarkEffectfulAction::ResponseSend {
                    peer_id: *peer_id,
                    snarks: snarks.clone(),
                });
                Ok(())
            }
            #[cfg(feature = "p2p-libp2p")]
            P2pChannelsSnarkAction::Libp2pBroadcast { snark, nonce } => {
                let dispatcher = state_context.into_dispatcher();
                let message = Box::new((snark.statement(), (snark).into()));
                let message = v2::NetworkPoolSnarkPoolDiffVersionedStableV2::AddSolvedWork(message);
                let nonce = nonce.into();
                let message = Box::new(GossipNetMessageV2::SnarkPoolDiff { message, nonce });
                dispatcher.push(P2pNetworkPubsubAction::Broadcast { message });
                Ok(())
            }
            #[cfg(not(feature = "p2p-libp2p"))]
            P2pChannelsSnarkAction::Libp2pBroadcast { .. } => Ok(()),
            P2pChannelsSnarkAction::Libp2pReceived { .. } => Ok(()),
        }
    }
}
