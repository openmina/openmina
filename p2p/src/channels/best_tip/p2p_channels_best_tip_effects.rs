use redux::ActionMeta;

#[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
use crate::P2pNetworkPubsubAction;
use crate::{
    channels::{ChannelId, MsgId, P2pChannelsService},
    peer::P2pPeerAction,
};

use super::{BestTipPropagationChannelMsg, P2pChannelsBestTipAction};

impl P2pChannelsBestTipAction {
    pub fn effects<Store, S>(self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
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
                #[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
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
                store.dispatch(P2pPeerAction::BestTipUpdate {
                    peer_id,
                    best_tip: best_tip.clone(),
                });
                store.dispatch(P2pChannelsBestTipAction::RequestSend { peer_id });
            }
            P2pChannelsBestTipAction::ResponseSend { peer_id, best_tip } => {
                if !store.state().is_libp2p_peer(&peer_id) {
                    let msg = BestTipPropagationChannelMsg::BestTip(best_tip.block);
                    store
                        .service()
                        .channel_send(peer_id, MsgId::first(), msg.into());
                    return;
                }
                #[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
                {
                    use mina_p2p_messages::gossip::GossipNetMessageV2;
                    let block = (*best_tip.block).clone();
                    let message = Box::new(GossipNetMessageV2::NewState(block));
                    // TODO(vlad): `P2pChannelsBestTipAction::ResponseSend`
                    // action is dispatched for each peer. So `P2pNetworkPubsubAction::Broadcast`
                    // will be called many times causing many duplicate
                    // broadcasts. Either in pubsub state machine, we
                    // need to filter out duplicate messages, or better,
                    // have a simple action to send pubsub message to a
                    // specific peer instead of sending to everyone.
                    // That way we can avoid duplicate state, since we
                    // already store last sent best tip here and we make
                    // sure we don't send same block to same peer again.
                    store.dispatch(P2pNetworkPubsubAction::Broadcast { message });
                }
            }
            P2pChannelsBestTipAction::Pending { .. } => {}
            P2pChannelsBestTipAction::RequestReceived { .. } => {}
        }
    }
}
