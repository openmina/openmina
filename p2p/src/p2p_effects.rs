use redux::ActionMeta;

use crate::{
    channels::{
        best_tip::P2pChannelsBestTipInitAction,
        snark_job_commitment::P2pChannelsSnarkJobCommitmentInitAction, ChannelId,
    },
    P2pPeerReadyAction,
};

impl P2pPeerReadyAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        P2pChannelsBestTipInitAction: redux::EnablingCondition<S>,
        P2pChannelsSnarkJobCommitmentInitAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        // Dispatches can be done without a loop, but inside we do
        // exhaustive matching so that we don't miss any channels.
        for id in ChannelId::iter_all() {
            match id {
                ChannelId::BestTipPropagation => {
                    store.dispatch(P2pChannelsBestTipInitAction { peer_id });
                }
                ChannelId::SnarkJobCommitmentPropagation => {
                    store.dispatch(P2pChannelsSnarkJobCommitmentInitAction { peer_id });
                }
            }
        }
    }
}
