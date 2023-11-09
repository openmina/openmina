use crate::watched_accounts::WatchedAccountsLedgerInitialStateGetInitAction;
use crate::Store;
use crate::{
    snark::block_verify::SnarkBlockVerifyInitAction,
    watched_accounts::WatchedAccountsBlockTransactionsIncludedAction,
};

use super::{
    ConsensusAction, ConsensusActionWithMeta, ConsensusBestTipUpdateAction,
    ConsensusBlockSnarkVerifyPendingAction, ConsensusShortRangeForkResolveAction,
};

pub fn consensus_effects<S: redux::Service>(store: &mut Store<S>, action: ConsensusActionWithMeta) {
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
        ConsensusAction::BlockSnarkVerifyPending(_) => {}
        ConsensusAction::BlockSnarkVerifySuccess(a) => {
            store.dispatch(ConsensusShortRangeForkResolveAction { hash: a.hash });
        }
        ConsensusAction::ShortRangeForkResolve(a) => {
            store.dispatch(ConsensusBestTipUpdateAction { hash: a.hash });
        }
        ConsensusAction::BestTipUpdate(_) => {
            if let Some(block) = store.state().consensus.best_tip_block_with_hash() {
                for account_id in store.state().watched_accounts.accounts() {
                    store.dispatch(WatchedAccountsLedgerInitialStateGetInitAction {
                        account_id: account_id.clone(),
                    });
                    store.dispatch(WatchedAccountsBlockTransactionsIncludedAction {
                        account_id: account_id,
                        block: block.clone(),
                    });
                }
            }
        }
        ConsensusAction::BestTipHistoryUpdate(_) => {}
    }
}
