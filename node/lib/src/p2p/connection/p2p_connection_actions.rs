use std::time::Duration;

use p2p::{connection::outgoing::P2pConnectionOutgoingState, P2pPeerStatus};

use super::*;

impl redux::EnablingCondition<crate::State> for outgoing::P2pConnectionOutgoingRandomInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for outgoing::P2pConnectionOutgoingInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for outgoing::P2pConnectionOutgoingReconnectAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        if !self.is_enabled(&state.p2p) {
            return false;
        }

        let Some(peer) = state.p2p.peers.get(&self.opts.peer_id) else {
            return false;
        };
        let delay_passed = match &peer.status {
            P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                P2pConnectionOutgoingState::Error { time, .. },
            ))
            | P2pPeerStatus::Disconnected { time, .. } => {
                state.time().checked_sub(*time) >= Some(Duration::from_secs(3))
            }
            _ => true,
        };
        delay_passed
    }
}

impl redux::EnablingCondition<crate::State> for outgoing::P2pConnectionOutgoingPendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for outgoing::P2pConnectionOutgoingErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for outgoing::P2pConnectionOutgoingSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}
