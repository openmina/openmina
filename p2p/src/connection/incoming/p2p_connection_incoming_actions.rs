use serde::{Deserialize, Serialize};

use openmina_core::requests::RpcId;

use crate::{webrtc, P2pAction, P2pState, PeerId};

use super::P2pConnectionIncomingInitOpts;

pub type P2pConnectionIncomingActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a P2pConnectionIncomingAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionIncomingAction {
    Init {
        opts: P2pConnectionIncomingInitOpts,
        rpc_id: Option<RpcId>,
    },
    AnswerSdpCreatePending {
        peer_id: PeerId,
    },
    AnswerSdpCreateError {
        peer_id: PeerId,
        error: String,
    },
    AnswerSdpCreateSuccess {
        peer_id: PeerId,
        sdp: String,
    },
    AnswerReady {
        peer_id: PeerId,
        answer: webrtc::Answer,
    },
    AnswerSendSuccess {
        peer_id: PeerId,
    },
    FinalizePending {
        peer_id: PeerId,
    },
    FinalizeError {
        peer_id: PeerId,
        error: String,
    },
    FinalizeSuccess {
        peer_id: PeerId,
    },
    Timeout {
        peer_id: PeerId,
    },
    Error {
        peer_id: PeerId,
        error: P2pConnectionIncomingError,
    },
    Success {
        peer_id: PeerId,
    },
    Libp2pReceived {
        peer_id: PeerId,
    },
}

impl P2pConnectionIncomingAction {
    pub fn peer_id(&self) -> Option<&PeerId> {
        match self {
            Self::Init { opts, .. } => Some(&opts.peer_id),
            Self::AnswerSdpCreatePending { peer_id }
            | Self::AnswerSdpCreateError { peer_id, .. }
            | Self::AnswerSdpCreateSuccess { peer_id, .. }
            | Self::AnswerReady { peer_id, .. }
            | Self::AnswerSendSuccess { peer_id }
            | Self::FinalizePending { peer_id }
            | Self::FinalizeError { peer_id, .. }
            | Self::FinalizeSuccess { peer_id }
            | Self::Timeout { peer_id }
            | Self::Error { peer_id, .. }
            | Self::Success { peer_id }
            | Self::Libp2pReceived { peer_id } => Some(peer_id),
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pConnectionIncomingAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        match self {
            P2pConnectionIncomingAction::Init { opts, .. } => {
                state.incoming_accept(opts.peer_id, &opts.offer).is_ok()
            }
            P2pConnectionIncomingAction::AnswerSdpCreatePending { peer_id } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                        P2pConnectionIncomingState::Init { .. },
                    )) => true,
                    _ => false,
                }),
            P2pConnectionIncomingAction::AnswerSdpCreateError { peer_id, .. } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                        P2pConnectionIncomingState::AnswerSdpCreatePending { .. },
                    )) => true,
                    _ => false,
                }),
            P2pConnectionIncomingAction::AnswerSdpCreateSuccess { peer_id, .. } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                        P2pConnectionIncomingState::AnswerSdpCreatePending { .. },
                    )) => true,
                    _ => false,
                }),
            P2pConnectionIncomingAction::AnswerReady { peer_id, .. } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                        P2pConnectionIncomingState::AnswerSdpCreateSuccess { .. },
                    )) => true,
                    _ => false,
                }),
            P2pConnectionIncomingAction::AnswerSendSuccess { peer_id } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                        P2pConnectionIncomingState::AnswerReady { .. },
                    )) => true,
                    _ => false,
                }),
            P2pConnectionIncomingAction::FinalizePending { peer_id } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                        P2pConnectionIncomingState::AnswerSendSuccess { .. },
                    )) => true,
                    _ => false,
                }),
            P2pConnectionIncomingAction::FinalizeError { peer_id, .. } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                        P2pConnectionIncomingState::FinalizePending { .. },
                    )) => true,
                    _ => false,
                }),
            P2pConnectionIncomingAction::FinalizeSuccess { peer_id } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                        P2pConnectionIncomingState::FinalizePending { .. },
                    )) => true,
                    _ => false,
                }),
            P2pConnectionIncomingAction::Timeout { peer_id } => state
                .peers
                .get(peer_id)
                .and_then(|peer| peer.status.as_connecting()?.as_incoming())
                .is_some(),
            P2pConnectionIncomingAction::Error { peer_id, error } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Incoming(s)) => match error {
                        P2pConnectionIncomingError::SdpCreateError(_) => {
                            matches!(s, P2pConnectionIncomingState::AnswerSdpCreatePending { .. })
                        }
                        P2pConnectionIncomingError::FinalizeError(_) => {
                            matches!(s, P2pConnectionIncomingState::FinalizePending { .. })
                        }
                        P2pConnectionIncomingError::Timeout => true,
                    },
                    _ => false,
                }),
            P2pConnectionIncomingAction::Success { peer_id } => {
                state
                    .peers
                    .get(peer_id)
                    .map_or(false, |peer| match &peer.status {
                        P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                            P2pConnectionIncomingState::FinalizeSuccess { .. },
                        )) => true,
                        _ => false,
                    })
            }

            P2pConnectionIncomingAction::Libp2pReceived { peer_id } => {
                state.peers.get(&peer_id).map_or(true, |peer| {
                    matches!(&peer.status, P2pPeerStatus::Disconnected { .. })
                })
            }
        }
    }
}

use crate::{
    connection::{P2pConnectionAction, P2pConnectionState},
    P2pPeerStatus,
};

use super::{P2pConnectionIncomingError, P2pConnectionIncomingState};

impl From<P2pConnectionIncomingAction> for P2pAction {
    fn from(a: P2pConnectionIncomingAction) -> Self {
        Self::Connection(P2pConnectionAction::Incoming(a))
    }
}
