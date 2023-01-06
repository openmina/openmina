use std::sync::Arc;

use mina_p2p_messages::v2::{
    MinaBaseAccountBinableArgStableV2, MinaBlockBlockStableV2, NonZeroCurvePoint, StateHash,
};
use serde::{Deserialize, Serialize};
use shared::block::BlockWithHash;

use crate::p2p::rpc::P2pRpcId;
use crate::p2p::PeerId;

use super::WatchedAccountBlockState;

pub type WatchedAccountsActionWithMeta = redux::ActionWithMeta<WatchedAccountsAction>;
pub type WatchedAccountsActionWithMetaRef<'a> = redux::ActionWithMeta<&'a WatchedAccountsAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum WatchedAccountsAction {
    Add(WatchedAccountsAddAction),
    TransactionsIncludedInBlock(WatchedAccountsBlockTransactionsIncludedAction),
    BlockLedgerQueryInit(WatchedAccountsBlockLedgerQueryInitAction),
    BlockLedgerQueryPending(WatchedAccountsBlockLedgerQueryPendingAction),
    BlockLedgerQuerySuccess(WatchedAccountsBlockLedgerQuerySuccessAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WatchedAccountsAddAction {
    pub pub_key: NonZeroCurvePoint,
}

impl redux::EnablingCondition<crate::State> for WatchedAccountsAddAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.watched_accounts.get(&self.pub_key).is_none()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WatchedAccountsBlockTransactionsIncludedAction {
    pub pub_key: NonZeroCurvePoint,
    pub block: BlockWithHash<Arc<MinaBlockBlockStableV2>>,
}

impl redux::EnablingCondition<crate::State> for WatchedAccountsBlockTransactionsIncludedAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        let diff = &self.block.block.body.staged_ledger_diff.diff;
        state.watched_accounts.contains(&self.pub_key)
            && super::account_relevant_transactions_in_diff_iter(&self.pub_key, diff).any(|_| true)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WatchedAccountsBlockLedgerQueryInitAction {
    pub pub_key: NonZeroCurvePoint,
    pub block_hash: StateHash,
}

impl redux::EnablingCondition<crate::State> for WatchedAccountsBlockLedgerQueryInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        let Some(acc) = state.watched_accounts.get(&self.pub_key) else { return false };
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
    pub pub_key: NonZeroCurvePoint,
    pub block_hash: StateHash,
    pub peer_id: PeerId,
    pub p2p_rpc_id: P2pRpcId,
}

impl redux::EnablingCondition<crate::State> for WatchedAccountsBlockLedgerQueryPendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        let Some(acc) = state.watched_accounts.get(&self.pub_key) else { return false };

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
    pub pub_key: NonZeroCurvePoint,
    pub block_hash: StateHash,
    pub ledger_account: MinaBaseAccountBinableArgStableV2,
}

impl redux::EnablingCondition<crate::State> for WatchedAccountsBlockLedgerQuerySuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        let Some(acc) = state.watched_accounts.get(&self.pub_key) else { return false };

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
impl_into_global_action!(WatchedAccountsBlockTransactionsIncludedAction);
impl_into_global_action!(WatchedAccountsBlockLedgerQueryInitAction);
impl_into_global_action!(WatchedAccountsBlockLedgerQueryPendingAction);
impl_into_global_action!(WatchedAccountsBlockLedgerQuerySuccessAction);
