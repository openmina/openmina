use super::sync::TransitionFrontierSyncState;
use super::{
    TransitionFrontierAction, TransitionFrontierActionWithMetaRef, TransitionFrontierState,
};

impl TransitionFrontierState {
    pub fn reducer(&mut self, action: TransitionFrontierActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            TransitionFrontierAction::Genesis(a) => self.genesis.reducer(meta.with_action(a)),
            TransitionFrontierAction::GenesisInject => {
                let Some(genesis) = self.genesis.block_with_real_or_dummy_proof() else {
                    return;
                };
                self.best_chain = vec![genesis];
                if !self.sync.is_pending() {
                    self.sync = TransitionFrontierSyncState::Synced { time: meta.time() };
                }
            }
            TransitionFrontierAction::Sync(a) => {
                self.sync.reducer(meta.with_action(a), &self.best_chain);
            }
            TransitionFrontierAction::Synced {
                needed_protocol_states: needed_protocol_state_hashes,
            } => {
                let TransitionFrontierSyncState::CommitSuccess {
                    chain,
                    needed_protocol_states,
                    ..
                } = &mut self.sync
                else {
                    return;
                };
                let mut needed_protocol_state_hashes = needed_protocol_state_hashes.clone();
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
