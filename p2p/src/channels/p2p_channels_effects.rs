use redux::ActionMeta;
use shared::block::BlockWithHash;
use snark::hash::state_hash;

use crate::disconnection::P2pDisconnectionInitAction;

use super::{
    best_tip::{
        BestTipPropagationChannelMsg, P2pChannelsBestTipReceivedAction,
        P2pChannelsBestTipRequestReceivedAction,
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
        P2pChannelsSnarkJobCommitmentRequestReceivedAction: redux::EnablingCondition<S>,
        P2pChannelsSnarkJobCommitmentPromiseReceivedAction: redux::EnablingCondition<S>,
        P2pChannelsSnarkJobCommitmentReceivedAction: redux::EnablingCondition<S>,
        P2pDisconnectionInitAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        let was_expected = match self.message {
            ChannelMsg::BestTipPropagation(msg) => match msg {
                BestTipPropagationChannelMsg::GetNext => {
                    store.dispatch(P2pChannelsBestTipRequestReceivedAction { peer_id })
                }
                BestTipPropagationChannelMsg::BestTip(best_tip) => {
                    let best_tip = BlockWithHash {
                        hash: state_hash(&*best_tip),
                        block: best_tip,
                    };
                    store.dispatch(P2pChannelsBestTipReceivedAction { peer_id, best_tip })
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
        };

        if !was_expected {
            // TODO(binier): have separate action for unexpected message.
            store.dispatch(P2pDisconnectionInitAction { peer_id });
        }
    }
}
