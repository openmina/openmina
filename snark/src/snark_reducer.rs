use super::{SnarkAction, SnarkActionWithMetaRef, SnarkState};

impl SnarkState {
    pub fn reducer(&mut self, action: SnarkActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            SnarkAction::BlockVerify(a) => self.block_verify.reducer(meta.with_action(a)),
            SnarkAction::WorkVerify(a) => self.work_verify.reducer(meta.with_action(a)),
        }
    }
}
