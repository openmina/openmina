use redux::ActionMeta;

use crate::channels::{ChannelId, MsgId, P2pChannelsService};

use super::{
    P2pChannelsRpcInitAction, P2pChannelsRpcPendingAction, P2pChannelsRpcRequestSendAction,
    P2pChannelsRpcResponseSendAction, RpcChannelMsg,
};

impl P2pChannelsRpcInitAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
        P2pChannelsRpcPendingAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        store.service().channel_open(peer_id, ChannelId::Rpc);
        store.dispatch(P2pChannelsRpcPendingAction { peer_id });
    }
}

impl P2pChannelsRpcRequestSendAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        let msg = RpcChannelMsg::Request(self.id, self.request);
        store
            .service()
            .channel_send(self.peer_id, MsgId::first(), msg.into());
    }
}

impl P2pChannelsRpcResponseSendAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        let msg = RpcChannelMsg::Response(self.id, self.response);
        store
            .service()
            .channel_send(self.peer_id, MsgId::first(), msg.into());
    }
}
