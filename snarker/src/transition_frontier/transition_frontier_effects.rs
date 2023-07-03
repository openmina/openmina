use crate::Store;

use super::sync::ledger::{
    TransitionFrontierSyncLedgerAction, TransitionFrontierSyncLedgerInitAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQueryAction,
    TransitionFrontierSyncLedgerStagedLedgerReconstructPendingAction,
};
use super::{
    TransitionFrontierAction, TransitionFrontierActionWithMeta,
    TransitionFrontierRootLedgerSyncPendingAction, TransitionFrontierRootLedgerSyncSuccessAction,
};

pub fn transition_frontier_effects<S: crate::Service>(
    store: &mut Store<S>,
    action: TransitionFrontierActionWithMeta,
) {
    let (action, meta) = action.split();

    match action {
        TransitionFrontierAction::SyncInit(_) => {
            store.dispatch(TransitionFrontierRootLedgerSyncPendingAction {});
        }
        TransitionFrontierAction::SyncBestTipUpdate(_) => {
            // if root snarked ledger changed.
            store.dispatch(TransitionFrontierSyncLedgerInitAction {});
            // if root snarked ledger stayed same but root block changed
            // while reconstructing staged ledger.
            store.dispatch(TransitionFrontierSyncLedgerStagedLedgerReconstructPendingAction {});
            store.dispatch(TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQueryAction {});
        }
        TransitionFrontierAction::RootLedgerSyncPending(_) => {
            store.dispatch(TransitionFrontierSyncLedgerInitAction {});
        }
        TransitionFrontierAction::RootLedgerSyncSuccess(_) => {}
        TransitionFrontierAction::SyncLedger(action) => match action {
            TransitionFrontierSyncLedgerAction::Init(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPending(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeersQuery(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeerQueryInit(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeerQueryRetry(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeerQueryPending(_) => {}
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeerQueryError(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeerQuerySuccess(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncChildHashesReceived(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncChildAccountsReceived(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncSuccess(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerReconstructPending(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsFetchInit(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsFetchPending(_) => {}
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsFetchError(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsFetchSuccess(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsApplyInit(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerPartsApplySuccess(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::StagedLedgerReconstructSuccess(action) => {
                action.effects(&meta, store);
            }
            TransitionFrontierSyncLedgerAction::Success(_) => {
                store.dispatch(TransitionFrontierRootLedgerSyncSuccessAction {});
            }
        },
    }
}
