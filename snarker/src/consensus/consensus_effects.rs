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
                for pub_key in store.state().watched_accounts.accounts() {
                    store.dispatch(WatchedAccountsLedgerInitialStateGetInitAction {
                        pub_key: pub_key.clone(),
                    });
                    store.dispatch(WatchedAccountsBlockTransactionsIncludedAction {
                        pub_key,
                        block: block.clone(),
                    });
                }
            }
        }
        ConsensusAction::BestTipHistoryUpdate(_) => {}
    }
}
