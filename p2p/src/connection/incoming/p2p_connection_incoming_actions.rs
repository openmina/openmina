use super::{
    P2pConnectionIncomingError, P2pConnectionIncomingInitOpts, P2pConnectionIncomingState,
};
use crate::{
    connection::{P2pConnectionAction, P2pConnectionState},
    webrtc, P2pAction, P2pPeerStatus, P2pState, PeerId,
};
use openmina_core::{requests::RpcId, ActionEvent};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(debug(opts), display(peer_id), display(error)))]
pub enum P2pConnectionIncomingAction {
    /// Incoming connection is initialized.
    Init {
        opts: P2pConnectionIncomingInitOpts,
        rpc_id: Option<RpcId>,
    },
    #[action_event(level = trace)]
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
        answer: Box<webrtc::Answer>,
    },
    AnswerSendSuccess {
        peer_id: PeerId,
    },
    /// Pending incoming connection finalization.
    #[action_event(level = trace)]
    FinalizePending {
        peer_id: PeerId,
    },
    /// Error finalizing incoming connection.
    FinalizeError {
        peer_id: PeerId,
        error: String,
    },
    /// Incoming connection finalized.
    FinalizeSuccess {
        peer_id: PeerId,
        remote_auth: webrtc::ConnectionAuthEncrypted,
    },
    /// Timeout establishing incoming connection.
    Timeout {
        peer_id: PeerId,
    },
    /// Error establishing incoming connection.
    #[action_event(level = warn, fields(display(peer_id), display(error)))]
    Error {
        peer_id: PeerId,
        error: P2pConnectionIncomingError,
    },
    /// Incoming connection is successful.
    #[action_event(level = info)]
    Success {
        peer_id: PeerId,
    },
    /// Detected incoming connection from this peer.
    FinalizePendingLibp2p {
        peer_id: PeerId,
        addr: SocketAddr,
    },
    /// Incoming libp2p connection is successful.
    Libp2pReceived {
        peer_id: PeerId,
    },
}

