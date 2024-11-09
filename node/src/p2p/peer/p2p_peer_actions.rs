use super::*;

impl redux::EnablingCondition<crate::State> for P2pPeerAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        if let P2pPeerAction::BestTipUpdate { best_tip, .. } = self {
            if !state.prevalidate_block(best_tip) {
                return false;
            }
        }
        state.p2p.is_enabled(self, time)
    }
}
