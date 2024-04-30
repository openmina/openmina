use super::{LedgerAction, LedgerActionWithMetaRef, LedgerState};

impl LedgerState {
    pub fn reducer(&mut self, action: LedgerActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            LedgerAction::Write(a) => self.write.reducer(meta.with_action(a)),
            LedgerAction::Read(a) => self.read.reducer(meta.with_action(a)),
        }
    }
}
