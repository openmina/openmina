use openmina_core::error;

use crate::{
    P2pLibP2pPeerState, P2pPeerState, P2pPeerStatus, P2pPeerStatusReady, P2pState,
    P2pWebRTCPeerState,
};

use super::{P2pPeerAction, P2pPeerActionWithMetaRef};

pub fn p2p_peer_reducer(state: &mut P2pState, action: P2pPeerActionWithMetaRef<'_>) {
    let (action, meta) = action.split();

    match action {
        P2pPeerAction::AddLibP2p(action) => {
            state.peers.insert(
                action.peer_id.clone(),
                P2pPeerState::Libp2p(P2pLibP2pPeerState {
                    dial_opts: action.addrs.clone(),
                    status: Default::default(),
                }),
            );
        }
        P2pPeerAction::AddWebRTC(action) => {
            state.peers.insert(
                action.peer_id.clone(),
                P2pPeerState::WebRTC(P2pWebRTCPeerState {
                    dial_opts: Some(action.addr.clone()),
                    status: Default::default(),
                }),
            );
        }
        P2pPeerAction::Reconnect(_) => {
            // noop
        }
        P2pPeerAction::Ready(action) => {
            let Some(peer) = state.peers.get_mut(&action.peer_id) else {
                error!(meta.time(); "peer {} not found", action.peer_id);
                return;
            };
            match peer {
                P2pPeerState::Default => {
                    error!(meta.time(); "peer {} is not initialized", action.peer_id);
                }
                P2pPeerState::WebRTC(P2pWebRTCPeerState { status, .. }) => {
                    *status = P2pPeerStatus::Ready(P2pPeerStatusReady::new(
                        action.incoming,
                        meta.time(),
                        &state.config.enabled_channels,
                    ));
                }
                P2pPeerState::Libp2p(P2pLibP2pPeerState { status, .. }) => {
                    *status = P2pPeerStatus::Ready(P2pPeerStatusReady::new(
                        action.incoming,
                        meta.time(),
                        &state.config.enabled_channels,
                    ));
                }
            }
        }
        P2pPeerAction::BestTipUpdate(action) => {
            let Some(peer) = state.get_ready_peer_mut(&action.peer_id) else {
                return;
            };
            peer.best_tip = Some(action.best_tip.clone());
        }
    }
}
