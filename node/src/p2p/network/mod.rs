pub use ::p2p::network::*;
use p2p::network::identify::{P2pNetworkIdentifyAction, P2pNetworkIdentifyStreamAction};

impl redux::EnablingCondition<crate::State> for P2pNetworkSchedulerAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        state.p2p.is_enabled(self, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkPnetAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        state.p2p.is_enabled(self, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSelectAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        state.p2p.is_enabled(self, time)
    }
}
impl redux::EnablingCondition<crate::State> for P2pNetworkNoiseAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        state.p2p.is_enabled(self, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkYamuxAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        state.p2p.is_enabled(self, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkRpcAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        state.p2p.is_enabled(self, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkIdentifyStreamAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        state.p2p.is_enabled(self, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkIdentifyAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        state.p2p.is_enabled(self, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkKademliaAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        state.p2p.is_enabled(self, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkKademliaStreamAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        state.p2p.is_enabled(self, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkKadRequestAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        state.p2p.is_enabled(self, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkKadBootstrapAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        state.p2p.is_enabled(self, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkPubsubAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        state.p2p.is_enabled(self, time)
    }
}

impl redux::EnablingCondition<crate::State> for P2pNetworkSchedulerEffectfulAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        state.p2p.is_enabled(self, time)
    }
}
