pub use ::p2p::discovery::*;

impl redux::EnablingCondition<crate::State> for P2pDiscoveryInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pDiscoverySuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pDiscoveryKademliaBootstrapAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pDiscoveryKademliaInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.p2p.enough_time_elapsed(state.time()) && self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pDiscoveryKademliaAddRouteAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pDiscoveryKademliaSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pDiscoveryKademliaFailureAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}
