use super::*;

impl redux::EnablingCondition<crate::State> for P2pConnectionOutgoingAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        state.p2p.is_enabled(self, time)
    }
}
