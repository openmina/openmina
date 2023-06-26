use std::collections::BTreeMap;

use mina_p2p_messages::v2::{
    LedgerHash, MinaBasePendingCoinbaseStableV2, MinaStateProtocolStateValueStableV2,
    TransactionSnarkScanStateStableV2,
};
use redux::Timestamp;
use serde::{Deserialize, Serialize};
use shared::block::ArcBlockWithHash;

use crate::ledger::LedgerAddress;
use crate::p2p::channels::rpc::P2pRpcId;
use crate::p2p::PeerId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransitionFrontierSyncLedgerState {
    Init {
        time: Timestamp,
        block: ArcBlockWithHash,
    },
    /// Doing BFS to sync snarked ledger tree.
    SnarkedLedgerSyncPending {
        time: Timestamp,
        block: ArcBlockWithHash,
        pending: BTreeMap<LedgerAddress, LedgerQueryPending>,
        /// `None` means we are done.
        next_addr: Option<LedgerAddress>,
    },
    SnarkedLedgerSyncSuccess {
        time: Timestamp,
        block: ArcBlockWithHash,
    },
    /// Fetching pieces required to reconstruct staged ledger from
    /// snarked ledger.
    StagedLedgerPartsFetchPending {
        time: Timestamp,
        block: ArcBlockWithHash,
        peer_id: PeerId,
    },
    /// Fetched pieces required to reconstruct staged ledger from
    /// snarked ledger.
    StagedLedgerPartsFetchSuccess {
        time: Timestamp,
        block: ArcBlockWithHash,
        staged_ledger_parts: StagedLedgerParts,
    },
    StagedLedgerReconstructPending {
        time: Timestamp,
        block: ArcBlockWithHash,
        staged_ledger_parts: StagedLedgerParts,
    },
    StagedLedgerReconstructSuccess {
        time: Timestamp,
        block: ArcBlockWithHash,
        staged_ledger_parts: StagedLedgerParts,
    },
    Success {
        time: Timestamp,
        block: ArcBlockWithHash,
    },
}

/// Pieces required to reconstruct staged ledger from snarked ledger.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StagedLedgerParts {
    pub scan_state: TransactionSnarkScanStateStableV2,
    pub staged_ledger_hash: LedgerHash,
    pub pending_coinbase: MinaBasePendingCoinbaseStableV2,
    pub needed_blocks: Vec<MinaStateProtocolStateValueStableV2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LedgerQueryPending {
    pub time: Timestamp,
    pub attempts: BTreeMap<PeerId, PeerRpcState>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerRpcState {
    Init { time: Timestamp },
    Pending { time: Timestamp, rpc_id: P2pRpcId },
}

impl TransitionFrontierSyncLedgerState {
    pub fn block(&self) -> &ArcBlockWithHash {
        match self {
            Self::Init { block, .. } => block,
            Self::SnarkedLedgerSyncPending { block, .. } => block,
            Self::SnarkedLedgerSyncSuccess { block, .. } => block,
            Self::StagedLedgerPartsFetchPending { block, .. } => block,
            Self::StagedLedgerPartsFetchSuccess { block, .. } => block,
            Self::StagedLedgerReconstructPending { block, .. } => block,
            Self::StagedLedgerReconstructSuccess { block, .. } => block,
            Self::Success { block, .. } => block,
        }
    }

    pub fn snarked_ledger_hash(&self) -> LedgerHash {
        self.block()
            .block
            .header
            .protocol_state
            .body
            .blockchain_state
            .ledger_proof_statement
            .target
            .first_pass_ledger
            .clone()
    }

    pub fn snarked_ledger_sync_next(&self) -> Option<LedgerAddress> {
        match self {
            Self::SnarkedLedgerSyncPending { next_addr, .. } => next_addr.clone(),
            _ => None,
        }
    }

    pub fn snarked_ledger_peer_query_get(
        &self,
        peer_id: &PeerId,
        rpc_id: P2pRpcId,
    ) -> Option<(&LedgerAddress, &LedgerQueryPending)> {
        match self {
            Self::SnarkedLedgerSyncPending { pending, .. } => {
                let expected_rpc_id = rpc_id;
                pending.iter().find(|(_, s)| {
                    s.attempts.get(peer_id).map_or(false, |s| match s {
                        PeerRpcState::Pending { rpc_id, .. } => *rpc_id == expected_rpc_id,
                        _ => false,
                    })
                })
            }
            _ => None,
        }
    }
}
