use openmina_core::{error, Substate};
use p2p::{P2pAction, P2pInitializeAction};

use crate::{Action, ActionWithMeta, EventSourceAction, P2p, State};

pub fn reducer(
    state: &mut State,
    action: &ActionWithMeta,
    dispatcher: &mut redux::Dispatcher<Action, State>,
) {
    let meta = action.meta().clone();
    match action.action() {
        Action::CheckTimeouts(_) => {}
        Action::EventSource(EventSourceAction::NewEvent { .. }) => {}
        Action::EventSource(_) => {}

        Action::P2p(a) => match a {
            P2pAction::Initialization(P2pInitializeAction::Initialize { chain_id }) => {
                if let Err(err) = state.p2p.initialize(chain_id) {
                    error!(meta.time(); summary = "error initializing p2p", error = display(err));
                }
            }
            action => match &mut state.p2p {
                P2p::Pending(_) => {
                    error!(meta.time(); summary = "p2p is not initialized", action = debug(action))
                }
                P2p::Ready(_) => p2p::P2pState::reducer(
                    Substate::new(state, dispatcher),
                    meta.with_action(action),
                ),
            },
        },
        Action::Ledger(a) => {
            state.ledger.reducer(meta.with_action(a));
        }
        Action::Snark(a) => {
            snark::SnarkState::reducer(Substate::new(state, dispatcher), meta.with_action(a));
        }
        Action::Consensus(a) => {
            crate::consensus::ConsensusState::reducer(
                Substate::new(state, dispatcher),
                meta.with_action(a),
            );
        }
        Action::TransitionFrontier(a) => {
            crate::transition_frontier::TransitionFrontierState::reducer(
                Substate::new(state, dispatcher),
                meta.with_action(a),
            );
        }
        Action::SnarkPool(a) => {
            crate::snark_pool::SnarkPoolState::reducer(
                Substate::new(state, dispatcher),
                meta.with_action(a),
            );
        }
        Action::SnarkPoolEffect(_) => {}
        Action::TransactionPool(a) => {
            state.transaction_pool.reducer(meta.with_action(a));
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
            crate::watched_accounts::WatchedAccountsState::reducer(
                Substate::new(state, dispatcher),
                meta.with_action(a),
            );
        }
    }

    // must be the last.
    state.action_applied(action);
}
