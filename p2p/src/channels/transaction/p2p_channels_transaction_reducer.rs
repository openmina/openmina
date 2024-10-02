use super::{
    P2pChannelsTransactionAction, P2pChannelsTransactionState, TransactionPropagationState,
};
use crate::{
    channels::transaction_effectful::P2pChannelsTransactionEffectfulAction, P2pNetworkPubsubAction,
    P2pState,
};
use mina_p2p_messages::{gossip::GossipNetMessageV2, v2};
use openmina_core::{bug_condition, Substate};
use redux::ActionWithMeta;

impl P2pChannelsTransactionState {
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, P2pState>,
        action: ActionWithMeta<&P2pChannelsTransactionAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();
        let p2p_state = state_context.get_substate_mut()?;

        let transaction_state = action
            .peer_id()
            .and_then(|peer_id| p2p_state.get_ready_peer_mut(peer_id))
            .map(|peer_state| &mut peer_state.channels.transaction)
            .ok_or_else(|| format!("Invalid state for: {action:?}"));

        match action {
            P2pChannelsTransactionAction::Init { peer_id } => {
                let state = transaction_state.inspect_err(|error| bug_condition!("{}", error))?;
                *state = Self::Init { time: meta.time() };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsTransactionEffectfulAction::Init { peer_id: *peer_id });
                Ok(())
            }
            P2pChannelsTransactionAction::Pending { .. } => {
                let state = transaction_state.inspect_err(|error| bug_condition!("{}", error))?;
                *state = Self::Pending { time: meta.time() };
                Ok(())
            }
            P2pChannelsTransactionAction::Ready { .. } => {
                let state = transaction_state.inspect_err(|error| bug_condition!("{}", error))?;
                *state = Self::Ready {
                    time: meta.time(),
                    local: TransactionPropagationState::WaitingForRequest { time: meta.time() },
                    remote: TransactionPropagationState::WaitingForRequest { time: meta.time() },
                    next_send_index: 0,
                };
                Ok(())
            }
            P2pChannelsTransactionAction::RequestSend { limit, peer_id, .. } => {
                let state = transaction_state.inspect_err(|error| bug_condition!("{}", error))?;
                let Self::Ready { local, .. } = state else {
                    bug_condition!(
                    "Invalid state for `P2pChannelsTransactionAction::RequestSend `, state: {:?}",
                    state
                );
                    return Ok(());
                };
                *local = TransactionPropagationState::Requested {
                    time: meta.time(),
                    requested_limit: *limit,
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsTransactionEffectfulAction::RequestSend {
                    peer_id: *peer_id,
                    limit: *limit,
                });
                Ok(())
            }
            P2pChannelsTransactionAction::PromiseReceived { promised_count, .. } => {
                let state = transaction_state.inspect_err(|error| bug_condition!("{}", error))?;
                let Self::Ready { local, .. } = state else {
                    bug_condition!(
                    "Invalid state for `P2pChannelsTransactionAction::PromiseReceived `, state: {:?}",
                    state
                );
                    return Ok(());
                };
                let TransactionPropagationState::Requested {
                    requested_limit, ..
                } = &local
                else {
                    bug_condition!(
                    "Invalid state for `P2pChannelsTransactionAction::PromiseReceived `, state: {:?}",
                    state
                );
                    return Ok(());
                };
                *local = TransactionPropagationState::Responding {
                    time: meta.time(),
                    requested_limit: *requested_limit,
                    promised_count: *promised_count,
                    current_count: 0,
                };
                Ok(())
            }
            P2pChannelsTransactionAction::Received { .. } => {
                let state = transaction_state.inspect_err(|error| bug_condition!("{}", error))?;
                let Self::Ready { local, .. } = state else {
                    bug_condition!(
                        "Invalid state for `P2pChannelsTransactionAction::Received `, state: {:?}",
                        state
                    );
                    return Ok(());
                };
                let TransactionPropagationState::Responding {
                    promised_count,
                    current_count,
                    ..
                } = local
                else {
                    return Ok(());
                };

                *current_count += 1;

                if current_count >= promised_count {
                    *local = TransactionPropagationState::Responded {
                        time: meta.time(),
                        count: *current_count,
                    };
                }
                Ok(())
            }
            P2pChannelsTransactionAction::RequestReceived { limit, .. } => {
                let state = transaction_state.inspect_err(|error| bug_condition!("{}", error))?;
                let Self::Ready { remote, .. } = state else {
                    bug_condition!(
                    "Invalid state for `P2pChannelsTransactionAction::RequestReceived `, state: {:?}",
                    state
                );
                    return Ok(());
                };
                *remote = TransactionPropagationState::Requested {
                    time: meta.time(),
                    requested_limit: *limit,
                };
                Ok(())
            }
            P2pChannelsTransactionAction::ResponseSend {
                transactions,
                last_index,
                peer_id,
                ..
            } => {
                let state = transaction_state.inspect_err(|error| bug_condition!("{}", error))?;
                let Self::Ready {
                    remote,
                    next_send_index,
                    ..
                } = state
                else {
                    bug_condition!(
                    "Invalid state for `P2pChannelsTransactionAction::ResponseSend `, state: {:?}",
                    state
                );
                    return Ok(());
                };
                *next_send_index = last_index + 1;

                let count = transactions.len() as u8;
                if count == 0 {
                    return Ok(());
                }

                *remote = TransactionPropagationState::Responded {
                    time: meta.time(),
                    count,
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pChannelsTransactionEffectfulAction::ResponseSend {
                    peer_id: *peer_id,
                    transactions: transactions.clone(),
                });
                Ok(())
            }
            P2pChannelsTransactionAction::Libp2pReceived { .. } => Ok(()),
            #[cfg(not(feature = "p2p-libp2p"))]
            P2pChannelsTransactionAction::Libp2pBroadcast { .. } => Ok(()),
            #[cfg(feature = "p2p-libp2p")]
            P2pChannelsTransactionAction::Libp2pBroadcast { transaction, nonce } => {
                let dispatcher = state_context.into_dispatcher();
                let message = v2::NetworkPoolTransactionPoolDiffVersionedStableV2(
                    std::iter::once(*transaction.clone()).collect(),
                );
                let nonce = nonce.into();
                let message = Box::new(GossipNetMessageV2::TransactionPoolDiff { message, nonce });
                dispatcher.push(P2pNetworkPubsubAction::Broadcast { message });
                Ok(())
            }
        }
    }
}
