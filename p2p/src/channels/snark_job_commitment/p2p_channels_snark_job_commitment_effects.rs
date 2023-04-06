use redux::ActionMeta;

use crate::channels::{ChannelId, P2pChannelsService};

use super::{
    P2pChannelsSnarkJobCommitmentInitAction, P2pChannelsSnarkJobCommitmentPendingAction,
    P2pChannelsSnarkJobCommitmentReadyAction,
};

impl P2pChannelsSnarkJobCommitmentInitAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
        P2pChannelsSnarkJobCommitmentPendingAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        store
            .service()
            .channel_open(peer_id, ChannelId::SnarkJobCommitmentPropagation);
        store.dispatch(P2pChannelsSnarkJobCommitmentPendingAction { peer_id });
    }
}

impl P2pChannelsSnarkJobCommitmentReadyAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        let peer_id = self.peer_id;
    }
}
