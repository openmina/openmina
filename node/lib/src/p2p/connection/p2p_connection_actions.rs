use super::*;

impl redux::EnablingCondition<crate::State> for outgoing::P2pConnectionOutgoingInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
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
