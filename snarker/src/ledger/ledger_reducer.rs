use super::{LedgerActionWithMetaRef, LedgerState};

impl LedgerState {
    pub fn reducer(&mut self, action: LedgerActionWithMetaRef<'_>) {
        let (action, _) = action.split();
        match action {
            _ => {}
        }
    }
}
