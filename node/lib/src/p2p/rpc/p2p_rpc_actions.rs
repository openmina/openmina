use std::time::Duration;

use super::*;

impl redux::EnablingCondition<crate::State> for outgoing::P2pRpcOutgoingInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        let check_quotas = state
            .p2p
            .get_ready_peer(&self.peer_id)
            // TODO(binier): uncomment once menu is saved in state.
            // p.rpc.supports(self.request.kind()) && p.rpc.outgoing.next_req_id() == self.rpc_id
            .filter(|p| p.rpc.outgoing.next_req_id() == self.rpc_id)
            .map_or(false, |p| {
                let kind = self.request.kind();
                let Some(stats) = p.rpc.outgoing.stats.get(&kind) else { return true };
                match kind {
                    P2pRpcKind::BestTipGet => {
                        let time_passed = state.time().checked_sub(stats.last_requested);
                        time_passed >= Some(Duration::from_secs(60))
                    }
                    _ => true,
                }
            });
        check_quotas && self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for outgoing::P2pRpcOutgoingPendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for outgoing::P2pRpcOutgoingReceivedAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for outgoing::P2pRpcOutgoingErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for outgoing::P2pRpcOutgoingSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for outgoing::P2pRpcOutgoingFinishAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}
