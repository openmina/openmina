use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};

use crate::{P2pState, PeerId};

pub type P2pChannelsBestTipActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a P2pChannelsBestTipAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pChannelsBestTipAction {
    Init(P2pChannelsBestTipInitAction),
    Pending(P2pChannelsBestTipPendingAction),
    Ready(P2pChannelsBestTipReadyAction),

    RequestSend(P2pChannelsBestTipRequestSendAction),
    Received(P2pChannelsBestTipReceivedAction),

    RequestReceived(P2pChannelsBestTipRequestReceivedAction),
    ResponseSend(P2pChannelsBestTipResponseSendAction),
}

impl P2pChannelsBestTipAction {
    pub fn peer_id(&self) -> &PeerId {
        match self {
            Self::Init(v) => &v.peer_id,
            Self::Pending(v) => &v.peer_id,
            Self::Ready(v) => &v.peer_id,
            Self::RequestSend(v) => &v.peer_id,
            Self::Received(v) => &v.peer_id,
            Self::RequestReceived(v) => &v.peer_id,
            Self::ResponseSend(v) => &v.peer_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsBestTipInitAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsBestTipInitAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_ready_peer(&self.peer_id).map_or(false, |p| {
            matches!(&p.channels.best_tip, P2pChannelsBestTipState::Enabled)
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsBestTipPendingAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsBestTipPendingAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_ready_peer(&self.peer_id).map_or(false, |p| {
            matches!(&p.channels.best_tip, P2pChannelsBestTipState::Init { .. })
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsBestTipReadyAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsBestTipReadyAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_ready_peer(&self.peer_id).map_or(false, |p| {
            matches!(
                &p.channels.best_tip,
                P2pChannelsBestTipState::Pending { .. }
            )
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsBestTipRequestSendAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsBestTipRequestSendAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_ready_peer(&self.peer_id)
            .map_or(false, |p| match &p.channels.best_tip {
                P2pChannelsBestTipState::Ready { local, .. } => match local {
                    BestTipPropagationState::WaitingForRequest { .. } => true,
                    BestTipPropagationState::Responded { .. } => true,
                    _ => false,
                },
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsBestTipReceivedAction {
    pub peer_id: PeerId,
    pub best_tip: ArcBlockWithHash,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsBestTipReceivedAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        // TODO(binier): use consensus to enforce that peer doesn't send
        // us inferrior block than it has in the past.
        state
            .get_ready_peer(&self.peer_id)
            .map_or(false, |p| match &p.channels.best_tip {
                P2pChannelsBestTipState::Ready { local, .. } => match local {
                    BestTipPropagationState::Requested { .. } => true,
                    _ => false,
                },
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsBestTipRequestReceivedAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsBestTipRequestReceivedAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_ready_peer(&self.peer_id)
            .map_or(false, |p| match &p.channels.best_tip {
                P2pChannelsBestTipState::Ready { remote, .. } => match remote {
                    BestTipPropagationState::WaitingForRequest { .. } => true,
                    BestTipPropagationState::Responded { .. } => true,
                    _ => false,
                },
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsBestTipResponseSendAction {
    pub peer_id: PeerId,
    pub best_tip: ArcBlockWithHash,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsBestTipResponseSendAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_ready_peer(&self.peer_id)
            .map_or(false, |p| match &p.channels.best_tip {
                P2pChannelsBestTipState::Ready {
                    remote, last_sent, ..
                } => {
                    if !matches!(remote, BestTipPropagationState::Requested { .. }) {
                        return false;
                    }
                    last_sent
                        .as_ref()
                        .map_or(true, |sent| sent.hash != self.best_tip.hash)
                }
                _ => false,
            })
    }
}

// --- From<LeafAction> for Action impls.

use crate::channels::P2pChannelsAction;

use super::{BestTipPropagationState, P2pChannelsBestTipState};

impl From<P2pChannelsBestTipInitAction> for crate::P2pAction {
    fn from(a: P2pChannelsBestTipInitAction) -> Self {
        Self::Channels(P2pChannelsAction::BestTip(a.into()))
    }
}

impl From<P2pChannelsBestTipPendingAction> for crate::P2pAction {
    fn from(a: P2pChannelsBestTipPendingAction) -> Self {
        Self::Channels(P2pChannelsAction::BestTip(a.into()))
    }
}

impl From<P2pChannelsBestTipReadyAction> for crate::P2pAction {
    fn from(a: P2pChannelsBestTipReadyAction) -> Self {
        Self::Channels(P2pChannelsAction::BestTip(a.into()))
    }
}

impl From<P2pChannelsBestTipRequestSendAction> for crate::P2pAction {
    fn from(a: P2pChannelsBestTipRequestSendAction) -> Self {
        Self::Channels(P2pChannelsAction::BestTip(a.into()))
    }
}

impl From<P2pChannelsBestTipReceivedAction> for crate::P2pAction {
    fn from(a: P2pChannelsBestTipReceivedAction) -> Self {
        Self::Channels(P2pChannelsAction::BestTip(a.into()))
    }
}

impl From<P2pChannelsBestTipRequestReceivedAction> for crate::P2pAction {
    fn from(a: P2pChannelsBestTipRequestReceivedAction) -> Self {
        Self::Channels(P2pChannelsAction::BestTip(a.into()))
    }
}

impl From<P2pChannelsBestTipResponseSendAction> for crate::P2pAction {
    fn from(a: P2pChannelsBestTipResponseSendAction) -> Self {
        Self::Channels(P2pChannelsAction::BestTip(a.into()))
    }
}
