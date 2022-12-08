use crate::{Action, ActionWithMeta, State};

pub fn reducer(state: &mut State, action: &ActionWithMeta) {
    let meta = action.meta().clone();
    match action.action() {
        Action::CheckTimeouts(_) => {}
        Action::EventSource(_) => {}

        Action::P2p(a) => {
            state.p2p.reducer(meta.with_action(a));
        }
        Action::Snark(a) => {
            state.snark.reducer(meta.with_action(a));
        }
        Action::Consensus(a) => {
            state.consensus.reducer(meta.with_action(a));
        }
        Action::Rpc(a) => {
            state.rpc.reducer(meta.with_action(a));
        }
    }

    // must be the last.
    state.action_applied(action);
}
