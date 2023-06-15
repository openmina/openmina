use crate::watched_accounts::WatchedAccountsLedgerInitialStateGetInitAction;
use crate::Store;
use crate::{
    p2p::channels::best_tip::P2pChannelsBestTipResponseSendAction,
    snark::block_verify::SnarkBlockVerifyInitAction,
    watched_accounts::WatchedAccountsBlockTransactionsIncludedAction,
};

use super::{
    ConsensusAction, ConsensusActionWithMeta, ConsensusBestTipUpdateAction,
    ConsensusBlockSnarkVerifyPendingAction, ConsensusShortRangeForkResolveAction,
};

pub fn consensus_effects<S: crate::Service>(store: &mut Store<S>, action: ConsensusActionWithMeta) {
    let (action, meta) = action.split();

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
            if let Some(block) = store.state.get().consensus.best_tip_block_with_hash() {
                if let Some(stats) = store.service.stats() {
                    // TODO(binier): call this once block is our best tip,
                    // meaning we have synced to it.
                    stats.new_best_tip(meta.time(), &block);
                }
                for pub_key in store.state().watched_accounts.accounts() {
                    store.dispatch(WatchedAccountsLedgerInitialStateGetInitAction {
                        pub_key: pub_key.clone(),
                    });
                    store.dispatch(WatchedAccountsBlockTransactionsIncludedAction {
                        pub_key,
                        block: block.clone(),
                    });
                }

                for peer_id in store.state().p2p.ready_peers() {
                    store.dispatch(P2pChannelsBestTipResponseSendAction {
                        peer_id,
                        best_tip: block.clone(),
                    });
                }
            }
        }
        ConsensusAction::BestTipHistoryUpdate(_) => {}
    }
}
