use crate::{P2pPeerStatus, P2pPeerStatusReady, P2pState};

use super::{P2pPeerAction, P2pPeerActionWithMetaRef};

pub fn p2p_peer_reducer(state: &mut P2pState, action: P2pPeerActionWithMetaRef<'_>) {
    let (action, meta) = action.split();

    match action {
        P2pPeerAction::Ready { peer_id, incoming } => {
            let Some(peer) = state.peers.get_mut(peer_id) else {
                return;
            };
            peer.status = P2pPeerStatus::Ready(P2pPeerStatusReady::new(
                *incoming,
                meta.time(),
                &state.config.enabled_channels,
            ));
        }
        P2pPeerAction::BestTipUpdate { peer_id, best_tip } => {
            let Some(peer) = state.get_ready_peer_mut(peer_id) else {
                return;
            };
            peer.best_tip = Some(best_tip.clone());
        }
    }
}
