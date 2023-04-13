use serde::{Deserialize, Serialize};

use crate::{P2pState, PeerId};

use super::{
    P2pChannelsSnarkJobCommitmentState, SnarkJobCommitment, SnarkJobCommitmentPropagationState,
};

pub type P2pChannelsSnarkJobCommitmentActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a P2pChannelsSnarkJobCommitmentAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pChannelsSnarkJobCommitmentAction {
    Init(P2pChannelsSnarkJobCommitmentInitAction),
    Pending(P2pChannelsSnarkJobCommitmentPendingAction),
    Ready(P2pChannelsSnarkJobCommitmentReadyAction),

    RequestSend(P2pChannelsSnarkJobCommitmentRequestSendAction),
    PromiseReceived(P2pChannelsSnarkJobCommitmentPromiseReceivedAction),
    Received(P2pChannelsSnarkJobCommitmentReceivedAction),

    RequestReceived(P2pChannelsSnarkJobCommitmentRequestReceivedAction),
    ResponseSend(P2pChannelsSnarkJobCommitmentResponseSendAction),
}

impl P2pChannelsSnarkJobCommitmentAction {
    pub fn peer_id(&self) -> &PeerId {
        match self {
            Self::Init(v) => &v.peer_id,
            Self::Pending(v) => &v.peer_id,
            Self::Ready(v) => &v.peer_id,
            Self::RequestSend(v) => &v.peer_id,
            Self::PromiseReceived(v) => &v.peer_id,
            Self::Received(v) => &v.peer_id,
            Self::RequestReceived(v) => &v.peer_id,
            Self::ResponseSend(v) => &v.peer_id,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsSnarkJobCommitmentRequestSendAction {
    pub peer_id: PeerId,
    pub limit: u8,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkJobCommitmentRequestSendAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_ready_peer(&self.peer_id).map_or(false, |p| {
            match &p.channels.snark_job_commitment {
                P2pChannelsSnarkJobCommitmentState::Ready { local, .. } => match local {
                    SnarkJobCommitmentPropagationState::WaitingForRequest { .. } => true,
                    SnarkJobCommitmentPropagationState::Responded { .. } => true,
                    _ => false,
                },
                _ => false,
            }
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsSnarkJobCommitmentPromiseReceivedAction {
    pub peer_id: PeerId,
    pub promised_count: u8,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkJobCommitmentPromiseReceivedAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_ready_peer(&self.peer_id).map_or(false, |p| {
            match &p.channels.snark_job_commitment {
                P2pChannelsSnarkJobCommitmentState::Ready { local, .. } => match local {
                    SnarkJobCommitmentPropagationState::Requested {
                        requested_limit, ..
                    } => self.promised_count <= *requested_limit,
                    _ => false,
                },
                _ => false,
            }
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsSnarkJobCommitmentReceivedAction {
    pub peer_id: PeerId,
    pub commitment: SnarkJobCommitment,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkJobCommitmentReceivedAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_ready_peer(&self.peer_id).map_or(false, |p| {
            match &p.channels.snark_job_commitment {
                P2pChannelsSnarkJobCommitmentState::Ready { local, .. } => match local {
                    SnarkJobCommitmentPropagationState::Responding { .. } => true,
                    _ => false,
                },
                _ => false,
            }
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsSnarkJobCommitmentRequestReceivedAction {
    pub peer_id: PeerId,
    pub limit: u8,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkJobCommitmentRequestReceivedAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        self.limit > 0
            && state.get_ready_peer(&self.peer_id).map_or(false, |p| {
                match &p.channels.snark_job_commitment {
                    P2pChannelsSnarkJobCommitmentState::Ready { remote, .. } => match remote {
                        SnarkJobCommitmentPropagationState::WaitingForRequest { .. } => true,
                        SnarkJobCommitmentPropagationState::Responded { .. } => true,
                        _ => false,
                    },
                    _ => false,
                }
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsSnarkJobCommitmentResponseSendAction {
    pub peer_id: PeerId,
    pub commitments: Vec<SnarkJobCommitment>,
    pub first_index: u64,
    pub last_index: u64,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkJobCommitmentResponseSendAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        !self.commitments.is_empty()
            && self.first_index < self.last_index
            && state.get_ready_peer(&self.peer_id).map_or(false, |p| {
                match &p.channels.snark_job_commitment {
                    P2pChannelsSnarkJobCommitmentState::Ready {
                        remote,
                        next_send_index,
                        ..
                    } => {
                        if self.first_index < *next_send_index {
                            return false;
                        }
                        match remote {
                            SnarkJobCommitmentPropagationState::Requested {
                                requested_limit,
                                ..
                            } => self.commitments.len() <= *requested_limit as usize,
                            _ => false,
                        }
                    }
                    _ => false,
                }
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

impl From<P2pChannelsSnarkJobCommitmentRequestSendAction> for crate::P2pAction {
    fn from(a: P2pChannelsSnarkJobCommitmentRequestSendAction) -> Self {
        Self::Channels(P2pChannelsAction::SnarkJobCommitment(a.into()))
    }
}

impl From<P2pChannelsSnarkJobCommitmentPromiseReceivedAction> for crate::P2pAction {
    fn from(a: P2pChannelsSnarkJobCommitmentPromiseReceivedAction) -> Self {
        Self::Channels(P2pChannelsAction::SnarkJobCommitment(a.into()))
    }
}

impl From<P2pChannelsSnarkJobCommitmentReceivedAction> for crate::P2pAction {
    fn from(a: P2pChannelsSnarkJobCommitmentReceivedAction) -> Self {
        Self::Channels(P2pChannelsAction::SnarkJobCommitment(a.into()))
    }
}

impl From<P2pChannelsSnarkJobCommitmentRequestReceivedAction> for crate::P2pAction {
    fn from(a: P2pChannelsSnarkJobCommitmentRequestReceivedAction) -> Self {
        Self::Channels(P2pChannelsAction::SnarkJobCommitment(a.into()))
    }
}

impl From<P2pChannelsSnarkJobCommitmentResponseSendAction> for crate::P2pAction {
    fn from(a: P2pChannelsSnarkJobCommitmentResponseSendAction) -> Self {
        Self::Channels(P2pChannelsAction::SnarkJobCommitment(a.into()))
    }
}
