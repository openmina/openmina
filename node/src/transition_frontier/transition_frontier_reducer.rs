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
            TransitionFrontierAction::Synced(a) => {
                let TransitionFrontierSyncState::BlocksSuccess {
                    chain,
                    needed_protocol_states,
                    ..
                } = &mut self.sync
                else {
                    return;
                };
                let mut needed_protocol_state_hashes = a.needed_protocol_states.clone();
                let new_chain = std::mem::take(chain);

                self.needed_protocol_states
                    .extend(std::mem::take(needed_protocol_states));
                self.needed_protocol_states
                    .retain(|k, _| needed_protocol_state_hashes.remove(k));

                for hash in needed_protocol_state_hashes {
                    let block = self
                        .best_chain
                        .iter()
                        .find(|b| b.hash == hash)
                        .or_else(|| new_chain.iter().find(|b| b.hash == hash));
                    // TODO(binier): error log instead.
                    let block = block.expect("we lack needed block!");
                    let protocol_state = block.header().protocol_state.clone();
                    self.needed_protocol_states.insert(hash, protocol_state);
                }

                self.best_chain = new_chain;
                self.sync = TransitionFrontierSyncState::Synced { time: meta.time() };
            }
        }
    }
}
