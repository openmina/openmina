use std::collections::{BTreeMap, VecDeque};

use mina_p2p_messages::v2::{
    LedgerHash, MinaBaseAccountBinableArgStableV2, MinaBaseTransactionStatusStableV2,
    MinaBaseUserCommandStableV2, NonZeroCurvePoint, StateHash, TransactionHash,
};
use serde::{Deserialize, Serialize};

use crate::p2p::PeerId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WatchedAccountBlockInfo {
    pub level: u32,
    pub hash: StateHash,
    pub pred_hash: StateHash,
    pub staged_ledger_hash: LedgerHash,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub hash: Option<TransactionHash>,
    pub data: MinaBaseUserCommandStableV2,
    pub status: MinaBaseTransactionStatusStableV2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WatchedAccountsLedgerInitialStateGetError {
    PeerDisconnected,
    // TransportError(P2pRpcOutgoingError),
    P2pRpcError(mina_p2p_messages::core::Info),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "state")]
pub enum WatchedAccountLedgerInitialState {
    Idle {
        time: redux::Timestamp,
    },
    Pending {
        time: redux::Timestamp,
        block: WatchedAccountBlockInfo,
        peer_id: PeerId,
    },
    Error {
        time: redux::Timestamp,
        error: WatchedAccountsLedgerInitialStateGetError,
        peer_id: PeerId,
    },
    Success {
        time: redux::Timestamp,
        block: WatchedAccountBlockInfo,
        data: Option<Box<MinaBaseAccountBinableArgStableV2>>,
    },
}

impl WatchedAccountLedgerInitialState {
    pub fn block(&self) -> Option<&WatchedAccountBlockInfo> {
        match self {
            Self::Pending { block, .. } => Some(block),
            _ => None,
        }
    }

    pub fn data(&self) -> Option<&MinaBaseAccountBinableArgStableV2> {
        match self {
            Self::Success { data, .. } => data.as_ref().map(AsRef::as_ref),
            _ => None,
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "state")]
pub enum WatchedAccountBlockState {
    /// Relevant transactions to the account has been included in the block.
    TransactionsInBlockBody {
        block: WatchedAccountBlockInfo,
        /// Transactions included in the block ordered by nonce from low to high.
        transactions: Vec<Transaction>,
    },
    /// Get account data from the ledger pending.
    LedgerAccountGetPending {
        block: WatchedAccountBlockInfo,
        /// Transactions included in the block ordered by nonce from low to high.
        transactions: Vec<Transaction>,
    },
    /// Get account data from the ledger success.
    LedgerAccountGetSuccess {
        block: WatchedAccountBlockInfo,
        /// Transactions included in the block ordered by nonce from low to high.
        transactions: Vec<Transaction>,
        ledger_account: Box<MinaBaseAccountBinableArgStableV2>,
    },
}

impl WatchedAccountBlockState {
    pub fn block(&self) -> &WatchedAccountBlockInfo {
        match self {
            Self::TransactionsInBlockBody { block, .. } => block,
            Self::LedgerAccountGetPending { block, .. } => block,
            Self::LedgerAccountGetSuccess { block, .. } => block,
        }
    }

    pub fn transactions(&self) -> &[Transaction] {
        match self {
            Self::TransactionsInBlockBody { transactions, .. } => transactions,
            Self::LedgerAccountGetPending { transactions, .. } => transactions,
            Self::LedgerAccountGetSuccess { transactions, .. } => transactions,
        }
    }

    pub fn ledger_account(&self) -> Option<&MinaBaseAccountBinableArgStableV2> {
        match self {
            Self::TransactionsInBlockBody { .. } => None,
            Self::LedgerAccountGetPending { .. } => None,
            Self::LedgerAccountGetSuccess { ledger_account, .. } => Some(ledger_account),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WatchedAccountState {
    pub initial_state: WatchedAccountLedgerInitialState,

    /// Blocks in which account updates has happened.
    pub blocks: VecDeque<WatchedAccountBlockState>,
    // /// Pending transactions which haven't been included in any blocks.
    // pub pending_transactions: BTreeMap<txhash, tx>,
}

impl WatchedAccountState {
    pub fn block_find_by_hash(&self, hash: &StateHash) -> Option<&WatchedAccountBlockState> {
        self.blocks.iter().rev().find(|b| &b.block().hash == hash)
    }

    pub fn block_find_by_hash_mut(
        &mut self,
        hash: &StateHash,
    ) -> Option<&mut WatchedAccountBlockState> {
        self.blocks
            .iter_mut()
            .rev()
            .find(|b| &b.block().hash == hash)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WatchedAccountsState {
    list: BTreeMap<NonZeroCurvePoint, WatchedAccountState>,
}

impl WatchedAccountsState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains(&self, key: &NonZeroCurvePoint) -> bool {
        self.list.contains_key(key)
    }

    pub fn get(&self, key: &NonZeroCurvePoint) -> Option<&WatchedAccountState> {
        self.list.get(key)
    }

    pub fn get_mut(&mut self, key: &NonZeroCurvePoint) -> Option<&mut WatchedAccountState> {
        self.list.get_mut(key)
    }

    pub fn insert(&mut self, key: NonZeroCurvePoint, value: WatchedAccountState) {
        self.list.insert(key, value);
    }

    pub fn iter(
        &self,
    ) -> impl '_ + Iterator<Item = (&'_ NonZeroCurvePoint, &'_ WatchedAccountState)> {
        self.list.iter()
    }

    pub fn accounts(&self) -> Vec<NonZeroCurvePoint> {
        self.iter().map(|v| v.0.clone()).collect()
    }
}
