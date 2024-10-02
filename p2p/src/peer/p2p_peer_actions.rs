use openmina_core::{block::ArcBlockWithHash, ActionEvent};
use serde::{Deserialize, Serialize};

use crate::{connection::outgoing::P2pConnectionOutgoingInitOpts, P2pState, PeerId};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(level = debug, fields(display(peer_id), debug(dial_opts), best_tip = display(&best_tip.hash), incoming))]
pub enum P2pPeerAction {
    /// Peer is discovered.
    #[action_event(level = debug)]
    Discovered {
        peer_id: PeerId,
        dial_opts: Option<P2pConnectionOutgoingInitOpts>,
    },
    /// Peer is ready.
    Ready { peer_id: PeerId, incoming: bool },
    /// Peer's best tip is updated.
    BestTipUpdate {
        peer_id: PeerId,
        best_tip: ArcBlockWithHash,
    },
    /// Remove peer from state
    Remove { peer_id: PeerId },
}

impl P2pPeerAction {
    pub fn peer_id(&self) -> &PeerId {
        match self {
            Self::Discovered { peer_id, .. } => peer_id,
            Self::Ready { peer_id, .. } => peer_id,
            Self::BestTipUpdate { peer_id, .. } => peer_id,
            Self::Remove { peer_id } => peer_id,
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pPeerAction {
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
        match self {
            Self::Discovered { peer_id, .. } => {
                peer_id != &state.my_id()
                    && state
                        .peers
                        .get(peer_id)
                        .map_or(true, |p| p.dial_opts.is_none())
                    && state.peers.len() < state.config.limits.max_peers_in_state()
            }
            Self::Ready { peer_id, .. } => state
                .peers
                .get(peer_id)
                .map_or(false, |p| p.status.is_connecting_success()),
            Self::BestTipUpdate { peer_id, .. } => {
                // TODO: don't enable if block inferior than existing peer's
                // best tip.
                state.get_ready_peer(peer_id).is_some()
            }
            Self::Remove { peer_id } => {
                state.peers.len() > state.config.limits.min_peers_in_state()
                    && state.peers.contains_key(peer_id)
            }
        }
    }
}
