use super::sync::TransitionFrontierSyncState;
use super::{
    TransitionFrontierAction, TransitionFrontierActionWithMetaRef, TransitionFrontierState,
};

impl TransitionFrontierState {
    pub fn reducer(
        mut state: crate::Substate<Self>,
        action: TransitionFrontierActionWithMetaRef<'_>,
    ) {
        let (action, meta) = action.split();
        match action {
            TransitionFrontierAction::Genesis(a) => {
                super::genesis::TransitionFrontierGenesisState::reducer(
                    openmina_core::Substate::from_compatible_substate(state),
                    meta.with_action(a),
                )
            }
            TransitionFrontierAction::GenesisEffect(_) => {}
            TransitionFrontierAction::GenesisInject => {
                let Some(genesis) = state.genesis.block_with_real_or_dummy_proof() else {
                    return;
                };
                state.best_chain = vec![genesis];
                if !state.sync.is_pending() {
                    state.sync = TransitionFrontierSyncState::Synced { time: meta.time() };
                }
            }
            TransitionFrontierAction::Sync(a) => {
                let best_chain = state.best_chain.clone();
                super::sync::TransitionFrontierSyncState::reducer(
                    openmina_core::Substate::from_compatible_substate(state),
                    meta.with_action(a),
                    &best_chain,
                );
            }
            TransitionFrontierAction::Synced {
                needed_protocol_states: needed_protocol_state_hashes,
            } => {
                let TransitionFrontierSyncState::CommitSuccess {
                    chain,
                    needed_protocol_states,
                    ..
                } = &mut state.sync
                else {
                    return;
                };
                let mut needed_protocol_state_hashes = needed_protocol_state_hashes.clone();
                let new_chain = std::mem::take(chain);
                let needed_protocol_states = std::mem::take(needed_protocol_states);

                state.needed_protocol_states.extend(needed_protocol_states);
                state
                    .needed_protocol_states
                    .retain(|k, _| needed_protocol_state_hashes.remove(k));

                for hash in needed_protocol_state_hashes {
                    let block = state
                        .best_chain
                        .iter()
                        .find(|b| b.hash == hash)
                        .or_else(|| new_chain.iter().find(|b| b.hash == hash));
                    // TODO(binier): error log instead.
                    let block = block.expect("we lack needed block!");
                    let protocol_state = block.header().protocol_state.clone();
                    state.needed_protocol_states.insert(hash, protocol_state);
                }

                state.best_chain = new_chain;
                state.sync = TransitionFrontierSyncState::Synced { time: meta.time() };
            }
        }
    }
}
