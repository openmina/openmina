use redux::ActionMeta;

use crate::Store;

use super::snarked::{
    TransitionFrontierSyncLedgerSnarkedPendingAction,
    TransitionFrontierSyncLedgerSnarkedSuccessAction,
};
use super::staged::{
    TransitionFrontierSyncLedgerStagedPartsFetchPendingAction,
    TransitionFrontierSyncLedgerStagedReconstructEmptyAction,
    TransitionFrontierSyncLedgerStagedSuccessAction,
};
use super::{TransitionFrontierSyncLedgerInitAction, TransitionFrontierSyncLedgerSuccessAction};

impl TransitionFrontierSyncLedgerInitAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerSnarkedPendingAction {});
    }
}

impl TransitionFrontierSyncLedgerSnarkedSuccessAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        if store.dispatch(TransitionFrontierSyncLedgerSuccessAction {}) {
        } else if store.dispatch(TransitionFrontierSyncLedgerStagedReconstructEmptyAction {}) {
        } else if store.dispatch(TransitionFrontierSyncLedgerStagedPartsFetchPendingAction {}) {
        }
    }
}

impl TransitionFrontierSyncLedgerStagedSuccessAction {
    pub fn effects<S: redux::Service>(self, _: &ActionMeta, store: &mut Store<S>) {
        store.dispatch(TransitionFrontierSyncLedgerSuccessAction {});
    }
}
