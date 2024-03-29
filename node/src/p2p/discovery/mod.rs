pub use ::p2p::discovery::*;

impl redux::EnablingCondition<crate::State> for P2pDiscoveryAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}