impl P2pConnectionIncomingAction {
    pub fn peer_id(&self) -> &PeerId {
        match self {
            Self::Init { opts, .. } => &opts.peer_id,
            Self::AnswerSdpCreatePending { peer_id }
            | Self::AnswerSdpCreateError { peer_id, .. }
            | Self::AnswerSdpCreateSuccess { peer_id, .. }
            | Self::AnswerReady { peer_id, .. }
            | Self::AnswerSendSuccess { peer_id }
            | Self::FinalizePending { peer_id }
            | Self::FinalizeError { peer_id, .. }
            | Self::FinalizeSuccess { peer_id, .. }
            | Self::Timeout { peer_id }
            | Self::Error { peer_id, .. }
            | Self::Success { peer_id }
            | Self::FinalizePendingLibp2p { peer_id, .. }
            | Self::Libp2pReceived { peer_id } => peer_id,
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pConnectionIncomingAction {
    fn is_enabled(&self, state: &P2pState, time: redux::Timestamp) -> bool {
        match self {
            P2pConnectionIncomingAction::Init { opts, .. } => {
                state.incoming_accept(opts.peer_id, &opts.offer).is_ok()
            }
            P2pConnectionIncomingAction::AnswerSdpCreatePending { peer_id } => {
                state.peers.get(peer_id).is_some_and(|peer| {
                    matches!(
                        &peer.status,
                        P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                            P2pConnectionIncomingState::Init { .. },
                        ))
                    )
                })
            }
            P2pConnectionIncomingAction::AnswerSdpCreateError { peer_id, .. } => {
                state.peers.get(peer_id).is_some_and(|peer| {
                    matches!(
                        &peer.status,
                        P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                            P2pConnectionIncomingState::AnswerSdpCreatePending { .. },
                        ))
                    )
                })
            }
            P2pConnectionIncomingAction::AnswerSdpCreateSuccess { peer_id, .. } => {
                state.peers.get(peer_id).is_some_and(|peer| {
                    matches!(
                        &peer.status,
                        P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                            P2pConnectionIncomingState::AnswerSdpCreatePending { .. },
                        ))
                    )
                })
            }
            P2pConnectionIncomingAction::AnswerReady { peer_id, .. } => {
                state.peers.get(peer_id).is_some_and(|peer| {
                    matches!(
                        &peer.status,
                        P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                            P2pConnectionIncomingState::AnswerSdpCreateSuccess { .. },
                        ))
                    )
                })
            }
            P2pConnectionIncomingAction::AnswerSendSuccess { peer_id } => {
                state.peers.get(peer_id).is_some_and(|peer| {
                    matches!(
                        &peer.status,
                        P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                            P2pConnectionIncomingState::AnswerReady { .. },
                        ))
                    )
                })
            }
            P2pConnectionIncomingAction::FinalizePending { peer_id } => {
                state.peers.get(peer_id).is_some_and(|peer| {
                    matches!(
                        &peer.status,
                        P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                            P2pConnectionIncomingState::AnswerSendSuccess { .. },
                        ))
                    )
                })
            }
            P2pConnectionIncomingAction::FinalizeError { peer_id, .. } => {
                state.peers.get(peer_id).is_some_and(|peer| {
                    matches!(
                        &peer.status,
                        P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                            P2pConnectionIncomingState::FinalizePending { .. },
                        ))
                    )
                })
            }
            P2pConnectionIncomingAction::FinalizeSuccess { peer_id, .. } => {
                state.peers.get(peer_id).is_some_and(|peer| {
                    matches!(
                        &peer.status,
                        P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                            P2pConnectionIncomingState::FinalizePending { .. },
                        ))
                    )
                })
            }
            P2pConnectionIncomingAction::Timeout { peer_id } => state
                .peers
                .get(peer_id)
                .and_then(|peer| peer.status.as_connecting()?.as_incoming())
                .is_some_and(|s| s.is_timed_out(time, &state.config.timeouts)),
            P2pConnectionIncomingAction::Error { peer_id, error } => state
                .peers
                .get(peer_id)
                .is_some_and(|peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Incoming(s)) => match error {
                        P2pConnectionIncomingError::SdpCreateError(_) => {
                            matches!(s, P2pConnectionIncomingState::AnswerSdpCreatePending { .. })
                        }
                        P2pConnectionIncomingError::FinalizeError(_) => {
                            matches!(s, P2pConnectionIncomingState::FinalizePending { .. })
                        }
                        P2pConnectionIncomingError::ConnectionAuthError => {
                            matches!(s, P2pConnectionIncomingState::FinalizeSuccess { .. })
                        }
                        P2pConnectionIncomingError::Timeout => true,
                    },
                    _ => false,
                }),
            P2pConnectionIncomingAction::Success { peer_id } => {
                state.peers.get(peer_id).is_some_and(|peer| {
                    matches!(
                        &peer.status,
                        P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                            P2pConnectionIncomingState::FinalizeSuccess { .. },
                        ))
                    )
                })
            }
            P2pConnectionIncomingAction::FinalizePendingLibp2p { .. } => {
                cfg!(feature = "p2p-libp2p")
            }
            P2pConnectionIncomingAction::Libp2pReceived { peer_id, .. } => {
                cfg!(feature = "p2p-libp2p")
                    && state.peers.get(peer_id).is_some_and(|peer| {
                        matches!(
                            &peer.status,
                            P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                                P2pConnectionIncomingState::FinalizePendingLibp2p { .. },
                            ))
                        )
                    })
            }
        }
    }
}

impl From<P2pConnectionIncomingAction> for P2pAction {
    fn from(a: P2pConnectionIncomingAction) -> Self {
        Self::Connection(P2pConnectionAction::Incoming(a))
    }
}
