use super::{
    best_tip::{BestTipPropagationChannelMsg, P2pChannelsBestTipAction, P2pChannelsBestTipState},
    rpc::{P2pChannelsRpcAction, P2pChannelsRpcState, RpcChannelMsg},
    snark::{P2pChannelsSnarkAction, P2pChannelsSnarkState, SnarkPropagationChannelMsg},
    snark_job_commitment::{
        P2pChannelsSnarkJobCommitmentAction, P2pChannelsSnarkJobCommitmentState,
        SnarkJobCommitmentPropagationChannelMsg,
    },
    streaming_rpc::{
        P2pChannelsStreamingRpcAction, P2pChannelsStreamingRpcState, StreamingRpcChannelMsg,
    },
    transaction::{
        P2pChannelsTransactionAction, P2pChannelsTransactionState, TransactionPropagationChannelMsg,
    },
    ChannelMsg, P2pChannelsAction, P2pChannelsMessageReceivedAction, P2pChannelsState,
};
use crate::{
    disconnection::{P2pDisconnectionAction, P2pDisconnectionReason},
    P2pState,
};
use openmina_core::{block::BlockWithHash, error, Substate};
use redux::{ActionWithMeta, Dispatcher};

impl P2pChannelsState {
    pub fn reducer<Action, State>(
        state_context: Substate<Action, State, P2pState>,
        action: ActionWithMeta<&P2pChannelsAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();

        match action {
            P2pChannelsAction::MessageReceived(action) => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                Self::dispatch_message(meta.with_action(action), dispatcher, state)
            }
            P2pChannelsAction::BestTip(action) => {
                P2pChannelsBestTipState::reducer(state_context, meta.with_action(action))
            }
            P2pChannelsAction::Transaction(action) => {
                P2pChannelsTransactionState::reducer(state_context, meta.with_action(action))
            }
            P2pChannelsAction::Snark(action) => {
                P2pChannelsSnarkState::reducer(state_context, meta.with_action(action))
            }
            P2pChannelsAction::SnarkJobCommitment(action) => {
                P2pChannelsSnarkJobCommitmentState::reducer(state_context, meta.with_action(action))
            }
            P2pChannelsAction::Rpc(action) => {
                P2pChannelsRpcState::reducer(state_context, meta.with_action(action))
            }
            P2pChannelsAction::StreamingRpc(action) => {
                P2pChannelsStreamingRpcState::reducer(state_context, meta.with_action(action))
            }
        }
    }

    fn dispatch_message<Action, State>(
        action: ActionWithMeta<&P2pChannelsMessageReceivedAction>,
        dispatcher: &mut Dispatcher<Action, State>,
        state: &State,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();
        let time = meta.time();

        let peer_id = action.peer_id;
        let chain_id = action.message.channel_id();

        let mut is_enabled = |action: Action| dispatcher.push_if_enabled(action, state, time);

        let was_expected = match *action.message.clone() {
            ChannelMsg::BestTipPropagation(msg) => match msg {
                BestTipPropagationChannelMsg::GetNext => {
                    is_enabled(P2pChannelsBestTipAction::RequestReceived { peer_id }.into())
                }
                BestTipPropagationChannelMsg::BestTip(best_tip) => {
                    match BlockWithHash::try_new(best_tip) {
                        Ok(best_tip) => is_enabled(
                            P2pChannelsBestTipAction::Received { peer_id, best_tip }.into(),
                        ),
                        Err(_) => {
                            error!(meta.time(); "BestTipPropagationChannelMsg::BestTip: Invalid bigint in block");
                            false
                        }
                    }
                }
            },
            ChannelMsg::TransactionPropagation(msg) => match msg {
                TransactionPropagationChannelMsg::GetNext { limit } => is_enabled(
                    P2pChannelsTransactionAction::RequestReceived { peer_id, limit }.into(),
                ),
                TransactionPropagationChannelMsg::WillSend { count } => is_enabled(
                    P2pChannelsTransactionAction::PromiseReceived {
                        peer_id,
                        promised_count: count,
                    }
                    .into(),
                ),
                TransactionPropagationChannelMsg::Transaction(transaction) => is_enabled(
                    P2pChannelsTransactionAction::Received {
                        peer_id,
                        transaction: Box::new(transaction),
                    }
                    .into(),
                ),
            },
            ChannelMsg::SnarkPropagation(msg) => match msg {
                SnarkPropagationChannelMsg::GetNext { limit } => {
                    is_enabled(P2pChannelsSnarkAction::RequestReceived { peer_id, limit }.into())
                }
                SnarkPropagationChannelMsg::WillSend { count } => is_enabled(
                    P2pChannelsSnarkAction::PromiseReceived {
                        peer_id,
                        promised_count: count,
                    }
                    .into(),
                ),
                SnarkPropagationChannelMsg::Snark(snark) => is_enabled(
                    P2pChannelsSnarkAction::Received {
                        peer_id,
                        snark: Box::new(snark),
                    }
                    .into(),
                ),
            },
            ChannelMsg::SnarkJobCommitmentPropagation(msg) => match msg {
                SnarkJobCommitmentPropagationChannelMsg::GetNext { limit } => is_enabled(
                    P2pChannelsSnarkJobCommitmentAction::RequestReceived { peer_id, limit }.into(),
                ),
                SnarkJobCommitmentPropagationChannelMsg::WillSend { count } => is_enabled(
                    P2pChannelsSnarkJobCommitmentAction::PromiseReceived {
                        peer_id,
                        promised_count: count,
                    }
                    .into(),
                ),
                SnarkJobCommitmentPropagationChannelMsg::Commitment(commitment) => is_enabled(
                    P2pChannelsSnarkJobCommitmentAction::Received {
                        peer_id,
                        commitment: Box::new(commitment),
                    }
                    .into(),
                ),
            },
            ChannelMsg::Rpc(msg) => match msg {
                RpcChannelMsg::Request(id, request) => is_enabled(
                    P2pChannelsRpcAction::RequestReceived {
                        peer_id,
                        id,
                        request: Box::new(request),
                    }
                    .into(),
                ),
                RpcChannelMsg::Response(id, response) => is_enabled(
                    P2pChannelsRpcAction::ResponseReceived {
                        peer_id,
                        id,
                        response: response.map(Box::new),
                    }
                    .into(),
                ),
            },
            ChannelMsg::StreamingRpc(msg) => match msg {
                StreamingRpcChannelMsg::Next(id) => is_enabled(
                    P2pChannelsStreamingRpcAction::ResponsePartNextSend { peer_id, id }.into(),
                ),
                StreamingRpcChannelMsg::Request(id, request) => is_enabled(
                    P2pChannelsStreamingRpcAction::RequestReceived {
                        peer_id,
                        id,
                        request: Box::new(request),
                    }
                    .into(),
                ),
                StreamingRpcChannelMsg::Response(id, response) => match response {
                    None => is_enabled(
                        P2pChannelsStreamingRpcAction::ResponseReceived {
                            peer_id,
                            id,
                            response: None,
                        }
                        .into(),
                    ),
                    Some(response) => is_enabled(
                        P2pChannelsStreamingRpcAction::ResponsePartReceived {
                            peer_id,
                            id,
                            response,
                        }
                        .into(),
                    ),
                },
            },
        };

        if !was_expected {
            let reason = P2pDisconnectionReason::P2pChannelMsgUnexpected(chain_id);
            dispatcher.push(P2pDisconnectionAction::Init { peer_id, reason });
        }

        Ok(())
    }
}
