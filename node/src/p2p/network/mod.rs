pub use ::p2p::network::*;

impl redux::EnablingCondition<crate::State> for P2pNetworkSchedulerInterfaceDetectedAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSchedulerInterfaceExpiredAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSchedulerIncomingConnectionIsReadyAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSchedulerIncomingDidAcceptAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSchedulerOutgoingDidConnectAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSchedulerIncomingDataIsReadyAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSchedulerIncomingDataDidReceiveAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSchedulerSelectDoneAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSchedulerSelectErrorAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSchedulerYamuxDidInitAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkPnetIncomingDataAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkPnetOutgoingDataAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkPnetSetupNonceAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSelectInitAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSelectIncomingDataAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSelectIncomingTokenAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSelectOutgoingTokensAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkNoiseInitAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkNoiseIncomingDataAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkNoiseIncomingChunkAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkNoiseOutgoingChunkAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkNoiseOutgoingDataAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkNoiseDecryptedDataAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkNoiseHandshakeDoneAction {
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
