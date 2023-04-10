use redux::ActionMeta;

use crate::channels::{ChannelId, MsgId, P2pChannelsService};

use super::{
    P2pChannelsSnarkJobCommitmentInitAction, P2pChannelsSnarkJobCommitmentPendingAction,
    P2pChannelsSnarkJobCommitmentReadyAction, P2pChannelsSnarkJobCommitmentReceivedAction,
    P2pChannelsSnarkJobCommitmentRequestSendAction,
    P2pChannelsSnarkJobCommitmentResponseSendAction, SnarkJobCommitmentPropagationChannelMsg,
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
        P2pChannelsSnarkJobCommitmentRequestSendAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        let limit = 16;
        store.dispatch(P2pChannelsSnarkJobCommitmentRequestSendAction { peer_id, limit });
    }
}

impl P2pChannelsSnarkJobCommitmentRequestSendAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        let peer_id = self.peer_id;
        let limit = self.limit;
        let msg = SnarkJobCommitmentPropagationChannelMsg::GetNext { limit };
        store
            .service()
            .channel_send(peer_id, MsgId::first(), msg.into());
    }
}

impl P2pChannelsSnarkJobCommitmentReceivedAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
        P2pChannelsSnarkJobCommitmentRequestSendAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        let limit = 16;
        store.dispatch(P2pChannelsSnarkJobCommitmentRequestSendAction { peer_id, limit });
    }
}

impl P2pChannelsSnarkJobCommitmentResponseSendAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        let peer_id = self.peer_id;
        let msg = SnarkJobCommitmentPropagationChannelMsg::WillSend {
            count: self.commitments.len() as u8,
        };
        store
            .service()
            .channel_send(peer_id, MsgId::first(), msg.into());

        for commitment in self.commitments {
            let msg = SnarkJobCommitmentPropagationChannelMsg::Commitment(commitment);
            store
                .service()
                .channel_send(peer_id, MsgId::first(), msg.into());
        }
    }
}
