use crate::transition_frontier::sync::{
    TransitionFrontierSyncBestTipUpdateAction, TransitionFrontierSyncInitAction,
};
use crate::watched_accounts::WatchedAccountsLedgerInitialStateGetInitAction;
use crate::Store;
use crate::{
    snark::block_verify::SnarkBlockVerifyInitAction,
    watched_accounts::WatchedAccountsBlockTransactionsIncludedAction,
};

use super::{
    ConsensusAction, ConsensusActionWithMeta, ConsensusBestTipUpdateAction,
    ConsensusBlockSnarkVerifyPendingAction, ConsensusDetectForkRangeAction,
    ConsensusLongRangeForkResolveAction, ConsensusShortRangeForkResolveAction,
};

pub fn consensus_effects<S: crate::Service>(store: &mut Store<S>, action: ConsensusActionWithMeta) {
    let (action, _) = action.split();

    match action {
        ConsensusAction::BlockReceived(action) => {
            let req_id = store.state().snark.block_verify.next_req_id();
            store.dispatch(SnarkBlockVerifyInitAction {
                req_id,
                block: (action.hash.clone(), action.block).into(),
            });
            store.dispatch(ConsensusBlockSnarkVerifyPendingAction {
                req_id,
                hash: action.hash,
            });
        }
        ConsensusAction::BlockChainProofUpdate(a) => {
            if store.state().consensus.best_tip.as_ref() == Some(&a.hash) {
                transition_frontier_new_best_tip(store);
            }
        }
        ConsensusAction::BlockSnarkVerifyPending(_) => {}
        ConsensusAction::BlockSnarkVerifySuccess(a) => {
            store.dispatch(ConsensusDetectForkRangeAction { hash: a.hash });
        }
        ConsensusAction::DetectForkRange(a) => {
            store.dispatch(ConsensusShortRangeForkResolveAction {
                hash: a.hash.clone(),
            });
            store.dispatch(ConsensusLongRangeForkResolveAction { hash: a.hash });
        }
        ConsensusAction::ShortRangeForkResolve(a) => {
            store.dispatch(ConsensusBestTipUpdateAction { hash: a.hash });
        }
        ConsensusAction::LongRangeForkResolve(a) => {
            store.dispatch(ConsensusBestTipUpdateAction { hash: a.hash });
        }
        ConsensusAction::BestTipUpdate(_) => {
            let Some(block) = store.state.get().consensus.best_tip_block_with_hash() else {
                return;
            };
            for pub_key in store.state().watched_accounts.accounts() {
                store.dispatch(WatchedAccountsLedgerInitialStateGetInitAction {
                    pub_key: pub_key.clone(),
                });
                store.dispatch(WatchedAccountsBlockTransactionsIncludedAction {
                    pub_key,
                    block: block.clone(),
                });
            }

            transition_frontier_new_best_tip(store);
        }
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
        println!(
            "+++ BEST TIP INIT, root snarked ledger: {}",
            root_block.snarked_ledger_hash().to_string()
        );
        store.dispatch(TransitionFrontierSyncInitAction {
            best_tip,
            root_block,
            blocks_inbetween,
        });
    } else {
        println!(
            "+++ BEST TIP UPDATE, root snarked ledger: {}",
            root_block.snarked_ledger_hash().to_string()
        );
        store.dispatch(TransitionFrontierSyncBestTipUpdateAction {
            best_tip,
            root_block,
            blocks_inbetween,
        });
    }
}
