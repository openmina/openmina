use openmina_core::block::BlockWithHash;
use redux::ActionMeta;

use crate::disconnection::{P2pDisconnectionInitAction, P2pDisconnectionReason};

use super::{
    best_tip::{
        BestTipPropagationChannelMsg, P2pChannelsBestTipReceivedAction,
        P2pChannelsBestTipRequestReceivedAction,
    },
    rpc::{
        P2pChannelsRpcRequestReceivedAction, P2pChannelsRpcResponseReceivedAction, RpcChannelMsg,
    },
    snark::{
        P2pChannelsSnarkPromiseReceivedAction, P2pChannelsSnarkReceivedAction,
        P2pChannelsSnarkRequestReceivedAction, SnarkPropagationChannelMsg,
    },
    snark_job_commitment::{
        P2pChannelsSnarkJobCommitmentPromiseReceivedAction,
        P2pChannelsSnarkJobCommitmentReceivedAction,
        P2pChannelsSnarkJobCommitmentRequestReceivedAction,
        SnarkJobCommitmentPropagationChannelMsg,
    },
    ChannelMsg, P2pChannelsMessageReceivedAction,
};

impl P2pChannelsMessageReceivedAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        P2pChannelsBestTipRequestReceivedAction: redux::EnablingCondition<S>,
        P2pChannelsBestTipReceivedAction: redux::EnablingCondition<S>,
        P2pChannelsSnarkRequestReceivedAction: redux::EnablingCondition<S>,
        P2pChannelsSnarkPromiseReceivedAction: redux::EnablingCondition<S>,
        P2pChannelsSnarkReceivedAction: redux::EnablingCondition<S>,
        P2pChannelsSnarkJobCommitmentRequestReceivedAction: redux::EnablingCondition<S>,
        P2pChannelsSnarkJobCommitmentPromiseReceivedAction: redux::EnablingCondition<S>,
        P2pChannelsSnarkJobCommitmentReceivedAction: redux::EnablingCondition<S>,
        P2pChannelsRpcRequestReceivedAction: redux::EnablingCondition<S>,
        P2pChannelsRpcResponseReceivedAction: redux::EnablingCondition<S>,
        P2pDisconnectionInitAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        let chan_id = self.message.channel_id();
        let was_expected = match self.message {
            ChannelMsg::BestTipPropagation(msg) => match msg {
                BestTipPropagationChannelMsg::GetNext => {
                    store.dispatch(P2pChannelsBestTipRequestReceivedAction { peer_id })
                }
                BestTipPropagationChannelMsg::BestTip(best_tip) => {
                    let best_tip = BlockWithHash::new(best_tip);
                    store.dispatch(P2pChannelsBestTipReceivedAction { peer_id, best_tip })
                }
            },
            ChannelMsg::SnarkPropagation(msg) => match msg {
                SnarkPropagationChannelMsg::GetNext { limit } => {
                    store.dispatch(P2pChannelsSnarkRequestReceivedAction { peer_id, limit })
                }
                SnarkPropagationChannelMsg::WillSend { count } => {
                    store.dispatch(P2pChannelsSnarkPromiseReceivedAction {
                        peer_id,
                        promised_count: count,
                    })
                }
                SnarkPropagationChannelMsg::Snark(snark) => {
                    store.dispatch(P2pChannelsSnarkReceivedAction { peer_id, snark })
                }
            },
            ChannelMsg::SnarkJobCommitmentPropagation(msg) => match msg {
                SnarkJobCommitmentPropagationChannelMsg::GetNext { limit } => {
                    store.dispatch(P2pChannelsSnarkJobCommitmentRequestReceivedAction {
                        peer_id,
                        limit,
                    })
                }
                SnarkJobCommitmentPropagationChannelMsg::WillSend { count } => {
                    store.dispatch(P2pChannelsSnarkJobCommitmentPromiseReceivedAction {
                        peer_id,
                        promised_count: count,
                    })
                }
                SnarkJobCommitmentPropagationChannelMsg::Commitment(commitment) => {
                    store.dispatch(P2pChannelsSnarkJobCommitmentReceivedAction {
                        peer_id,
                        commitment,
                    })
                }
            },
            ChannelMsg::Rpc(msg) => match msg {
                RpcChannelMsg::Request(id, request) => {
                    store.dispatch(P2pChannelsRpcRequestReceivedAction {
                        peer_id,
                        id,
                        request,
                    })
                }
                RpcChannelMsg::Response(id, response) => {
                    store.dispatch(P2pChannelsRpcResponseReceivedAction {
                        peer_id,
                        id,
                        response,
                    })
                }
            },
        };

        if !was_expected {
            let reason = P2pDisconnectionReason::P2pChannelMsgUnexpected(chan_id);
            store.dispatch(P2pDisconnectionInitAction { peer_id, reason });
        }
    }
}
