use serde::{Deserialize, Serialize};

use crate::{P2pState, PeerId};

use super::P2pChannelsSnarkJobCommitmentState;

pub type P2pChannelsSnarkJobCommitmentActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a P2pChannelsSnarkJobCommitmentAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pChannelsSnarkJobCommitmentAction {
    Init(P2pChannelsSnarkJobCommitmentInitAction),
    Pending(P2pChannelsSnarkJobCommitmentPendingAction),
    Ready(P2pChannelsSnarkJobCommitmentReadyAction),
}

impl P2pChannelsSnarkJobCommitmentAction {
    pub fn peer_id(&self) -> &PeerId {
        match self {
            Self::Init(v) => &v.peer_id,
            Self::Pending(v) => &v.peer_id,
            Self::Ready(v) => &v.peer_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsSnarkJobCommitmentInitAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkJobCommitmentInitAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_ready_peer(&self.peer_id).map_or(false, |p| {
            matches!(
                &p.channels.snark_job_commitment,
                P2pChannelsSnarkJobCommitmentState::Enabled
            )
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsSnarkJobCommitmentPendingAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkJobCommitmentPendingAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_ready_peer(&self.peer_id).map_or(false, |p| {
            matches!(
                &p.channels.snark_job_commitment,
                P2pChannelsSnarkJobCommitmentState::Init { .. }
            )
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsSnarkJobCommitmentReadyAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkJobCommitmentReadyAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_ready_peer(&self.peer_id).map_or(false, |p| {
            matches!(
                &p.channels.snark_job_commitment,
                P2pChannelsSnarkJobCommitmentState::Pending { .. }
            )
        })
    }
}

// --- From<LeafAction> for Action impls.

use crate::channels::P2pChannelsAction;

impl From<P2pChannelsSnarkJobCommitmentInitAction> for crate::P2pAction {
    fn from(a: P2pChannelsSnarkJobCommitmentInitAction) -> Self {
        Self::Channels(P2pChannelsAction::SnarkJobCommitment(a.into()))
    }
}

impl From<P2pChannelsSnarkJobCommitmentPendingAction> for crate::P2pAction {
    fn from(a: P2pChannelsSnarkJobCommitmentPendingAction) -> Self {
        Self::Channels(P2pChannelsAction::SnarkJobCommitment(a.into()))
    }
}

impl From<P2pChannelsSnarkJobCommitmentReadyAction> for crate::P2pAction {
    fn from(a: P2pChannelsSnarkJobCommitmentReadyAction) -> Self {
        Self::Channels(P2pChannelsAction::SnarkJobCommitment(a.into()))
    }
}
