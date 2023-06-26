use crate::Store;

use super::{LedgerAction, LedgerActionWithMeta, LedgerService};

pub fn ledger_effects<S: LedgerService>(store: &mut Store<S>, action: LedgerActionWithMeta) {
    let (action, _) = action.split();

    match action {
        LedgerAction::ChildHashesAdd(action) => {
            store
                .service()
                .hashes_matrix_set(&action.ledger_id, &action.parent, action.hashes)
                .unwrap();
        }
        LedgerAction::ChildAccountsAdd(action) => {
            store
                .service()
                .accounts_set(&action.ledger_id, &action.parent, action.accounts)
                .unwrap();
        }
    }
}
