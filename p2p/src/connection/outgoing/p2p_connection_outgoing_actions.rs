use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use openmina_core::requests::RpcId;

use crate::connection::P2pConnectionErrorResponse;
use crate::{webrtc, P2pState, PeerId};

use super::{P2pConnectionOutgoingError, P2pConnectionOutgoingInitOpts};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(opts), display(peer_id), display(error)))]
pub enum P2pConnectionOutgoingAction {
    /// Initialize connection to a random peer.
    #[action_event(level = trace)]
    RandomInit,
    /// Initialize connection to a new peer.
    Init {
        opts: P2pConnectionOutgoingInitOpts,
        rpc_id: Option<RpcId>,
    },
    /// Reconnect to an existing peer.
    // TODO: rename `Init` and `Reconnect` to `New` and `Connect` or something
    Reconnect {
        opts: P2pConnectionOutgoingInitOpts,
        rpc_id: Option<RpcId>,
    },
    #[action_event(level = trace)]
    OfferSdpCreatePending {
        peer_id: PeerId,
    },
    OfferSdpCreateError {
        peer_id: PeerId,
        error: String,
    },
    OfferSdpCreateSuccess {
        peer_id: PeerId,
        sdp: String,
    },
    OfferReady {
        peer_id: PeerId,
        offer: Box<webrtc::Offer>,
    },
    OfferSendSuccess {
        peer_id: PeerId,
    },
    #[action_event(level = trace)]
    AnswerRecvPending {
        peer_id: PeerId,
    },
    AnswerRecvError {
        peer_id: PeerId,
        error: P2pConnectionErrorResponse,
    },
    AnswerRecvSuccess {
        peer_id: PeerId,
        answer: Box<webrtc::Answer>,
    },
    #[action_event(level = trace)]
    FinalizePending {
        peer_id: PeerId,
    },
    /// Error finalizing outgoing connection.
    #[action_event(level = debug)]
    FinalizeError {
        peer_id: PeerId,
        error: String,
    },
    /// Outgoing connection successfully finalized.
    FinalizeSuccess {
        peer_id: PeerId,
    },
    /// Timeout establishing connection to a peer.
    Timeout {
        peer_id: PeerId,
    },
    /// Error connecting to a peer.
    #[action_event(level = debug)]
    Error {
        peer_id: PeerId,
        error: P2pConnectionOutgoingError,
    },
    /// Outgoing connection is successful.
    Success {
        peer_id: PeerId,
    },
}

impl redux::EnablingCondition<P2pState> for P2pConnectionOutgoingAction {
    fn is_enabled(&self, state: &P2pState, time: redux::Timestamp) -> bool {
        match self {
            P2pConnectionOutgoingAction::RandomInit =>  !state.already_has_min_peers() && state.disconnected_peers().next().is_some(),
            P2pConnectionOutgoingAction::Init { opts, .. } => {
                !state.already_has_min_peers() &&
                &state.my_id() != opts.peer_id() &&
                state
                    .peers
                    .get(opts.peer_id())
                    .map_or(true, |peer| !peer.status.is_connected_or_connecting())
            }
            P2pConnectionOutgoingAction::Reconnect { opts, .. } => {
                !state.already_has_min_peers()
                    && state.peers.get(opts.peer_id()).map_or(false, |peer| {
                        peer.can_reconnect(time, &state.config.timeouts)
                    })
            }
            P2pConnectionOutgoingAction::OfferSdpCreatePending { peer_id } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| matches!(&peer.status, P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionOutgoingState::Init { .. },
                    )))),
            P2pConnectionOutgoingAction::OfferSdpCreateError { peer_id, .. } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| matches!(&peer.status, P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionOutgoingState::OfferSdpCreatePending { .. },
                    )))),
            P2pConnectionOutgoingAction::OfferSdpCreateSuccess { peer_id, .. } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| matches!(&peer.status, P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionOutgoingState::OfferSdpCreatePending { .. },
                    )))),
            P2pConnectionOutgoingAction::OfferReady { peer_id, .. } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| matches!(&peer.status, P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionOutgoingState::OfferSdpCreateSuccess { .. },
                    )))),
            P2pConnectionOutgoingAction::OfferSendSuccess { peer_id } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| matches!(&peer.status, P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionOutgoingState::OfferReady { .. },
                    )))),
            P2pConnectionOutgoingAction::AnswerRecvPending { peer_id } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| matches!(&peer.status, P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionOutgoingState::OfferSendSuccess { .. },
                    )))),
            P2pConnectionOutgoingAction::AnswerRecvError { peer_id, .. } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| matches!(&peer.status, P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionOutgoingState::AnswerRecvPending { .. },
                    )))),
            P2pConnectionOutgoingAction::AnswerRecvSuccess { peer_id, .. } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| matches!(&peer.status, P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionOutgoingState::AnswerRecvPending { .. },
                    )))),
            P2pConnectionOutgoingAction::FinalizePending { peer_id } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| matches!(&peer.status, P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(v)) if match v {
                        P2pConnectionOutgoingState::Init { opts, .. } => opts.is_libp2p(),
                        P2pConnectionOutgoingState::AnswerRecvSuccess { .. } => true,
                        _ => false,
                    })),
            P2pConnectionOutgoingAction::FinalizeError { peer_id, .. } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| matches!(&peer.status, P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionOutgoingState::FinalizePending { .. },
                    )))),
            P2pConnectionOutgoingAction::FinalizeSuccess { peer_id } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| matches!(&peer.status, P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionOutgoingState::FinalizePending { .. },
                    )))),
            P2pConnectionOutgoingAction::Timeout { peer_id } => state
                .peers
                .get(peer_id)
                .and_then(|peer| peer.status.as_connecting()?.as_outgoing())
                .map_or(false, |s| s.is_timed_out(time, &state.config.timeouts)),
            P2pConnectionOutgoingAction::Error { peer_id, error } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(s)) => match error {
                        P2pConnectionOutgoingError::SdpCreateError(_) => {
                            matches!(s, P2pConnectionOutgoingState::OfferSdpCreatePending { .. })
                        }
                        P2pConnectionOutgoingError::Rejected(_)
                        | P2pConnectionOutgoingError::RemoteSignalDecryptionFailed
                        | P2pConnectionOutgoingError::RemoteInternalError => {
                            matches!(s, P2pConnectionOutgoingState::AnswerRecvPending { .. })
                        }
                        P2pConnectionOutgoingError::FinalizeError(_) => {
                            matches!(s, P2pConnectionOutgoingState::FinalizePending { .. })
                        }
                        P2pConnectionOutgoingError::Timeout => true,
                    },
                    _ => false,
                }),
            P2pConnectionOutgoingAction::Success { peer_id } => {
                state
                    .peers
                    .get(peer_id)
                    .map_or(false, |peer| matches!(&peer.status, P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                            P2pConnectionOutgoingState::FinalizeSuccess { .. },
                        ))))
            }
        }
    }
}

// --- From<LeafAction> for Action impls.
use crate::{
    connection::{P2pConnectionAction, P2pConnectionState},
    P2pPeerStatus,
};

use super::P2pConnectionOutgoingState;

impl From<P2pConnectionOutgoingAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a))
    }
}
