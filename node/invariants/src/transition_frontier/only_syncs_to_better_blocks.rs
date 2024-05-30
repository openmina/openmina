use node::core::block::ArcBlockWithHash;
use node::core::consensus::consensus_take;
use node::{ActionKind, ActionWithMeta, Store};

use crate::{Invariant, InvariantResult};

/// Makes sure transition frontier always only starts syncing and syncs
/// to a better block than the latest best tip.
#[derive(documented::Documented, Default, Clone, Copy)]
pub struct TransitionFrontierOnlySyncsToBetterBlocks;

impl Invariant for TransitionFrontierOnlySyncsToBetterBlocks {
    type InternalState = (Option<ArcBlockWithHash>, Option<ArcBlockWithHash>);
    fn triggers(&self) -> &[ActionKind] {
        &[
            ActionKind::TransitionFrontierSyncInit,
            ActionKind::TransitionFrontierSyncBestTipUpdate,
            ActionKind::TransitionFrontierSynced,
        ]
    }

    fn check<S: redux::Service>(
        self,
        (prev_best_tip, prev_target_best_tip): &mut Self::InternalState,
        store: &Store<S>,
        _action: &ActionWithMeta,
    ) -> InvariantResult {
        let transition_frontier = &store.state().transition_frontier;
        let best_tip = transition_frontier.best_tip();
        let target_best_tip = transition_frontier.sync.best_tip();
        let mut checked = false;

        // make sure new best tip is better than the prev one.
        match (best_tip, prev_best_tip.as_ref()) {
            (Some(best_tip), None) => {
                *prev_best_tip = Some(best_tip.clone());
            }
            (Some(best_tip), Some(prev_tip)) if best_tip.hash() != prev_tip.hash() => {
                checked = true;
                if !consensus_take(
                    prev_tip.consensus_state(),
                    best_tip.consensus_state(),
                    prev_tip.hash(),
                    best_tip.hash(),
                ) {
                    return InvariantResult::Violation(format!(
                        "best tip got downgraded!\nprev({}): {}\nnew({}): {}",
                        prev_tip.hash(),
                        serde_json::to_string(prev_tip.consensus_state()).unwrap(),
                        best_tip.hash(),
                        serde_json::to_string(best_tip.consensus_state()).unwrap(),
                    ));
                }
                *prev_best_tip = Some(best_tip.clone());
            }
            _ => {}
        }

        // make sure new best tip target is better than current best tip.
        if let (Some(target_best_tip), Some(best_tip)) = (target_best_tip, best_tip) {
            checked = true;
            if !best_tip.is_genesis()
                && !consensus_take(
                    best_tip.consensus_state(),
                    target_best_tip.consensus_state(),
                    best_tip.hash(),
                    target_best_tip.hash(),
                )
            {
                return InvariantResult::Violation(format!(
                    "best tip target not better than current best tip!\nprev({}): {}\nnew({}): {}",
                    best_tip.hash(),
                    serde_json::to_string(best_tip.consensus_state()).unwrap(),
                    target_best_tip.hash(),
                    serde_json::to_string(target_best_tip.consensus_state()).unwrap(),
                ));
            }
        }

        // make sure new best tip target is better than the prev one.
        match (target_best_tip, prev_target_best_tip.as_ref()) {
            (Some(new_target), None) => {
                *prev_target_best_tip = Some(new_target.clone());
            }
            (Some(new_target), Some(prev_target)) if new_target.hash() != prev_target.hash() => {
                checked = true;
                if !consensus_take(
                    prev_target.consensus_state(),
                    new_target.consensus_state(),
                    prev_target.hash(),
                    new_target.hash(),
                ) {
                    return InvariantResult::Violation(format!(
                        "best tip target got downgraded!\nprev({}): {}\nnew({}): {}",
                        prev_target.hash(),
                        serde_json::to_string(prev_target.consensus_state()).unwrap(),
                        new_target.hash(),
                        serde_json::to_string(new_target.consensus_state()).unwrap(),
                    ));
                }
                *prev_target_best_tip = Some(new_target.clone());
            }
            _ => {}
        }

        if checked {
            InvariantResult::Ok
        } else {
            InvariantResult::Updated
        }
    }
}
