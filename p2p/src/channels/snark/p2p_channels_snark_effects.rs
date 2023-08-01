use redux::ActionMeta;

use crate::channels::{ChannelId, MsgId, P2pChannelsService};

use super::{
    P2pChannelsSnarkInitAction, P2pChannelsSnarkLibp2pBroadcastAction,
    P2pChannelsSnarkPendingAction, P2pChannelsSnarkReadyAction, P2pChannelsSnarkReceivedAction,
    P2pChannelsSnarkRequestSendAction, P2pChannelsSnarkResponseSendAction,
    SnarkPropagationChannelMsg,
};

impl P2pChannelsSnarkInitAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
        P2pChannelsSnarkPendingAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        store
            .service()
            .channel_open(peer_id, ChannelId::SnarkPropagation);
        store.dispatch(P2pChannelsSnarkPendingAction { peer_id });
    }
}

impl P2pChannelsSnarkReadyAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
        P2pChannelsSnarkRequestSendAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        let limit = 16;
        store.dispatch(P2pChannelsSnarkRequestSendAction { peer_id, limit });
    }
}

impl P2pChannelsSnarkRequestSendAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        let peer_id = self.peer_id;
        let limit = self.limit;
        let msg = SnarkPropagationChannelMsg::GetNext { limit };
        store
            .service()
            .channel_send(peer_id, MsgId::first(), msg.into());
    }
}

impl P2pChannelsSnarkReceivedAction {
    pub fn effects<Store, S>(&self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
        P2pChannelsSnarkRequestSendAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        let limit = 16;
        store.dispatch(P2pChannelsSnarkRequestSendAction { peer_id, limit });
    }
}

impl P2pChannelsSnarkResponseSendAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        let peer_id = self.peer_id;
        let msg = SnarkPropagationChannelMsg::WillSend {
            count: self.snarks.len() as u8,
        };
        store
            .service()
            .channel_send(peer_id, MsgId::first(), msg.into());

        for snark in self.snarks {
            let msg = SnarkPropagationChannelMsg::Snark(snark);
            store
                .service()
                .channel_send(peer_id, MsgId::first(), msg.into());
        }
    }
}

impl P2pChannelsSnarkLibp2pBroadcastAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        store.service().libp2p_broadcast_snark(self.snark);
    }
}
