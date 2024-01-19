use crate::{connection::P2pConnectionState, P2pLibP2pPeerState, P2pPeerStatus};

use super::{
    incoming::P2pConnectionLibP2pIncomingState, outgoing::P2pConnectionLibP2pOutgoingState,
    P2pConnectionLibP2pAction, P2pConnectionLibP2pActionWithMetaRef,
};

impl P2pLibP2pPeerState {
    pub fn reducer(&mut self, action: P2pConnectionLibP2pActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        let P2pLibP2pPeerState { dial_opts, status } = self;
        match action {
            P2pConnectionLibP2pAction::Outgoing(action) => {
                if let P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(state)) = status {
                    state.reducer(meta.with_action(action));
                } else {
                    let mut state = P2pConnectionLibP2pOutgoingState::default();
                    state.reducer(meta.with_action(action));
                    *status = P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(state));
                };
            }
            P2pConnectionLibP2pAction::Incoming(action) => {
                let incoming_state = if let P2pPeerStatus::Connecting(
                    P2pConnectionState::Incoming(state),
                ) = status
                {
                    state.reducer(meta.with_action(action));
                } else {
                    let mut state = P2pConnectionLibP2pIncomingState::default();
                    state.reducer(meta.with_action(action));
                    *status = P2pPeerStatus::Connecting(P2pConnectionState::Incoming(state));
                };
            }
        }
    }
}
