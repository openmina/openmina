use std::sync::Arc;

use mina_p2p_messages::v2::{MinaBaseAccountBinableArgStableV2, MinaBlockBlockStableV2, StateHash};
use serde::{Deserialize, Serialize};
use shared::block::BlockWithHash;

use crate::p2p::rpc::P2pRpcId;
use crate::p2p::PeerId;

use super::{
    WatchedAccountBlockInfo, WatchedAccountBlockState, WatchedAccountId,
    WatchedAccountLedgerInitialState, WatchedAccountsLedgerInitialStateGetError,
};

pub type WatchedAccountsActionWithMeta = redux::ActionWithMeta<WatchedAccountsAction>;
pub type WatchedAccountsActionWithMetaRef<'a> = redux::ActionWithMeta<&'a WatchedAccountsAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum WatchedAccountsAction {
    Add(WatchedAccountsAddAction),

    LedgerInitialStateGetInit(WatchedAccountsLedgerInitialStateGetInitAction),
    LedgerInitialStateGetPending(WatchedAccountsLedgerInitialStateGetPendingAction),
    LedgerInitialStateGetError(WatchedAccountsLedgerInitialStateGetErrorAction),
    LedgerInitialStateGetRetry(WatchedAccountsLedgerInitialStateGetRetryAction),
    LedgerInitialStateGetSuccess(WatchedAccountsLedgerInitialStateGetSuccessAction),

    TransactionsIncludedInBlock(WatchedAccountsBlockTransactionsIncludedAction),
    BlockLedgerQueryInit(WatchedAccountsBlockLedgerQueryInitAction),
    BlockLedgerQueryPending(WatchedAccountsBlockLedgerQueryPendingAction),
    BlockLedgerQuerySuccess(WatchedAccountsBlockLedgerQuerySuccessAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WatchedAccountsAddAction {
    pub account_id: WatchedAccountId,
}

impl redux::EnablingCondition<crate::State> for WatchedAccountsAddAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.watched_accounts.get(&self.account_id).is_none()
    }
}

fn should_request_ledger_initial_state(
    state: &crate::State,
    account_id: &WatchedAccountId,
) -> bool {
    state
        .watched_accounts
        .get(account_id)
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
            WatchedAccountLedgerInitialState::Success { block, .. } => !state
                .consensus
                .is_part_of_main_chain(block.level, &block.hash)
                .unwrap_or(true),
        })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WatchedAccountsLedgerInitialStateGetInitAction {
    pub account_id: WatchedAccountId,
}

