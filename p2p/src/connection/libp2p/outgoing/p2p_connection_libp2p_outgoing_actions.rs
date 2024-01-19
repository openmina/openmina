use openmina_core::requests::RpcId;
use redux::EnablingCondition;
use serde::{Deserialize, Serialize};

use crate::{
    connection::{libp2p::P2pConnectionLibP2pAction, P2pConnectionAction, P2pConnectionState},
    P2pAction, P2pPeerStatus, P2pState, PeerId,
};

use super::{P2pConnectionLibP2pOutgoingError, P2pConnectionLibP2pOutgoingState};

#[derive(Serialize, Deserialize, Debug, Clone, derive_more::From)]
pub enum P2pConnectionLibP2pOutgoingAction {
    Init(P2pConnectionLibP2pOutgoingInitAction),
    FinalizePending(P2pConnectionLibP2pOutgoingFinalizePendingAction),
    FinalizeSuccess(P2pConnectionLibP2pOutgoingFinalizeSuccessAction),
    FinalizeError(P2pConnectionLibP2pOutgoingFinalizeErrorAction),
    FinalizeTimeout(P2pConnectionLibP2pOutgoingFinalizeTimeoutAction),
    Success(P2pConnectionLibP2pOutgoingSuccessAction),
    Error(P2pConnectionLibP2pOutgoingErrorAction),
}

impl P2pConnectionLibP2pOutgoingAction {
    pub fn peer_id(&self) -> Option<&PeerId> {
        Some(match self {
            P2pConnectionLibP2pOutgoingAction::Init(v) => &v.peer_id,
            P2pConnectionLibP2pOutgoingAction::FinalizePending(v) => &v.peer_id,
            P2pConnectionLibP2pOutgoingAction::FinalizeSuccess(v) => &v.peer_id,
            P2pConnectionLibP2pOutgoingAction::FinalizeError(v) => &v.peer_id,
            P2pConnectionLibP2pOutgoingAction::FinalizeTimeout(v) => &v.peer_id,
            P2pConnectionLibP2pOutgoingAction::Success(v) => &v.peer_id,
            P2pConnectionLibP2pOutgoingAction::Error(v) => &v.peer_id,
        })
    }
}

/// Initializes outgoing libp2p connection to the specified peer.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionLibP2pOutgoingInitAction {
    pub peer_id: PeerId,
    pub rpc_id: Option<RpcId>,
}

impl EnablingCondition<P2pState> for P2pConnectionLibP2pOutgoingInitAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_libp2p_peer(&self.peer_id).map_or(false, |peer| {
            matches!(
                &peer.status,
                P2pPeerStatus::Default
                    | P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionLibP2pOutgoingState::Default
                            | P2pConnectionLibP2pOutgoingState::Error(_)
                    ))
            )
        })
    }
}

/// Signals that the connection pending finalization.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionLibP2pOutgoingFinalizePendingAction {
    pub peer_id: PeerId,
}

impl EnablingCondition<P2pState> for P2pConnectionLibP2pOutgoingFinalizePendingAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_libp2p_peer(&self.peer_id).map_or(false, |peer| {
            matches!(
                &peer.status,
                P2pPeerStatus::Default
                    | P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionLibP2pOutgoingState::Init(_)
                    ))
            )
        })
    }
}

/// Signals that the connection is finalized succesfully.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionLibP2pOutgoingFinalizeSuccessAction {
    pub peer_id: PeerId,
}

impl EnablingCondition<P2pState> for P2pConnectionLibP2pOutgoingFinalizeSuccessAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_libp2p_peer(&self.peer_id).map_or(false, |peer| {
            matches!(
                &peer.status,
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionLibP2pOutgoingState::FinalizePending(_)
                ))
            )
        })
    }
}

/// Signals that the connection is finalized succesfully.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionLibP2pOutgoingFinalizeErrorAction {
    pub peer_id: PeerId,
    pub error: String,
}

impl EnablingCondition<P2pState> for P2pConnectionLibP2pOutgoingFinalizeErrorAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_libp2p_peer(&self.peer_id).map_or(false, |peer| {
            matches!(
                &peer.status,
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionLibP2pOutgoingState::FinalizePending(_)
                ))
            )
        })
    }
}

/// Signals timeout while waiting for connection finalization.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionLibP2pOutgoingFinalizeTimeoutAction {
    pub peer_id: PeerId,
}

impl EnablingCondition<P2pState> for P2pConnectionLibP2pOutgoingFinalizeTimeoutAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_libp2p_peer(&self.peer_id).map_or(false, |peer| {
            matches!(
                &peer.status,
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionLibP2pOutgoingState::FinalizePending(_)
                ))
            )
        })
    }
}

/// Signals that the connection is not established because of error.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionLibP2pOutgoingSuccessAction {
    pub peer_id: PeerId,
    pub rpc_id: Option<RpcId>,
}

impl EnablingCondition<P2pState> for P2pConnectionLibP2pOutgoingSuccessAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_libp2p_peer(&self.peer_id).map_or(false, |peer| {
            matches!(
                &peer.status,
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionLibP2pOutgoingState::Success(_)
                ))
            )
        })
    }
}

/// Signals that the connection is not established because of error.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionLibP2pOutgoingErrorAction {
    pub peer_id: PeerId,
    pub error: P2pConnectionLibP2pOutgoingError,
    pub rpc_id: Option<RpcId>,
}

impl EnablingCondition<P2pState> for P2pConnectionLibP2pOutgoingErrorAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_libp2p_peer(&self.peer_id).map_or(false, |peer| {
            matches!(
                &peer.status,
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionLibP2pOutgoingState::Error(_)
                ))
            )
        })
    }
}

macro_rules! into_p2p_action {
    ($($action:ident),* $(,)?) => {
        $(
            impl From<$action> for P2pAction {
                fn from(value: $action) -> Self {
                    P2pAction::Connection(P2pConnectionAction::LibP2p(P2pConnectionLibP2pAction::Outgoing(value.into())))
                }
            }
        )*
    };
}

into_p2p_action!(
    P2pConnectionLibP2pOutgoingInitAction,
    P2pConnectionLibP2pOutgoingFinalizePendingAction,
    P2pConnectionLibP2pOutgoingFinalizeSuccessAction,
    P2pConnectionLibP2pOutgoingFinalizeErrorAction,
    P2pConnectionLibP2pOutgoingFinalizeTimeoutAction,
    P2pConnectionLibP2pOutgoingSuccessAction,
    P2pConnectionLibP2pOutgoingErrorAction,
);
