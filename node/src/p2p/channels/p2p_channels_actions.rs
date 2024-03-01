use super::*;

impl redux::EnablingCondition<crate::State> for P2pChannelsMessageReceivedAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        self.is_enabled(&state.p2p, time)
    }
}
