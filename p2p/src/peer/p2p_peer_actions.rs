use openmina_core::{action_info, block::ArcBlockWithHash, log::ActionEvent};
use serde::{Deserialize, Serialize};

use crate::{P2pState, PeerId};

pub type P2pPeerActionWithMeta = redux::ActionWithMeta<P2pPeerAction>;
pub type P2pPeerActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pPeerAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pPeerAction {
    Ready {
        peer_id: PeerId,
        incoming: bool,
    },
    BestTipUpdate {
        peer_id: PeerId,
        best_tip: ArcBlockWithHash,
    },
}

impl P2pPeerAction {
    pub fn peer_id(&self) -> &PeerId {
        match self {
            Self::Ready { peer_id, .. } => peer_id,
            Self::BestTipUpdate { peer_id, .. } => peer_id,
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pPeerAction {
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
        match self {
            Self::Ready { peer_id, .. } => state
                .peers
                .get(peer_id)
                .map_or(false, |p| p.status.is_connecting_success()),
            Self::BestTipUpdate { peer_id, .. } => {
                // TODO: don't enable if block inferior than existing peer's
                // best tip.
                state.get_ready_peer(peer_id).is_some()
            }
        }
    }
}

impl ActionEvent for P2pPeerAction {
    fn action_event<T>(&self, context: &T)
    where
        T: openmina_core::log::EventContext,
    {
        match self {
            P2pPeerAction::Ready { peer_id, incoming } => action_info!(
                context,
                summary = "Peer is ready",
                peer_id = display(peer_id),
                incoming,
            ),
            P2pPeerAction::BestTipUpdate { peer_id, best_tip } => action_info!(
                context,
                summary = "Best tip updated",
                peer_id = display(peer_id),
                best_tip = display(&best_tip.hash),
            ),
        }
    }
}
