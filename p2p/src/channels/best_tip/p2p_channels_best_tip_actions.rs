use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};

use crate::{
    channels::{best_tip::P2pChannelsBestTipState, P2pChannelsAction},
    P2pState, PeerId,
};

use super::BestTipPropagationState;

pub type P2pChannelsBestTipActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a P2pChannelsBestTipAction>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2pChannelsBestTipAction {
    Init {
        peer_id: PeerId,
    },
    Pending {
        peer_id: PeerId,
    },
    Ready {
        peer_id: PeerId,
    },
    RequestSend {
        peer_id: PeerId,
    },
    Received {
        peer_id: PeerId,
        best_tip: ArcBlockWithHash,
    },
    RequestReceived {
        peer_id: PeerId,
    },
    ResponseSend {
        peer_id: PeerId,
        best_tip: ArcBlockWithHash,
    },
}

impl P2pChannelsBestTipAction {
    pub fn peer_id(&self) -> &PeerId {
        match self {
            Self::Init { peer_id }
            | Self::Pending { peer_id }
            | Self::Ready { peer_id }
            | Self::RequestSend { peer_id }
            | Self::Received { peer_id, .. }
            | Self::RequestReceived { peer_id }
            | Self::ResponseSend { peer_id, .. } => peer_id,
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pChannelsBestTipAction {
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
        match self {
            P2pChannelsBestTipAction::Init { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(&p.channels.best_tip, P2pChannelsBestTipState::Enabled)
                })
            }
            P2pChannelsBestTipAction::Pending { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(&p.channels.best_tip, P2pChannelsBestTipState::Init { .. })
                })
            }
            P2pChannelsBestTipAction::Ready { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(
                        &p.channels.best_tip,
                        P2pChannelsBestTipState::Pending { .. }
                    )
                })
            }
            P2pChannelsBestTipAction::RequestSend { peer_id } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.best_tip {
                    P2pChannelsBestTipState::Ready { local, .. } => match local {
                        BestTipPropagationState::WaitingForRequest { .. } => true,
                        BestTipPropagationState::Responded { .. } => true,
                        _ => false,
                    },
                    _ => false,
                }),
            P2pChannelsBestTipAction::Received { peer_id, .. } => {
                // TODO(binier): use consensus to enforce that peer doesn't send
                // us inferior block than it has in the past.
                state
                    .get_ready_peer(peer_id)
                    .map_or(false, |p| match &p.channels.best_tip {
                        P2pChannelsBestTipState::Ready { local, .. } => match local {
                            BestTipPropagationState::Requested { .. } => true,
                            _ => false,
                        },
                        _ => false,
                    })
            }
            P2pChannelsBestTipAction::RequestReceived { peer_id } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.best_tip {
                    P2pChannelsBestTipState::Ready { remote, .. } => match remote {
                        BestTipPropagationState::WaitingForRequest { .. } => true,
                        BestTipPropagationState::Responded { .. } => true,
                        _ => false,
                    },
                    _ => false,
                }),
            P2pChannelsBestTipAction::ResponseSend { peer_id, best_tip } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.best_tip {
                    P2pChannelsBestTipState::Ready {
                        remote, last_sent, ..
                    } => {
                        if !matches!(remote, BestTipPropagationState::Requested { .. }) {
                            return false;
                        }
                        last_sent
                            .as_ref()
                            .map_or(true, |sent| sent.hash != best_tip.hash)
                    }
                    _ => false,
                }),
        }
    }
}

impl From<P2pChannelsBestTipAction> for crate::P2pAction {
    fn from(action: P2pChannelsBestTipAction) -> Self {
        Self::Channels(P2pChannelsAction::BestTip(action))
    }
}
