use serde::{Deserialize, Serialize};

use crate::{P2pState, PeerId};

use super::{
    P2pChannelsSnarkJobCommitmentState, SnarkJobCommitment, SnarkJobCommitmentPropagationState,
};

pub type P2pChannelsSnarkJobCommitmentActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a P2pChannelsSnarkJobCommitmentAction>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2pChannelsSnarkJobCommitmentAction {
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
        limit: u8,
    },
    PromiseReceived {
        peer_id: PeerId,
        promised_count: u8,
    },
    Received {
        peer_id: PeerId,
        commitment: SnarkJobCommitment,
    },
    RequestReceived {
        peer_id: PeerId,
        limit: u8,
    },
    ResponseSend {
        peer_id: PeerId,
        commitments: Vec<SnarkJobCommitment>,
        first_index: u64,
        last_index: u64,
    },
}

impl P2pChannelsSnarkJobCommitmentAction {
    pub fn peer_id(&self) -> &PeerId {
        match self {
            Self::Init { peer_id }
            | Self::Pending { peer_id }
            | Self::Ready { peer_id }
            | Self::RequestSend { peer_id, .. }
            | Self::PromiseReceived { peer_id, .. }
            | Self::Received { peer_id, .. }
            | Self::RequestReceived { peer_id, .. }
            | Self::ResponseSend { peer_id, .. } => peer_id,
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkJobCommitmentAction {
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
        match self {
            P2pChannelsSnarkJobCommitmentAction::Init { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(
                        &p.channels.snark_job_commitment,
                        P2pChannelsSnarkJobCommitmentState::Enabled
                    )
                })
            }
            P2pChannelsSnarkJobCommitmentAction::Pending { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(
                        &p.channels.snark_job_commitment,
                        P2pChannelsSnarkJobCommitmentState::Init { .. }
                    )
                })
            }
            P2pChannelsSnarkJobCommitmentAction::Ready { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(
                        &p.channels.snark_job_commitment,
                        P2pChannelsSnarkJobCommitmentState::Pending { .. }
                    )
                })
            }
            P2pChannelsSnarkJobCommitmentAction::RequestSend { peer_id, .. } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.snark_job_commitment {
                    P2pChannelsSnarkJobCommitmentState::Ready { local, .. } => match local {
                        SnarkJobCommitmentPropagationState::WaitingForRequest { .. } => true,
                        SnarkJobCommitmentPropagationState::Responded { .. } => true,
                        _ => false,
                    },
                    _ => false,
                }),
            P2pChannelsSnarkJobCommitmentAction::PromiseReceived {
                peer_id,
                promised_count,
            } => state.get_ready_peer(peer_id).map_or(false, |p| {
                match &p.channels.snark_job_commitment {
                    P2pChannelsSnarkJobCommitmentState::Ready { local, .. } => match local {
                        SnarkJobCommitmentPropagationState::Requested {
                            requested_limit, ..
                        } => *promised_count > 0 && promised_count <= requested_limit,
                        _ => false,
                    },
                    _ => false,
                }
            }),
            P2pChannelsSnarkJobCommitmentAction::Received { peer_id, .. } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.snark_job_commitment {
                    P2pChannelsSnarkJobCommitmentState::Ready { local, .. } => match local {
                        SnarkJobCommitmentPropagationState::Responding { .. } => true,
                        _ => false,
                    },
                    _ => false,
                }),
            P2pChannelsSnarkJobCommitmentAction::RequestReceived { peer_id, limit } => {
                *limit > 0
                    && state.get_ready_peer(peer_id).map_or(false, |p| {
                        match &p.channels.snark_job_commitment {
                            P2pChannelsSnarkJobCommitmentState::Ready { remote, .. } => {
                                match remote {
                                    SnarkJobCommitmentPropagationState::WaitingForRequest {
                                        ..
                                    } => true,
                                    SnarkJobCommitmentPropagationState::Responded { .. } => true,
                                    _ => false,
                                }
                            }
                            _ => false,
                        }
                    })
            }
            P2pChannelsSnarkJobCommitmentAction::ResponseSend {
                peer_id,
                commitments,
                first_index,
                last_index,
            } => {
                !commitments.is_empty()
                    && first_index < last_index
                    && state.get_ready_peer(peer_id).map_or(false, |p| {
                        match &p.channels.snark_job_commitment {
                            P2pChannelsSnarkJobCommitmentState::Ready {
                                remote,
                                next_send_index,
                                ..
                            } => {
                                if first_index < next_send_index {
                                    return false;
                                }
                                match remote {
                                    SnarkJobCommitmentPropagationState::Requested {
                                        requested_limit,
                                        ..
                                    } => commitments.len() <= *requested_limit as usize,
                                    _ => false,
                                }
                            }
                            _ => false,
                        }
                    })
            }
        }
    }
}

use crate::channels::P2pChannelsAction;

impl From<P2pChannelsSnarkJobCommitmentAction> for crate::P2pAction {
    fn from(action: P2pChannelsSnarkJobCommitmentAction) -> Self {
        Self::Channels(P2pChannelsAction::SnarkJobCommitment(action))
    }
}
