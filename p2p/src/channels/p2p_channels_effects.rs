use openmina_core::block::BlockWithHash;
use redux::ActionMeta;

use crate::disconnection::{P2pDisconnectionAction, P2pDisconnectionReason};

use super::{
    best_tip::{BestTipPropagationChannelMsg, P2pChannelsBestTipAction},
    rpc::{P2pChannelsRpcAction, RpcChannelMsg},
    snark::{P2pChannelsSnarkAction, SnarkPropagationChannelMsg},
    snark_job_commitment::{
        P2pChannelsSnarkJobCommitmentAction, SnarkJobCommitmentPropagationChannelMsg,
    },
    streaming_rpc::{P2pChannelsStreamingRpcAction, StreamingRpcChannelMsg},
    transaction::{P2pChannelsTransactionAction, TransactionPropagationChannelMsg},
    ChannelMsg, P2pChannelsMessageReceivedAction,
};

impl P2pChannelsMessageReceivedAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
    {
        let peer_id = self.peer_id;
        let chan_id = self.message.channel_id();

        let was_expected = match *self.message {
            ChannelMsg::BestTipPropagation(msg) => match msg {
                BestTipPropagationChannelMsg::GetNext => {
                    store.dispatch(P2pChannelsBestTipAction::RequestReceived { peer_id })
                }
                BestTipPropagationChannelMsg::BestTip(best_tip) => {
                    let best_tip = BlockWithHash::new(best_tip);
                    store.dispatch(P2pChannelsBestTipAction::Received { peer_id, best_tip })
                }
            },
            ChannelMsg::TransactionPropagation(msg) => match msg {
                TransactionPropagationChannelMsg::GetNext { limit } => {
                    store.dispatch(P2pChannelsTransactionAction::RequestReceived { peer_id, limit })
                }
                TransactionPropagationChannelMsg::WillSend { count } => {
                    store.dispatch(P2pChannelsTransactionAction::PromiseReceived {
                        peer_id,
                        promised_count: count,
                    })
                }
                TransactionPropagationChannelMsg::Transaction(transaction) => {
                    store.dispatch(P2pChannelsTransactionAction::Received {
                        peer_id,
                        transaction: Box::new(transaction),
                    })
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
                    store.dispatch(P2pChannelsSnarkAction::Received {
                        peer_id,
                        snark: Box::new(snark),
                    })
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
                        commitment: Box::new(commitment),
                    })
                }
            },
            ChannelMsg::Rpc(msg) => match msg {
                RpcChannelMsg::Request(id, request) => {
                    store.dispatch(P2pChannelsRpcAction::RequestReceived {
                        peer_id,
                        id,
                        request: Box::new(request),
                    })
                }
                RpcChannelMsg::Response(id, response) => {
                    store.dispatch(P2pChannelsRpcAction::ResponseReceived {
                        peer_id,
                        id,
                        response: response.map(Box::new),
                    })
                }
            },
            ChannelMsg::StreamingRpc(msg) => match msg {
                StreamingRpcChannelMsg::Next(id) => store
                    .dispatch(P2pChannelsStreamingRpcAction::ResponsePartNextSend { peer_id, id }),
                StreamingRpcChannelMsg::Request(id, request) => {
                    store.dispatch(P2pChannelsStreamingRpcAction::RequestReceived {
                        peer_id,
                        id,
                        request: Box::new(request),
                    })
                }
                StreamingRpcChannelMsg::Response(id, response) => match response {
                    None => store.dispatch(P2pChannelsStreamingRpcAction::ResponseReceived {
                        peer_id,
                        id,
                        response: None,
                    }),
                    Some(response) => {
                        store.dispatch(P2pChannelsStreamingRpcAction::ResponsePartReceived {
                            peer_id,
                            id,
                            response,
                        })
                    }
                },
            },
        };

        if !was_expected {
            let reason = P2pDisconnectionReason::P2pChannelMsgUnexpected(chan_id);
            store.dispatch(P2pDisconnectionAction::Init { peer_id, reason });
        }
    }
}
