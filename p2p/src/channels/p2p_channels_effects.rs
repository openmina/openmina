use openmina_core::block::BlockWithHash;
use redux::ActionMeta;

use crate::disconnection::{P2pDisconnectionInitAction, P2pDisconnectionReason};

use super::{
    best_tip::{BestTipPropagationChannelMsg, P2pChannelsBestTipAction},
    rpc::{P2pChannelsRpcAction, RpcChannelMsg},
    snark::{P2pChannelsSnarkAction, SnarkPropagationChannelMsg},
    snark_job_commitment::{
        P2pChannelsSnarkJobCommitmentAction, SnarkJobCommitmentPropagationChannelMsg,
    },
    ChannelMsg, P2pChannelsMessageReceivedAction,
};

impl P2pChannelsMessageReceivedAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        P2pChannelsBestTipAction: redux::EnablingCondition<S>,
        P2pChannelsSnarkAction: redux::EnablingCondition<S>,
        P2pChannelsSnarkJobCommitmentAction: redux::EnablingCondition<S>,
        P2pChannelsRpcAction: redux::EnablingCondition<S>,
        P2pDisconnectionInitAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        let chan_id = self.message.channel_id();
        let was_expected = match self.message {
            ChannelMsg::BestTipPropagation(msg) => match msg {
                BestTipPropagationChannelMsg::GetNext => {
                    store.dispatch(P2pChannelsBestTipAction::RequestReceived { peer_id })
                }
                BestTipPropagationChannelMsg::BestTip(best_tip) => {
                    let best_tip = BlockWithHash::new(best_tip);
                    store.dispatch(P2pChannelsBestTipAction::Received { peer_id, best_tip })
                }
            },
            ChannelMsg::SnarkPropagation(msg) => match msg {
                SnarkPropagationChannelMsg::GetNext { limit } => {
                    store.dispatch(P2pChannelsSnarkAction::RequestReceived { peer_id, limit })
                }
                SnarkPropagationChannelMsg::WillSend { count } => {
                    store.dispatch(P2pChannelsSnarkAction::PromiseReceived {
                        peer_id,
                        promised_count: count,
                    })
                }
                SnarkPropagationChannelMsg::Snark(snark) => {
                    store.dispatch(P2pChannelsSnarkAction::Received { peer_id, snark })
                }
            },
            ChannelMsg::SnarkJobCommitmentPropagation(msg) => match msg {
                SnarkJobCommitmentPropagationChannelMsg::GetNext { limit } => {
                    store.dispatch(P2pChannelsSnarkJobCommitmentAction::RequestReceived {
                        peer_id,
                        limit,
                    })
                }
                SnarkJobCommitmentPropagationChannelMsg::WillSend { count } => {
                    store.dispatch(P2pChannelsSnarkJobCommitmentAction::PromiseReceived {
                        peer_id,
                        promised_count: count,
                    })
                }
                SnarkJobCommitmentPropagationChannelMsg::Commitment(commitment) => {
                    store.dispatch(P2pChannelsSnarkJobCommitmentAction::Received {
                        peer_id,
                        commitment,
                    })
                }
            },
            ChannelMsg::Rpc(msg) => match msg {
                RpcChannelMsg::Request(id, request) => {
                    store.dispatch(P2pChannelsRpcAction::RequestReceived {
                        peer_id,
                        id,
                        request,
                    })
                }
                RpcChannelMsg::Response(id, response) => {
                    store.dispatch(P2pChannelsRpcAction::ResponseReceived {
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
