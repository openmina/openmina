use redux::Timestamp;

use crate::{P2pPeerState, P2pPeerStatus, P2pPeerStatusReady, P2pState};

use super::{P2pPeerAction, P2pPeerActionWithMetaRef};

pub fn p2p_peer_reducer(state: &mut P2pState, action: P2pPeerActionWithMetaRef<'_>) {
    let (action, meta) = action.split();

    match action {
        P2pPeerAction::Discovered { peer_id, dial_opts } => {
            let peer_state = state.peers.entry(*peer_id).or_insert_with(|| P2pPeerState {
                is_libp2p: true,
                dial_opts: None,
                status: P2pPeerStatus::Disconnected {
                    time: Timestamp::ZERO,
                },
            });
            if let Some(dial_opts) = dial_opts {
                peer_state.dial_opts.get_or_insert(dial_opts.clone());
            }
        }
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
