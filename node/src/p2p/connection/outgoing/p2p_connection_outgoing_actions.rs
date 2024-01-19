use std::time::Duration;

use p2p::peer::P2pPeerReconnectAction;

use super::*;

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCOutgoingInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pPeerReconnectAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        if !self.is_enabled(&state.p2p) {
            return false;
        }

        let Some(peer) = state.p2p.peers.get(&self.peer_id) else {
            return false;
        };
        let Some(time) = peer.not_connected_since() else {
            return false;
        };
        state.time().checked_sub(time) >= Some(Duration::from_secs(30))
    }
}

impl redux::EnablingCondition<crate::State>
    for P2pConnectionWebRTCOutgoingOfferSdpCreatePendingAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State>
    for P2pConnectionWebRTCOutgoingOfferSdpCreateErrorAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State>
    for P2pConnectionWebRTCOutgoingOfferSdpCreateSuccessAction
{
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCOutgoingOfferReadyAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCOutgoingOfferSendSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCOutgoingAnswerRecvPendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCOutgoingAnswerRecvErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCOutgoingAnswerRecvSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCOutgoingFinalizePendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCOutgoingFinalizeErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCOutgoingFinalizeSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCOutgoingTimeoutAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        let peer = state.p2p.get_webrtc_peer(&self.peer_id);
        let timed_out = peer
            .and_then(|peer| peer.status.as_connecting()?.as_outgoing())
            .map_or(false, |s| s.is_timed_out(state.time()));
        timed_out && self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCOutgoingErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionWebRTCOutgoingSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionLibP2pOutgoingInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionLibP2pOutgoingFinalizePendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionLibP2pOutgoingFinalizeSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionLibP2pOutgoingFinalizeErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionLibP2pOutgoingFinalizeTimeoutAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p) && {
            let peer = state.p2p.get_libp2p_peer(&self.peer_id);
            peer.and_then(|peer| peer.status.as_connecting()?.as_outgoing())
                .map_or(false, |s| s.is_timed_out(state.time(), Duration::from_secs(30)))
        }
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionLibP2pOutgoingSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionLibP2pOutgoingErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}
