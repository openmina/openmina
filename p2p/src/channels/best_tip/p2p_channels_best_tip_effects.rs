use redux::ActionMeta;

use crate::{
    channels::{ChannelId, MsgId, P2pChannelsService},
    peer::P2pPeerBestTipUpdateAction,
};

use super::{BestTipPropagationChannelMsg, P2pChannelsBestTipAction};

impl P2pChannelsBestTipAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
        P2pChannelsBestTipAction: redux::EnablingCondition<S>,
        P2pPeerBestTipUpdateAction: redux::EnablingCondition<S>,
    {
        match self {
            P2pChannelsBestTipAction::Init { peer_id } => {
                store
                    .service()
                    .channel_open(peer_id, ChannelId::BestTipPropagation);
                store.dispatch(P2pChannelsBestTipAction::Pending { peer_id });
            }
            P2pChannelsBestTipAction::Ready { peer_id } => {
                store.dispatch(P2pChannelsBestTipAction::RequestSend { peer_id });
                if store.state().is_libp2p_peer(&peer_id) {
                    store.dispatch(P2pChannelsBestTipAction::RequestReceived { peer_id });
                }
            }
            P2pChannelsBestTipAction::RequestSend { peer_id } => {
                let msg = BestTipPropagationChannelMsg::GetNext;
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());
            }
            P2pChannelsBestTipAction::Received { peer_id, best_tip } => {
                store.dispatch(P2pPeerBestTipUpdateAction {
                    peer_id,
                    best_tip: best_tip.clone(),
                });
                store.dispatch(P2pChannelsBestTipAction::RequestSend { peer_id });
                if store.state().is_libp2p_peer(&peer_id) {
                    store.dispatch(P2pChannelsBestTipAction::RequestReceived { peer_id });
                }
            }
            P2pChannelsBestTipAction::ResponseSend { peer_id, best_tip } => {
                let msg = BestTipPropagationChannelMsg::BestTip(best_tip.block);
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());
            }
            P2pChannelsBestTipAction::Pending { .. } => {}
            P2pChannelsBestTipAction::RequestReceived { .. } => {}
        }
    }
}
