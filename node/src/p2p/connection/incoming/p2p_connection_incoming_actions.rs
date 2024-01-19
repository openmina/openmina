use super::*;

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCIncomingInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCIncomingAnswerSdpCreatePendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCIncomingAnswerSdpCreateErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCIncomingAnswerSdpCreateSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCIncomingAnswerReadyAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCIncomingAnswerSendSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCIncomingFinalizePendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCIncomingFinalizeErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCIncomingFinalizeSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCIncomingTimeoutAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        let peer = state.p2p.get_webrtc_peer(&self.peer_id);
        let timed_out = peer
            .and_then(|peer| peer.status.as_connecting()?.as_incoming())
            .map_or(false, |s| s.is_timed_out(state.time()));
        timed_out && self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCIncomingErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCIncomingSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCIncomingLibp2pReceivedAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionLibP2pIncomingSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}
