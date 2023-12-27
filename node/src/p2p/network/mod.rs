pub use ::p2p::network::*;

impl redux::EnablingCondition<crate::State> for P2pNetworkSchedulerInterfaceDetectedAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSchedulerInterfaceExpiredAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSchedulerOutgoingDidConnectAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSchedulerIncomingDataIsReadyAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSchedulerIncomingDataDidReceiveAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSchedulerSelectDoneAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSchedulerSelectErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkPnetIncomingDataAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkPnetOutgoingDataAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkPnetSetupNonceAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSelectInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSelectIncomingDataAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSelectIncomingTokenAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSelectOutgoingTokensAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkNoiseInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkNoiseIncomingDataAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkNoiseIncomingChunkAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkNoiseOutgoingChunkAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkNoiseOutgoingDataAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkNoiseDecryptedDataAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkNoiseHandshakeDoneAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkYamuxIncomingDataAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkYamuxOutgoingDataAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkYamuxIncomingFrameAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}
