//! Implements the reducer function for the transition frontier state,
//! handling state transitions in response to dispatched actions.

use super::sync::{SyncError, TransitionFrontierSyncState};
use super::{
    TransitionFrontierAction, TransitionFrontierActionWithMetaRef, TransitionFrontierState,
};
use openmina_core::block::AppliedBlock;

impl TransitionFrontierState {
    /// Main reducer function for the transition frontier state.
    /// 
    /// This function handles all actions that can be dispatched to the transition frontier,
    /// delegating to sub-reducers for specific components like genesis, candidates, and sync.
    pub fn reducer(
        mut state_context: crate::Substate<Self>,
        action: TransitionFrontierActionWithMetaRef<'_>,
    ) {
        let Ok(state) = state_context.get_substate_mut() else {
            // TODO: log or propagate
            return;
        };
        let (action, meta) = action.split();

        // Drop the diff, it's been processed in the effect
        state.chain_diff.take();

        match action {
            TransitionFrontierAction::Genesis(a) => {
                super::genesis::TransitionFrontierGenesisState::reducer(
                    openmina_core::Substate::from_compatible_substate(state_context),
                    meta.with_action(a),
                )
            }
            TransitionFrontierAction::GenesisEffect(_) => {}
            TransitionFrontierAction::GenesisInject => {
                let Some(genesis) = state.genesis.block_with_real_or_dummy_proof() else {
                    return;
                };
                let genesis = AppliedBlock {
                    block: genesis,
                    just_emitted_a_proof: true,
                };
                state.best_chain = vec![genesis];
                state.sync = TransitionFrontierSyncState::Synced { time: meta.time() };
            }
            TransitionFrontierAction::GenesisProvenInject => {
                let Some(genesis) = state.genesis.proven_block() else {
                    return;
                };
                if let Some(block) = state.best_chain.get_mut(0) {
                    block.block = genesis.clone();
                } else {
                    let genesis = AppliedBlock {
                        block: genesis.clone(),
                        just_emitted_a_proof: true,
                    };
                    state.best_chain = vec![genesis];
                }
                if !state.sync.is_pending() {
                    state.sync = TransitionFrontierSyncState::Synced { time: meta.time() };
                }
            }
            TransitionFrontierAction::Candidate(a) => {
                super::candidate::TransitionFrontierCandidatesState::reducer(
                    openmina_core::Substate::from_compatible_substate(state_context),
                    meta.with_action(a),
                );
            }
            TransitionFrontierAction::Sync(a) => {
                let best_chain = state.best_chain.clone();
                super::sync::TransitionFrontierSyncState::reducer(
                    openmina_core::Substate::from_compatible_substate(state_context),
                    meta.with_action(a),
                    &best_chain,
                );
            }
            TransitionFrontierAction::Synced {
                needed_protocol_states,
            } => {
                let Some(sync) = state.sync.as_commit_success() else {
                    return;
                };
                let new_chain = sync.new_chain.clone();

                // Add needed protocol states
                for hash in needed_protocol_states {
                    let Some(protocol_state) = sync.needed_protocol_states.get(&hash).cloned() else {
                        continue;
                    };
                    state.needed_protocol_states.insert(hash, protocol_state);
                }

                state.blacklist.retain(|_, height| {
                    // prune blocks from black list that can't end up
                    // into transition frontier anymore due to consensus
                    // reasons.
                    let tip = new_chain.last().unwrap();
                    height
                        .checked_add(tip.constants().k.as_u32())
                        .expect("overflow")
                        > tip.height()
                });
                state.chain_diff = state.maybe_make_chain_diff(&new_chain);
                state.best_chain = new_chain;
                state.sync = TransitionFrontierSyncState::Synced { time: meta.time() };
            }
            TransitionFrontierAction::SyncFailed { error, .. } => {
                match error {
                    SyncError::BlockApplyFailed(block, _) => {
                        state.blacklist.insert(block.hash().clone(), block.height());
                    }
                }
                state.sync = TransitionFrontierSyncState::Synced { time: meta.time() };
            }
        }
    }
}
