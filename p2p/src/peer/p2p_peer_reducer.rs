use crate::{P2pPeerStatus, P2pPeerStatusReady, P2pState};

use super::{P2pPeerAction, P2pPeerActionWithMetaRef};

pub fn p2p_peer_reducer(state: &mut P2pState, action: P2pPeerActionWithMetaRef<'_>) {
    let (action, _meta) = action.split();

    match action {
        P2pPeerAction::Ready(action) => {
            let Some(peer) = state.peers.get_mut(&action.peer_id) else {
                    return;
                };
            peer.status =
                P2pPeerStatus::Ready(P2pPeerStatusReady::new(&state.config.enabled_channels));
        }
        P2pPeerAction::BestTipUpdate(action) => {
            let Some(peer) = state.get_ready_peer_mut(&action.peer_id) else {
                    return;
                };
            peer.best_tip = Some(action.best_tip.clone());
        }
    }
}
