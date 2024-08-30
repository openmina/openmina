use redux::ActionMeta;

use crate::channels::{
    best_tip::P2pChannelsBestTipAction, rpc::P2pChannelsRpcAction, snark::P2pChannelsSnarkAction,
    snark_job_commitment::P2pChannelsSnarkJobCommitmentAction,
    streaming_rpc::P2pChannelsStreamingRpcAction, transaction::P2pChannelsTransactionAction,
    ChannelId,
};

use super::P2pPeerAction;

impl P2pPeerAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
    {
        match self {
            P2pPeerAction::Discovered { .. } => {
                // TODO
            }
            P2pPeerAction::Ready { peer_id, .. } => {
                // Dispatches can be done without a loop, but inside we do
                // exhaustive matching so that we don't miss any channels.
                for id in ChannelId::iter_all() {
                    match id {
                        ChannelId::BestTipPropagation => {
                            store.dispatch(P2pChannelsBestTipAction::Init { peer_id });
                        }
                        ChannelId::TransactionPropagation => {
                            store.dispatch(P2pChannelsTransactionAction::Init { peer_id });
                        }
                        ChannelId::SnarkPropagation => {
                            store.dispatch(P2pChannelsSnarkAction::Init { peer_id });
                        }
                        ChannelId::SnarkJobCommitmentPropagation => {
                            store.dispatch(P2pChannelsSnarkJobCommitmentAction::Init { peer_id });
                        }
                        ChannelId::Rpc => {
                            store.dispatch(P2pChannelsRpcAction::Init { peer_id });
                        }
                        ChannelId::StreamingRpc => {
                            store.dispatch(P2pChannelsStreamingRpcAction::Init { peer_id });
                        }
                    }
                }
            }
            P2pPeerAction::BestTipUpdate { .. } => {}
        }
    }
}
