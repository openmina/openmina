use serde::{Deserialize, Serialize};

use crate::{
    connection::{
        libp2p::{outgoing::P2pConnectionLibP2pOutgoingState, P2pConnectionLibP2pAction},
        P2pConnectionAction, P2pConnectionState,
    },
    P2pAction, P2pConnectionId, P2pPeerStatus, PeerId,
};

use super::P2pConnectionLibP2pIncomingState;

#[derive(Clone, Debug, Serialize, Deserialize, derive_more::From)]
pub enum P2pConnectionLibP2pIncomingAction {
    Success(P2pConnectionLibP2pIncomingSuccessAction),
}

impl P2pConnectionLibP2pIncomingAction {
    pub fn peer_id(&self) -> Option<&PeerId> {
        match self {
            P2pConnectionLibP2pIncomingAction::Success(action) => Some(&action.peer_id),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct P2pConnectionLibP2pIncomingSuccessAction {
    pub connection_id: P2pConnectionId,
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<crate::P2pState> for P2pConnectionLibP2pIncomingSuccessAction {
    fn is_enabled(&self, state: &crate::P2pState) -> bool {
        state
            .get_libp2p_peer(&self.peer_id)
            .map_or(true, |peer| match &peer.status {
                P2pPeerStatus::Default => true,
                P2pPeerStatus::Connecting(
                    P2pConnectionState::Incoming(P2pConnectionLibP2pIncomingState::Default)
                    | P2pConnectionState::Outgoing(P2pConnectionLibP2pOutgoingState::Error(_)),
                ) => true,
                P2pPeerStatus::Disconnected { time } => true,
                _ => false,
            })
    }
}

impl From<P2pConnectionLibP2pIncomingSuccessAction> for P2pAction {
    fn from(value: P2pConnectionLibP2pIncomingSuccessAction) -> Self {
        P2pAction::Connection(P2pConnectionAction::LibP2p(
            P2pConnectionLibP2pAction::Incoming(value.into()),
        ))
    }
}
