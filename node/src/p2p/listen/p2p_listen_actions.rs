use p2p::listen::P2pListenAction;
use redux::EnablingCondition;

use crate::State;

impl EnablingCondition<State> for P2pListenAction {
    fn is_enabled(&self, state: &State) -> bool {
        self.is_enabled(&state.p2p)
    }
}
