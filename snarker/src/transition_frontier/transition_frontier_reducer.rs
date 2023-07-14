use super::sync::TransitionFrontierSyncState;
use super::{
    TransitionFrontierAction, TransitionFrontierActionWithMetaRef, TransitionFrontierState,
};

impl TransitionFrontierState {
    pub fn reducer(&mut self, action: TransitionFrontierActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            TransitionFrontierAction::Sync(a) => {
                self.sync
                    .reducer(meta.with_action(a), &self.config, &self.best_chain);
            }
            TransitionFrontierAction::Synced(_) => {
                let TransitionFrontierSyncState::BlocksSuccess { chain, .. } = &mut self.sync else { return };
                self.best_chain = std::mem::take(chain);
                self.sync = TransitionFrontierSyncState::Synced { time: meta.time() };
            }
        }
    }
}