impl redux::EnablingCondition<crate::State> for WatchedAccountsLedgerInitialStateGetInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        should_request_ledger_initial_state(state, &self.account_id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WatchedAccountsLedgerInitialStateGetPendingAction {
    pub account_id: WatchedAccountId,
    pub block: WatchedAccountBlockInfo,
    pub peer_id: PeerId,
    pub p2p_rpc_id: P2pRpcId,
}

impl redux::EnablingCondition<crate::State> for WatchedAccountsLedgerInitialStateGetPendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        should_request_ledger_initial_state(state, &self.account_id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WatchedAccountsLedgerInitialStateGetErrorAction {
    pub account_id: WatchedAccountId,
    pub error: WatchedAccountsLedgerInitialStateGetError,
}

impl redux::EnablingCondition<crate::State> for WatchedAccountsLedgerInitialStateGetErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .watched_accounts
            .get(&self.account_id)
            .map_or(false, |a| {
                matches!(
                    &a.initial_state,
                    WatchedAccountLedgerInitialState::Pending { .. }
                )
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WatchedAccountsLedgerInitialStateGetRetryAction {
    pub account_id: WatchedAccountId,
}

impl redux::EnablingCondition<crate::State> for WatchedAccountsLedgerInitialStateGetRetryAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .watched_accounts
            .get(&self.account_id)
            .map_or(false, |a| match &a.initial_state {
                WatchedAccountLedgerInitialState::Error { time, .. } => state
                    .time()
                    .checked_sub(*time)
                    .map_or(false, |d| d.as_secs() >= 3),
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WatchedAccountsLedgerInitialStateGetSuccessAction {
    pub account_id: WatchedAccountId,
    pub data: Option<MinaBaseAccountBinableArgStableV2>,
}

impl redux::EnablingCondition<crate::State> for WatchedAccountsLedgerInitialStateGetSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .watched_accounts
            .get(&self.account_id)
            .map_or(false, |a| {
                matches!(
                    &a.initial_state,
                    WatchedAccountLedgerInitialState::Pending { .. }
                )
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WatchedAccountsBlockTransactionsIncludedAction {
    pub account_id: WatchedAccountId,
    pub block: BlockWithHash<Arc<MinaBlockBlockStableV2>>,
}

impl redux::EnablingCondition<crate::State> for WatchedAccountsBlockTransactionsIncludedAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        let diff = &self.block.block.body.staged_ledger_diff.diff;
        state
            .watched_accounts
            .get(&self.account_id)
            .map_or(false, |v| v.initial_state.is_success())
            && super::account_relevant_transactions_in_diff_iter(&self.account_id, diff)
                .any(|_| true)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WatchedAccountsBlockLedgerQueryInitAction {
    pub account_id: WatchedAccountId,
    pub block_hash: StateHash,
}

impl redux::EnablingCondition<crate::State> for WatchedAccountsBlockLedgerQueryInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        let Some(acc) = state.watched_accounts.get(&self.account_id) else {
            return false;
        };
        acc.blocks
            .iter()
            .rev()
            .find(|b| b.block().hash == self.block_hash)
            .filter(|b| matches!(b, WatchedAccountBlockState::TransactionsInBlockBody { .. }))
            .is_some()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WatchedAccountsBlockLedgerQueryPendingAction {
    pub account_id: WatchedAccountId,
    pub block_hash: StateHash,
    pub peer_id: PeerId,
    pub p2p_rpc_id: P2pRpcId,
}

impl redux::EnablingCondition<crate::State> for WatchedAccountsBlockLedgerQueryPendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        let Some(acc) = state.watched_accounts.get(&self.account_id) else {
            return false;
        };

        let should_req_for_block = acc
            .block_find_by_hash(&self.block_hash)
            .filter(|b| matches!(b, WatchedAccountBlockState::TransactionsInBlockBody { .. }))
            .is_some();

        let p2p_rpc_is_pending = None
            .or_else(|| {
                let peer = state.p2p.get_ready_peer(&self.peer_id)?;
                peer.rpc.outgoing.get(self.p2p_rpc_id)
            })
            .map_or(false, |v| v.is_init() || v.is_pending());

        should_req_for_block && p2p_rpc_is_pending
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WatchedAccountsBlockLedgerQuerySuccessAction {
    pub account_id: WatchedAccountId,
    pub block_hash: StateHash,
    pub ledger_account: MinaBaseAccountBinableArgStableV2,
}

impl redux::EnablingCondition<crate::State> for WatchedAccountsBlockLedgerQuerySuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        let Some(acc) = state.watched_accounts.get(&self.account_id) else {
            return false;
        };

        acc.block_find_by_hash(&self.block_hash)
            .filter(|b| matches!(b, WatchedAccountBlockState::LedgerAccountGetPending { .. }))
            .is_some()
    }
}

macro_rules! impl_into_global_action {
    ($a:ty) => {
        impl From<$a> for crate::Action {
            fn from(value: $a) -> Self {
                Self::WatchedAccounts(value.into())
            }
        }
    };
}

impl_into_global_action!(WatchedAccountsAddAction);

impl_into_global_action!(WatchedAccountsLedgerInitialStateGetInitAction);
impl_into_global_action!(WatchedAccountsLedgerInitialStateGetPendingAction);
impl_into_global_action!(WatchedAccountsLedgerInitialStateGetErrorAction);
impl_into_global_action!(WatchedAccountsLedgerInitialStateGetRetryAction);
impl_into_global_action!(WatchedAccountsLedgerInitialStateGetSuccessAction);

impl_into_global_action!(WatchedAccountsBlockTransactionsIncludedAction);
impl_into_global_action!(WatchedAccountsBlockLedgerQueryInitAction);
impl_into_global_action!(WatchedAccountsBlockLedgerQueryPendingAction);
impl_into_global_action!(WatchedAccountsBlockLedgerQuerySuccessAction);
