use std::sync::Arc;

use mina_p2p_messages::v2::{
    MinaBaseAccountBinableArgStableV2, MinaBlockBlockStableV2, NonZeroCurvePoint, StateHash,
};
use openmina_core::block::BlockWithHash;
use serde::{Deserialize, Serialize};

use crate::p2p::PeerId;

use super::{
    WatchedAccountBlockInfo, WatchedAccountBlockState, WatchedAccountLedgerInitialState,
    WatchedAccountsLedgerInitialStateGetError,
};

pub type WatchedAccountsActionWithMeta = redux::ActionWithMeta<WatchedAccountsAction>;
pub type WatchedAccountsActionWithMetaRef<'a> = redux::ActionWithMeta<&'a WatchedAccountsAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WatchedAccountsAction {
    Add {
        pub_key: NonZeroCurvePoint,
    },
    LedgerInitialStateGetInit {
        pub_key: NonZeroCurvePoint,
    },
    LedgerInitialStateGetPending {
        pub_key: NonZeroCurvePoint,
        block: WatchedAccountBlockInfo,
        peer_id: PeerId,
    },
    LedgerInitialStateGetError {
        pub_key: NonZeroCurvePoint,
        error: WatchedAccountsLedgerInitialStateGetError,
    },
    LedgerInitialStateGetRetry {
        pub_key: NonZeroCurvePoint,
    },
    LedgerInitialStateGetSuccess {
        pub_key: NonZeroCurvePoint,
        data: Option<MinaBaseAccountBinableArgStableV2>,
    },
    TransactionsIncludedInBlock {
        pub_key: NonZeroCurvePoint,
        block: BlockWithHash<Arc<MinaBlockBlockStableV2>>,
    },
    BlockLedgerQueryInit {
        pub_key: NonZeroCurvePoint,
        block_hash: StateHash,
    },
    BlockLedgerQueryPending {
        pub_key: NonZeroCurvePoint,
        block_hash: StateHash,
        peer_id: PeerId,
    },
    BlockLedgerQuerySuccess {
        pub_key: NonZeroCurvePoint,
        block_hash: StateHash,
        ledger_account: MinaBaseAccountBinableArgStableV2,
    },
}

fn should_request_ledger_initial_state(state: &crate::State, pub_key: &NonZeroCurvePoint) -> bool {
    state
        .watched_accounts
        .get(pub_key)
        .filter(|_| state.consensus.best_tip.is_some())
        .map_or(false, |a| match &a.initial_state {
            WatchedAccountLedgerInitialState::Idle { .. } => true,
            WatchedAccountLedgerInitialState::Error { .. } => true,
            WatchedAccountLedgerInitialState::Pending { block, .. } => {
                let Some(best_tip) = state.consensus.best_tip() else {
                    return false;
                };
                &block.hash != best_tip.hash
            }
            // TODO(binier)
            WatchedAccountLedgerInitialState::Success { .. } => false,
            // WatchedAccountLedgerInitialState::Success { block, .. } => !state
            //     .consensus
            //     .is_part_of_main_chain(block.level, &block.hash)
            //     .unwrap_or(true),
        })
}

impl redux::EnablingCondition<crate::State> for WatchedAccountsAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        match self {
            WatchedAccountsAction::Add { pub_key } => state.watched_accounts.get(pub_key).is_none(),
            WatchedAccountsAction::LedgerInitialStateGetInit { pub_key } => {
                should_request_ledger_initial_state(state, pub_key)
            }
            WatchedAccountsAction::LedgerInitialStateGetPending { pub_key, .. } => {
                should_request_ledger_initial_state(state, pub_key)
            }
            WatchedAccountsAction::LedgerInitialStateGetError { pub_key, .. } => {
                state.watched_accounts.get(pub_key).map_or(false, |a| {
                    matches!(
                        &a.initial_state,
                        WatchedAccountLedgerInitialState::Pending { .. }
                    )
                })
            }
            WatchedAccountsAction::LedgerInitialStateGetRetry { pub_key } => state
                .watched_accounts
                .get(pub_key)
                .map_or(false, |a| match &a.initial_state {
                    WatchedAccountLedgerInitialState::Error { time: t, .. } => {
                        time.checked_sub(*t).map_or(false, |d| d.as_secs() >= 3)
                    }
                    _ => false,
                }),
            WatchedAccountsAction::LedgerInitialStateGetSuccess { pub_key, .. } => {
                state.watched_accounts.get(pub_key).map_or(false, |a| {
                    matches!(
                        &a.initial_state,
                        WatchedAccountLedgerInitialState::Pending { .. }
                    )
                })
            }
            WatchedAccountsAction::TransactionsIncludedInBlock { pub_key, block } => {
                let diff = &block.block.body.staged_ledger_diff.diff;
                state
                    .watched_accounts
                    .get(pub_key)
                    .map_or(false, |v| v.initial_state.is_success())
                    && super::account_relevant_transactions_in_diff_iter(pub_key, diff)
                        .any(|_| true)
            }
            WatchedAccountsAction::BlockLedgerQueryInit {
                pub_key,
                block_hash,
            } => {
                let Some(acc) = state.watched_accounts.get(pub_key) else {
                    return false;
                };
                acc.block_find_by_hash(&block_hash)
                    .filter(|b| {
                        matches!(b, WatchedAccountBlockState::TransactionsInBlockBody { .. })
                    })
                    .is_some()
            }
            WatchedAccountsAction::BlockLedgerQueryPending {
                pub_key,
                block_hash,
                ..
            } => {
                let Some(acc) = state.watched_accounts.get(pub_key) else {
                    return false;
                };

                let should_req_for_block = acc
                    .block_find_by_hash(block_hash)
                    .filter(|b| {
                        matches!(b, WatchedAccountBlockState::TransactionsInBlockBody { .. })
                    })
                    .is_some();

                // TODO(binier)
                let p2p_rpc_is_pending = false;
                // let p2p_rpc_is_pending = None
                //     .or_else(|| {
                //         let peer = state.p2p.get_ready_peer(&self.peer_id)?;
                //         peer.rpc.outgoing.get(self.p2p_rpc_id)
                //     })
                //     .map_or(false, |v| v.is_init() || v.is_pending());

                should_req_for_block && p2p_rpc_is_pending
            }
            WatchedAccountsAction::BlockLedgerQuerySuccess {
                pub_key,
                block_hash,
                ..
            } => {
                let Some(acc) = state.watched_accounts.get(pub_key) else {
                    return false;
                };

                acc.block_find_by_hash(block_hash)
                    .filter(|b| {
                        matches!(b, WatchedAccountBlockState::LedgerAccountGetPending { .. })
                    })
                    .is_some()
            }
        }
    }
}
