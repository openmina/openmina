use crate::Store;

use super::sync::ledger::{
    TransitionFrontierSyncLedgerAction, TransitionFrontierSyncLedgerInitAction,
    TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQueryAction,
};
use super::{
    TransitionFrontierAction, TransitionFrontierActionWithMeta,
    TransitionFrontierRootLedgerSyncPendingAction,
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
            store.dispatch(TransitionFrontierSyncLedgerInitAction {});
            store.dispatch(TransitionFrontierSyncLedgerSnarkedLedgerSyncPeersQueryAction {});
        }
        TransitionFrontierAction::RootLedgerSyncPending(_) => {
            store.dispatch(TransitionFrontierSyncLedgerInitAction {});
        }
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
            _ => {
                todo!("sync done");
            }
        },
    }
}
