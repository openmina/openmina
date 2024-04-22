use crate::transition_frontier::sync::TransitionFrontierSyncAction;
use crate::watched_accounts::WatchedAccountsAction;
use crate::Store;
use crate::{snark::block_verify::SnarkBlockVerifyAction, Action};

use super::{ConsensusAction, ConsensusActionWithMeta};

pub fn consensus_effects<S: crate::Service>(store: &mut Store<S>, action: ConsensusActionWithMeta) {
    let (action, _) = action.split();

    match action {
        ConsensusAction::BlockReceived { hash, block, .. } => {
            let req_id = store.state().snark.block_verify.next_req_id();
            store.dispatch(SnarkBlockVerifyAction::Init {
                req_id,
                block: (hash.clone(), block).into(),
                verify_success_cb: redux::Callback::new(|args| {
                    let hash = *args.downcast().expect("correct arguments");
                    Box::<Action>::new(ConsensusAction::BlockSnarkVerifySuccess { hash }.into())
                }),
            });
            store.dispatch(ConsensusAction::BlockSnarkVerifyPending { req_id, hash });
        }
        ConsensusAction::BlockChainProofUpdate { hash, .. } => {
            if store.state().consensus.best_tip.as_ref() == Some(&hash) {
                transition_frontier_new_best_tip(store);
            }
        }
        ConsensusAction::BlockSnarkVerifyPending { .. } => {}
        ConsensusAction::BlockSnarkVerifySuccess { hash } => {
            store.dispatch(ConsensusAction::DetectForkRange { hash });
        }
        ConsensusAction::DetectForkRange { hash } => {
            store.dispatch(ConsensusAction::ShortRangeForkResolve { hash: hash.clone() });
            store.dispatch(ConsensusAction::LongRangeForkResolve { hash });
        }
        ConsensusAction::ShortRangeForkResolve { hash } => {
            store.dispatch(ConsensusAction::BestTipUpdate { hash });
        }
        ConsensusAction::LongRangeForkResolve { hash } => {
            store.dispatch(ConsensusAction::BestTipUpdate { hash });
        }
        ConsensusAction::BestTipUpdate { .. } => {
            let Some(block) = store.state.get().consensus.best_tip_block_with_hash() else {
                return;
            };
            for pub_key in store.state().watched_accounts.accounts() {
                store.dispatch(WatchedAccountsAction::LedgerInitialStateGetInit {
                    pub_key: pub_key.clone(),
                });
                store.dispatch(WatchedAccountsAction::TransactionsIncludedInBlock {
                    pub_key,
                    block: block.clone(),
                });
            }

            transition_frontier_new_best_tip(store);
        }
        ConsensusAction::Prune => {}
    }
}

fn transition_frontier_new_best_tip<S: crate::Service>(store: &mut Store<S>) {
    let state = store.state();
    let Some(best_tip) = state.consensus.best_tip_block_with_hash() else {
        return;
    };
    let pred_hash = best_tip.pred_hash();

    let Some((blocks_inbetween, root_block)) =
        state.consensus.best_tip_chain_proof.clone().or_else(|| {
            let old_best_tip = state.transition_frontier.best_tip()?;
            let mut iter = state.transition_frontier.best_chain.iter();
            if old_best_tip.hash() == pred_hash {
                iter.next();
                let root_block = iter.next()?.clone();
                let hashes = iter.map(|b| b.hash.clone()).collect();
                Some((hashes, root_block))
            } else if old_best_tip.pred_hash() == pred_hash {
                let root_block = iter.next()?.clone();
                let hashes = iter.rev().skip(1).rev().map(|b| b.hash.clone()).collect();
                Some((hashes, root_block))
            } else {
                None
            }
        })
    else {
        return;
    };

    if !state.transition_frontier.sync.is_pending() && !state.transition_frontier.sync.is_synced() {
        store.dispatch(TransitionFrontierSyncAction::Init {
            best_tip,
            root_block,
            blocks_inbetween,
        });
    } else {
        store.dispatch(TransitionFrontierSyncAction::BestTipUpdate {
            best_tip,
            root_block,
            blocks_inbetween,
        });
    }
}
