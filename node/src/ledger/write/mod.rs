mod ledger_write_actions;
pub use ledger_write_actions::*;

mod ledger_write_state;
pub use ledger_write_state::*;

mod ledger_write_reducer;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use ledger::scan_state::scan_state::transaction_snark::OneOrTwo;
use ledger::scan_state::scan_state::AvailableJobMessage;
use mina_p2p_messages::v2;
use serde::{Deserialize, Serialize};

use crate::block_producer::StagedLedgerDiffCreateOutput;
use crate::core::block::ArcBlockWithHash;
use crate::core::snark::{Snark, SnarkJobId};
use crate::transition_frontier::sync::ledger::staged::StagedLedgerAuxAndPendingCoinbasesValid;
use crate::transition_frontier::sync::TransitionFrontierRootSnarkedLedgerUpdates;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub enum LedgerWriteKind {
    StagedLedgerReconstruct,
    StagedLedgerDiffCreate,
    BlockApply,
    Commit,
}

#[derive(strum_macros::Display, Serialize, Deserialize, Debug, Clone)]
pub enum LedgerWriteRequest {
    StagedLedgerReconstruct {
        snarked_ledger_hash: v2::LedgerHash,
        parts: Option<Arc<StagedLedgerAuxAndPendingCoinbasesValid>>,
    },
    StagedLedgerDiffCreate {
        pred_block: ArcBlockWithHash,
        global_slot_since_genesis: v2::MinaNumbersGlobalSlotSinceGenesisMStableV1,
        producer: v2::NonZeroCurvePoint,
        delegator: v2::NonZeroCurvePoint,
        coinbase_receiver: v2::NonZeroCurvePoint,
        completed_snarks: BTreeMap<SnarkJobId, Snark>,
        supercharge_coinbase: bool,
    },
    BlockApply {
        block: ArcBlockWithHash,
        pred_block: ArcBlockWithHash,
    },
    Commit {
        ledgers_to_keep: BTreeSet<v2::LedgerHash>,
        root_snarked_ledger_updates: TransitionFrontierRootSnarkedLedgerUpdates,
        needed_protocol_states: BTreeMap<v2::StateHash, v2::MinaStateProtocolStateValueStableV2>,
        new_root: ArcBlockWithHash,
        new_best_tip: ArcBlockWithHash,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LedgerWriteResponse {
    StagedLedgerReconstruct {
        staged_ledger_hash: v2::LedgerHash,
        result: Result<(), String>,
    },
    StagedLedgerDiffCreate {
        pred_block_hash: v2::StateHash,
        global_slot_since_genesis: v2::MinaNumbersGlobalSlotSinceGenesisMStableV1,
        result: Result<Box<StagedLedgerDiffCreateOutput>, String>,
    },
    BlockApply {
        block_hash: v2::StateHash,
        result: Result<(), String>,
    },
    Commit {
        best_tip_hash: v2::StateHash,
        result: CommitResult,
    },
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct CommitResult {
    pub available_jobs: Vec<OneOrTwo<AvailableJobMessage>>,
    pub needed_protocol_states: BTreeSet<v2::StateHash>,
}

impl LedgerWriteRequest {
    pub fn kind(&self) -> LedgerWriteKind {
        match self {
            Self::StagedLedgerReconstruct { .. } => LedgerWriteKind::StagedLedgerReconstruct,
            Self::StagedLedgerDiffCreate { .. } => LedgerWriteKind::StagedLedgerDiffCreate,
            Self::BlockApply { .. } => LedgerWriteKind::BlockApply,
            Self::Commit { .. } => LedgerWriteKind::Commit,
        }
    }
}

impl LedgerWriteResponse {
    pub fn kind(&self) -> LedgerWriteKind {
        match self {
            Self::StagedLedgerReconstruct { .. } => LedgerWriteKind::StagedLedgerReconstruct,
            Self::StagedLedgerDiffCreate { .. } => LedgerWriteKind::StagedLedgerDiffCreate,
            Self::BlockApply { .. } => LedgerWriteKind::BlockApply,
            Self::Commit { .. } => LedgerWriteKind::Commit,
        }
    }
}
