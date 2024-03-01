use std::time::Duration;

use serde::{Deserialize, Serialize};

use openmina_core::requests::RpcId;

use crate::connection::incoming::P2pConnectionIncomingState;
use crate::connection::P2pConnectionErrorResponse;
use crate::{webrtc, P2pState, PeerId};

use super::{P2pConnectionOutgoingError, P2pConnectionOutgoingInitOpts};

pub type P2pConnectionOutgoingActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a P2pConnectionOutgoingAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionOutgoingAction {
    RandomInit,
    Init {
        opts: P2pConnectionOutgoingInitOpts,
        rpc_id: Option<RpcId>,
    },
    Reconnect {
        opts: P2pConnectionOutgoingInitOpts,
        rpc_id: Option<RpcId>,
    },
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
        offer: webrtc::Offer,
    },
    OfferSendSuccess {
        peer_id: PeerId,
    },
    AnswerRecvPending {
        peer_id: PeerId,
    },
    AnswerRecvError {
        peer_id: PeerId,
        error: P2pConnectionErrorResponse,
    },
    AnswerRecvSuccess {
        peer_id: PeerId,
        answer: webrtc::Answer,
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
        error: P2pConnectionOutgoingError,
    },
    Success {
        peer_id: PeerId,
    },
}

impl P2pConnectionOutgoingAction {
    pub fn peer_id(&self) -> Option<&PeerId> {
        match self {
            Self::RandomInit => None,
            Self::Init { opts, .. } | Self::Reconnect { opts, .. } => Some(opts.peer_id()),
            Self::OfferSdpCreatePending { peer_id, .. }
            | Self::OfferSdpCreateError { peer_id, .. }
            | Self::OfferSdpCreateSuccess { peer_id, .. }
            | Self::OfferReady { peer_id, .. }
            | Self::OfferSendSuccess { peer_id }
            | Self::AnswerRecvPending { peer_id }
            | Self::AnswerRecvError { peer_id, .. }
            | Self::AnswerRecvSuccess { peer_id, .. }
            | Self::FinalizePending { peer_id }
            | Self::FinalizeError { peer_id, .. }
            | Self::FinalizeSuccess { peer_id }
            | Self::Timeout { peer_id }
            | Self::Error { peer_id, .. }
            | Self::Success { peer_id } => Some(peer_id),
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pConnectionOutgoingAction {
    fn is_enabled(&self, state: &P2pState, time: redux::Timestamp) -> bool {
        match self {
            P2pConnectionOutgoingAction::RandomInit => {
                !state.already_has_min_peers() && !state.initial_unused_peers().is_empty()
            }
            P2pConnectionOutgoingAction::Init { opts, .. } => {
                !state.already_has_min_peers() && !state.peers.contains_key(opts.peer_id())
            }
            P2pConnectionOutgoingAction::Reconnect { opts, .. } => {
                if state.already_has_min_peers() {
                    return false;
                }
                state
                    .peers
                    .iter()
                    .filter_map(|(id, p)| match &p.status {
                        P2pPeerStatus::Connecting(s) => {
                            match s {
                                P2pConnectionState::Outgoing(
                                    P2pConnectionOutgoingState::Error { time, .. },
                                )
                                | P2pConnectionState::Incoming(
                                    P2pConnectionIncomingState::Error { time, .. },
                                ) => Some((*time, id, &p.dial_opts)),
                                _ => None,
                            }
                        }
                        P2pPeerStatus::Disconnected { time } => Some((*time, id, &p.dial_opts)),
                        _ => None,
                    })
                    .min_by_key(|(time, ..)| *time)
                    .filter(|(_, id, _)| *id == opts.peer_id())
                    .filter(|(.., peer_opts)| peer_opts.as_ref().map_or(true, |o| o == opts))
                    .map_or(false, |(t, ..)| {
                        time.checked_sub(t) >= Some(Duration::from_secs(30))
                    })
            }
            P2pConnectionOutgoingAction::OfferSdpCreatePending { peer_id } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionOutgoingState::Init { .. },
                    )) => true,
                    _ => false,
                }),
            P2pConnectionOutgoingAction::OfferSdpCreateError { peer_id, .. } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionOutgoingState::OfferSdpCreatePending { .. },
                    )) => true,
                    _ => false,
                }),
            P2pConnectionOutgoingAction::OfferSdpCreateSuccess { peer_id, .. } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionOutgoingState::OfferSdpCreatePending { .. },
                    )) => true,
                    _ => false,
                }),
            P2pConnectionOutgoingAction::OfferReady { peer_id, .. } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionOutgoingState::OfferSdpCreateSuccess { .. },
                    )) => true,
                    _ => false,
                }),
            P2pConnectionOutgoingAction::OfferSendSuccess { peer_id } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionOutgoingState::OfferReady { .. },
                    )) => true,
                    _ => false,
                }),
            P2pConnectionOutgoingAction::AnswerRecvPending { peer_id } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionOutgoingState::OfferSendSuccess { .. },
                    )) => true,
                    _ => false,
                }),
            P2pConnectionOutgoingAction::AnswerRecvError { peer_id, .. } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionOutgoingState::AnswerRecvPending { .. },
                    )) => true,
                    _ => false,
                }),
            P2pConnectionOutgoingAction::AnswerRecvSuccess { peer_id, .. } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionOutgoingState::AnswerRecvPending { .. },
                    )) => true,
                    _ => false,
                }),
            P2pConnectionOutgoingAction::FinalizePending { peer_id } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(v)) => match v {
                        P2pConnectionOutgoingState::Init { opts, .. } => opts.is_libp2p(),
                        P2pConnectionOutgoingState::AnswerRecvSuccess { .. } => true,
                        _ => false,
                    },
                    _ => false,
                }),
            P2pConnectionOutgoingAction::FinalizeError { peer_id, .. } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionOutgoingState::FinalizePending { .. },
                    )) => true,
                    _ => false,
                }),
            P2pConnectionOutgoingAction::FinalizeSuccess { peer_id } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionOutgoingState::FinalizePending { .. },
                    )) => true,
                    _ => false,
                }),
            P2pConnectionOutgoingAction::Timeout { peer_id } => state
                .peers
                .get(peer_id)
                .and_then(|peer| peer.status.as_connecting()?.as_outgoing())
                .map_or(false, |s| s.is_timed_out(time)),
            P2pConnectionOutgoingAction::Error { peer_id, error } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(s)) => match error {
                        P2pConnectionOutgoingError::SdpCreateError(_) => {
                            matches!(s, P2pConnectionOutgoingState::OfferSdpCreatePending { .. })
                        }
                        P2pConnectionOutgoingError::Rejected(_)
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
                    .map_or(false, |peer| match &peer.status {
                        P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                            P2pConnectionOutgoingState::FinalizeSuccess { .. },
                        )) => true,
                        _ => false,
                    })
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
