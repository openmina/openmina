use super::P2pChannelsBestTipEffectfulAction;
use crate::channels::{
    best_tip::{BestTipPropagationChannelMsg, P2pChannelsBestTipAction},
    ChannelId, MsgId, P2pChannelsService,
};
use redux::ActionMeta;

impl P2pChannelsBestTipEffectfulAction {
    pub fn effects<Store, S>(self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        match self {
            P2pChannelsBestTipEffectfulAction::Init { peer_id } => {
                store
                    .service()
                    .channel_open(peer_id, ChannelId::BestTipPropagation);
                store.dispatch(P2pChannelsBestTipAction::Pending { peer_id });
            }
            P2pChannelsBestTipEffectfulAction::RequestSend { peer_id } => {
                let msg = BestTipPropagationChannelMsg::GetNext;
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());
            }
            P2pChannelsBestTipEffectfulAction::ResponseSend { peer_id, best_tip } => {
                let msg = BestTipPropagationChannelMsg::BestTip(best_tip.block);
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());
            }
        }
    }
}
