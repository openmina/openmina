use p2p::listen::{P2pListenNewAction, P2pListenExpiredAction, P2pListenErrorAction, P2pListenClosedAction};
use redux::EnablingCondition;

use crate::State;

impl EnablingCondition<State> for P2pListenNewAction {
    fn is_enabled(&self, state: &State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl EnablingCondition<State> for P2pListenExpiredAction {
    fn is_enabled(&self, state: &State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl EnablingCondition<State> for P2pListenErrorAction {
    fn is_enabled(&self, state: &State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl EnablingCondition<State> for P2pListenClosedAction {
    fn is_enabled(&self, state: &State) -> bool {
        self.is_enabled(&state.p2p)
    }
}
