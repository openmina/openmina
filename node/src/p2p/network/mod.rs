pub use ::p2p::network::*;

impl redux::EnablingCondition<crate::State> for P2pNetworkSchedulerAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkPnetAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSelectAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}
impl redux::EnablingCondition<crate::State> for P2pNetworkNoiseAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkYamuxIncomingDataAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkYamuxOutgoingDataAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkYamuxIncomingFrameAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkYamuxOutgoingFrameAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkYamuxPingStreamAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkYamuxOpenStreamAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkRpcInitAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkRpcIncomingDataAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkRpcIncomingMessageAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkRpcOutgoingQueryAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkRpcOutgoingResponseAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkRpcOutgoingDataAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkKademliaAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkKademliaStreamAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkKadRequestAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkKadBootstrapAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}
