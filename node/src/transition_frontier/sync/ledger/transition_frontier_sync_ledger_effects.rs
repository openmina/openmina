use redux::ActionMeta;

use crate::Store;

use super::snarked::TransitionFrontierSyncLedgerSnarkedAction;
use super::staged::TransitionFrontierSyncLedgerStagedAction;
use super::TransitionFrontierSyncLedgerAction;

pub fn transition_frontier_sync_ledger_init_effects<S: redux::Service>(
    _: &ActionMeta,
    store: &mut Store<S>,
) {
    store.dispatch(TransitionFrontierSyncLedgerSnarkedAction::Pending);
}

pub fn transition_frontier_sync_ledger_snarked_success_effects<S: redux::Service>(
    _: &ActionMeta,
    store: &mut Store<S>,
) {
    if store.dispatch(TransitionFrontierSyncLedgerAction::Success) {
    } else if store.dispatch(TransitionFrontierSyncLedgerStagedAction::ReconstructEmpty) {
    } else if store.dispatch(TransitionFrontierSyncLedgerStagedAction::PartsFetchPending) {
    }
}

pub fn transition_frontier_sync_ledger_staged_success_effects<S: redux::Service>(
    _: &ActionMeta,
    store: &mut Store<S>,
) {
    store.dispatch(TransitionFrontierSyncLedgerAction::Success);
}
