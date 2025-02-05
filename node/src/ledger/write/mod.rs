mod ledger_write_actions;
use ledger::scan_state::transaction_logic::valid;
use ledger::{Account, AccountId, AccountIndex, TokenId};
pub use ledger_write_actions::*;

mod ledger_write_state;
pub use ledger_write_state::*;
use openmina_core::block::AppliedBlock;

mod ledger_write_reducer;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use ledger::scan_state::scan_state::transaction_snark::OneOrTwo;
use ledger::scan_state::scan_state::AvailableJobMessage;
use mina_p2p_messages::v2::{self, StateBodyHash};
use serde::{Deserialize, Serialize};

use crate::block_producer_effectful::StagedLedgerDiffCreateOutput;
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LedgerWriteRequest {
    StagedLedgerReconstruct {
        snarked_ledger_hash: v2::LedgerHash,
        parts: Option<Arc<StagedLedgerAuxAndPendingCoinbasesValid>>,
    },
    StagedLedgerDiffCreate {
        pred_block: AppliedBlock,
        global_slot_since_genesis: v2::MinaNumbersGlobalSlotSinceGenesisMStableV1,
        is_new_epoch: bool,
        producer: v2::NonZeroCurvePoint,
        delegator: v2::NonZeroCurvePoint,
        coinbase_receiver: v2::NonZeroCurvePoint,
        completed_snarks: BTreeMap<SnarkJobId, Snark>,
        supercharge_coinbase: bool,
        transactions_by_fee: Vec<valid::UserCommand>,
    },
    BlockApply {
        block: ArcBlockWithHash,
        pred_block: AppliedBlock,
        skip_verification: bool,
    },
    Commit {
        ledgers_to_keep: LedgersToKeep,
        root_snarked_ledger_updates: TransitionFrontierRootSnarkedLedgerUpdates,
        needed_protocol_states: BTreeMap<v2::StateHash, v2::MinaStateProtocolStateValueStableV2>,
        new_root: AppliedBlock,
        new_best_tip: AppliedBlock,
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
        result: Result<Arc<StagedLedgerDiffCreateOutput>, String>,
    },
    BlockApply {
        block_hash: v2::StateHash,
        result: Result<BlockApplyResult, String>,
    },
    Commit {
        best_tip_hash: v2::StateHash,
        result: CommitResult,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockApplyResult {
    pub block: ArcBlockWithHash,
    pub just_emitted_a_proof: bool,
    pub archive_data: Option<BlockApplyResultArchive>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockApplyResultArchive {
    pub accounts_accessed: Vec<(AccountIndex, Account)>,
    pub accounts_created: Vec<(AccountId, u64)>,
    pub tokens_used: BTreeSet<(TokenId, Option<AccountId>)>,
    pub sender_receipt_chains_from_parent_ledger: Vec<(AccountId, v2::ReceiptChainHash)>,
}

impl TryFrom<&BlockApplyResult> for v2::ArchiveTransitionFronntierDiff {
    type Error = String;

    fn try_from(value: &BlockApplyResult) -> Result<Self, Self::Error> {
        if let Some(archive_data) = &value.archive_data {
            let res = Self::BreadcrumbAdded {
                // TODO(adonagy): check if we need the StateBodyHash, if no keep the None
                block: (
                    (*value.block.block).clone(),
                    (
                        value
                            .block
                            .header()
                            .protocol_state
                            .body
                            .try_hash()
                            .ok()
                            .map(StateBodyHash::from),
                        value.block.hash().clone(),
                    ),
                ),
                accounts_accessed: archive_data
                    .accounts_accessed
                    .iter()
                    .map(|(index, account)| (index.0.into(), account.into()))
                    .collect(),
                accounts_created: archive_data
                    .accounts_created
                    .iter()
                    .map(|(account_id, fee)| {
                        (
                            (*account_id).clone().into(),
                            v2::CurrencyFeeStableV1((*fee).into()),
                        )
                    })
                    .collect(),
                tokens_used: archive_data
                    .tokens_used
                    .iter()
                    .map(|(token_id, account_id)| {
                        (
                            token_id.into(),
                            account_id.clone().map(|account_id| account_id.into()),
                        )
                    })
                    .collect(),
                sender_receipt_chains_from_parent_ledger: archive_data
                    .sender_receipt_chains_from_parent_ledger
                    .iter()
                    .map(|(account_id, receipt_chain_hash)| {
                        (
                            (*account_id).clone().into(),
                            receipt_chain_hash.clone().into_inner(),
                        )
                    })
                    .collect(),
            };
            Ok(res)
        } else {
            Err("Archive data not available, not running in archive mode".to_string())
        }
    }
}

impl TryFrom<&BlockApplyResult> for v2::PrecomputedBlock {
    type Error = String;

    fn try_from(value: &BlockApplyResult) -> Result<Self, Self::Error> {
        let archive_transition_frontier_diff: v2::ArchiveTransitionFronntierDiff =
            value.try_into()?;

        let res = Self {
            scheduled_time: value
                .block
                .header()
                .protocol_state
                .body
                .blockchain_state
                .timestamp,
            protocol_state: value.block.header().protocol_state.clone(),
            protocol_state_proof: value
                .block
                .header()
                .protocol_state_proof
                .as_ref()
                .clone()
                .into(),
            staged_ledger_diff: value.block.body().staged_ledger_diff.clone(),
            // TODO(adonagy): add the actual delta transition chain proof
            delta_transition_chain_proof: (
                mina_p2p_messages::v2::LedgerHash::zero(),
                mina_p2p_messages::list::List::new(),
            ),
            protocol_version: value.block.header().current_protocol_version.clone(),
            proposed_protocol_version: None,
            accounts_accessed: archive_transition_frontier_diff.accounts_accessed(),
            accounts_created: archive_transition_frontier_diff.accounts_created(),
            tokens_used: archive_transition_frontier_diff.tokens_used(),
        };

        Ok(res)
    }
}

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Default, Clone)]
pub struct LedgersToKeep {
    snarked: BTreeSet<v2::LedgerHash>,
    staged: BTreeSet<Arc<v2::MinaBaseStagedLedgerHashStableV1>>,
}

impl LedgersToKeep {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains<'a, T>(&self, key: T) -> bool
    where
        T: 'a + Into<LedgerToKeep<'a>>,
    {
        match key.into() {
            LedgerToKeep::Snarked(hash) => self.snarked.contains(hash),
            LedgerToKeep::Staged(hash) => self.staged.contains(hash),
        }
    }

    pub fn add_snarked(&mut self, hash: v2::LedgerHash) -> bool {
        self.snarked.insert(hash)
    }

    pub fn add_staged(&mut self, hash: Arc<v2::MinaBaseStagedLedgerHashStableV1>) -> bool {
        self.staged.insert(hash)
    }
}

impl<'a> FromIterator<&'a ArcBlockWithHash> for LedgersToKeep {
    fn from_iter<T: IntoIterator<Item = &'a ArcBlockWithHash>>(iter: T) -> Self {
        let mut res = Self::new();
        let best_tip = iter.into_iter().fold(None, |best_tip, block| {
            res.add_snarked(block.snarked_ledger_hash().clone());
            res.add_staged(Arc::new(block.staged_ledger_hashes().clone()));
            match best_tip {
                None => Some(block),
                Some(tip) if tip.height() < block.height() => Some(block),
                old => old,
            }
        });

        if let Some(best_tip) = best_tip {
            res.add_snarked(best_tip.staking_epoch_ledger_hash().clone());
            res.add_snarked(best_tip.next_epoch_ledger_hash().clone());
        }

        res
    }
}

#[derive(derive_more::From)]
pub enum LedgerToKeep<'a> {
    Snarked(&'a v2::LedgerHash),
    Staged(&'a v2::MinaBaseStagedLedgerHashStableV1),
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct CommitResult {
    pub alive_masks: usize,
    pub available_jobs: Arc<Vec<OneOrTwo<AvailableJobMessage>>>,
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
