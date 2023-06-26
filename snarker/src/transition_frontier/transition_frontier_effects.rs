use crate::Store;

use super::sync::ledger::{
    TransitionFrontierSyncLedgerAction, TransitionFrontierSyncLedgerInitAction,
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
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeerQueryPending(_) => {}
            TransitionFrontierSyncLedgerAction::SnarkedLedgerSyncPeerQuerySuccess(action) => {
                action.effects(&meta, store);
            }
            _ => {
                todo!("sync done");
            }
        },
    }
}
