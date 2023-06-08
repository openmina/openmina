use redux::ActionMeta;

use crate::channels::{ChannelId, MsgId, P2pChannelsService};

use super::{
    BestTipPropagationChannelMsg, P2pChannelsBestTipInitAction, P2pChannelsBestTipPendingAction,
    P2pChannelsBestTipReadyAction, P2pChannelsBestTipReceivedAction,
    P2pChannelsBestTipRequestReceivedAction, P2pChannelsBestTipRequestSendAction,
    P2pChannelsBestTipResponseSendAction,
};

impl P2pChannelsBestTipInitAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
        P2pChannelsBestTipPendingAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        store
            .service()
            .channel_open(peer_id, ChannelId::BestTipPropagation);
        store.dispatch(P2pChannelsBestTipPendingAction { peer_id });
    }
}

impl P2pChannelsBestTipReadyAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
        P2pChannelsBestTipRequestSendAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        store.dispatch(P2pChannelsBestTipRequestSendAction { peer_id });
    }
}

impl P2pChannelsBestTipRequestSendAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        let msg = BestTipPropagationChannelMsg::GetNext;
        store
            .service()
            .channel_send(self.peer_id, MsgId::first(), msg.into());
    }
}

impl P2pChannelsBestTipReceivedAction {
    pub fn effects<Store, S>(&self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
        P2pChannelsBestTipRequestSendAction: redux::EnablingCondition<S>,
        P2pChannelsBestTipRequestReceivedAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        store.dispatch(P2pChannelsBestTipRequestSendAction { peer_id });
        if store.state().is_libp2p_peer(&peer_id) {
            store.dispatch(P2pChannelsBestTipRequestReceivedAction { peer_id });
        }
    }
}

impl P2pChannelsBestTipResponseSendAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        let msg = BestTipPropagationChannelMsg::BestTip(self.best_tip.block);
        store
            .service()
            .channel_send(self.peer_id, MsgId::first(), msg.into());
    }
}
