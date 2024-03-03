use openmina_core::error;
use p2p::{P2pAction, P2pInitializeAction};

use crate::{Action, ActionWithMeta, EventSourceAction, P2p, State};

pub fn reducer(
    state: &mut State,
    action: &ActionWithMeta,
    global_state: &State,
    dispatcher: &mut redux::ActionQueue<Action, State>,
) {
    let meta = action.meta().clone();
    match action.action() {
        Action::CheckTimeouts(_) => {}
        Action::EventSource(EventSourceAction::NewEvent { .. }) => {}
        Action::EventSource(_) => {}

        Action::P2p(a) => state.p2p.reduce(meta.with_action(a)),
        Action::Ledger(a) => {
            state.ledger.reducer(meta.with_action(a));
        }
        Action::Snark(a) => {
            state.snark.reducer(meta.with_action(a));
        }
        Action::Consensus(a) => {
            state.consensus.reducer(meta.with_action(a));
        }
        Action::TransitionFrontier(a) => {
            state.transition_frontier.reducer(meta.with_action(a));
        }
        Action::SnarkPool(a) => {
            state.snark_pool.reducer(meta.with_action(a));
        }
        Action::BlockProducer(a) => {
            state
                .block_producer
                .reducer(meta.with_action(a), &state.transition_frontier.best_chain);
        }
        Action::ExternalSnarkWorker(a) => {
            state.external_snark_worker.reducer(meta.with_action(a));
        }
        Action::Rpc(a) => {
            state.rpc.reducer(meta.with_action(a));
        }
        Action::WatchedAccounts(a) => {
            state.watched_accounts.reducer(meta.with_action(a));
        }
    }

    // must be the last.
    state.action_applied(action);
}

impl P2p {
    fn reduce(&mut self, action: redux::ActionWithMeta<&P2pAction>) {
        let (action, meta) = action.split();
        match action {
            P2pAction::Initialization(P2pInitializeAction::Initialize { chain_id }) => {
                if let Err(err) = self.initialize(chain_id) {
                    error!(meta.time(); summary = "error initializing p2p", error = display(err));
                }
            }
            action => match self {
                P2p::Pending(_) => {
                    error!(meta.time(); summary = "p2p is not initialized", action = debug(action))
                }
                P2p::Ready(p2p_state) => p2p_state.reducer(meta.with_action(action)),
            },
        }
    }
}
