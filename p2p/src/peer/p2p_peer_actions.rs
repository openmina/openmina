use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};

use crate::{P2pAction, P2pState, PeerId};

pub type P2pPeerActionWithMeta = redux::ActionWithMeta<P2pPeerAction>;
pub type P2pPeerActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pPeerAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pPeerAction {
    Ready(P2pPeerReadyAction),
    BestTipUpdate(P2pPeerBestTipUpdateAction),
}

impl P2pPeerAction {
    pub fn peer_id(&self) -> &PeerId {
        match self {
            Self::Ready(v) => &v.peer_id,
            Self::BestTipUpdate(v) => &v.peer_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPeerReadyAction {
    pub peer_id: PeerId,
    pub incoming: bool,
}

impl redux::EnablingCondition<P2pState> for P2pPeerReadyAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |p| p.status.is_connecting_success())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPeerBestTipUpdateAction {
    pub peer_id: PeerId,
    pub best_tip: ArcBlockWithHash,
}

impl redux::EnablingCondition<P2pState> for P2pPeerBestTipUpdateAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        // TODO(binier): don't enable if block inferrior than existing peer's
        // best tip.
        state.get_ready_peer(&self.peer_id).is_some()
    }
}

impl From<P2pPeerReadyAction> for P2pAction {
    fn from(value: P2pPeerReadyAction) -> Self {
        Self::Peer(value.into())
    }
}

impl From<P2pPeerBestTipUpdateAction> for P2pAction {
    fn from(value: P2pPeerBestTipUpdateAction) -> Self {
        Self::Peer(value.into())
    }
}
