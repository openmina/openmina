use std::collections::{BTreeMap, VecDeque};

use ledger::AccountIndex;
use mina_p2p_messages::v2;
use openmina_core::block::{AppliedBlock, ArcBlockWithHash};
use serde::{Deserialize, Serialize};

use crate::{
    block_producer::{BlockProducerWonSlot, BlockProducerWonSlotDiscardReason, BlockWithoutProof},
    core::block::BlockHash,
};

const MAX_HISTORY: usize = 2048;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct BlockProducerStats {
    pub(super) attempts: VecDeque<BlockProductionAttempt>,
    pub vrf_evaluator: BTreeMap<u32, VrfEvaluatorStats>,
    pub last_produced_block: Option<ArcBlockWithHash>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProductionAttempt {
    pub won_slot: BlockProductionAttemptWonSlot,
    pub block: Option<ProducedBlock>,
    pub times: BlockProductionTimes,
    #[serde(flatten)]
    pub status: BlockProductionStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProductionAttemptWonSlot {
    pub slot_time: redux::Timestamp,
    pub global_slot: u32,
    pub epoch: u32,
    pub delegator: (v2::NonZeroCurvePoint, AccountIndex),
    pub value_with_threshold: Option<(f64, f64)>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProductionTimes {
    pub scheduled: redux::Timestamp,
    pub staged_ledger_diff_create_start: Option<redux::Timestamp>,
    pub staged_ledger_diff_create_end: Option<redux::Timestamp>,
    pub produced: Option<redux::Timestamp>,
    pub proof_create_start: Option<redux::Timestamp>,
    pub proof_create_end: Option<redux::Timestamp>,
    pub proof_create_error: Option<redux::Timestamp>,
    pub block_apply_start: Option<redux::Timestamp>,
    pub block_apply_end: Option<redux::Timestamp>,
    pub committed: Option<redux::Timestamp>,
    pub discarded: Option<redux::Timestamp>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "status")]
pub enum BlockProductionStatus {
    Scheduled,
    StagedLedgerDiffCreatePending,
    StagedLedgerDiffCreateSuccess,
    Produced,
    ProofCreatePending,
    ProofCreateSuccess,
    ProofCreateError {
        error: String,
    },
    BlockApplyPending,
    BlockApplySuccess,
    Committed,
    Canonical {
        last_observed_confirmations: u32,
    },
    Orphaned {
        orphaned_by: BlockHash,
    },
    Discarded {
        discard_reason: BlockProducerWonSlotDiscardReason,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProducedBlock {
    pub hash: BlockHash,
    pub height: u32,
    pub transactions: ProducedBlockTransactions,
    pub completed_works_count: usize,
    pub coinbase: u64,
    pub fees: u64,
    pub snark_fees: u64,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ProducedBlockTransactions {
    pub payments: u16,
    pub delegations: u16,
    pub zkapps: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VrfEvaluatorStats {
    pub total_slots: u32,
    pub evaluated_slots: u32,
}

impl Default for VrfEvaluatorStats {
    fn default() -> Self {
        Self {
            total_slots: 7140,
            evaluated_slots: 0,
        }
    }
}

impl BlockProducerStats {
    fn latest_attempt_block_hash_matches(&self, hash: &BlockHash) -> bool {
        self.attempts
            .back()
            .and_then(|v| v.block.as_ref())
            .is_some_and(|b| &b.hash == hash)
    }

    pub fn collect_attempts(&self) -> Vec<BlockProductionAttempt> {
        self.attempts.iter().cloned().collect()
    }

    pub fn new_best_chain(&mut self, time: redux::Timestamp, chain: &[AppliedBlock]) {
        let (best_tip, chain) = chain.split_last().unwrap();
        let root_block = chain.first().unwrap_or(best_tip);

        self.committed(time, best_tip.hash());

        self.attempts
            .iter_mut()
            .rev()
            .take_while(|v| v.won_slot.global_slot >= root_block.global_slot())
            .filter(|attempt| {
                matches!(
                    attempt.status,
                    BlockProductionStatus::Committed
                        | BlockProductionStatus::Canonical { .. }
                        | BlockProductionStatus::Orphaned { .. }
                )
            })
            .for_each(|attempt| {
                let Some(block) = attempt.block.as_ref() else {
                    return;
                };
                let Some(i) = block.height.checked_sub(root_block.height()) else {
                    return;
                };

                match chain.get(i as usize) {
                    Some(b) if b.hash() == &block.hash => {
                        attempt.status = BlockProductionStatus::Canonical {
                            last_observed_confirmations: best_tip
                                .height()
                                .saturating_sub(block.height),
                        };
                    }
                    Some(b) => {
                        attempt.status = BlockProductionStatus::Orphaned {
                            orphaned_by: b.hash().clone(),
                        };
                    }
                    None => {}
                }
            });
    }

    fn update<F>(&mut self, kind: &'static str, with: F)
    where
        F: FnOnce(&mut BlockProductionAttempt) -> bool,
    {
        match self.attempts.pop_back() {
            None => {
                openmina_core::log::error!(openmina_core::log::system_time();
                    kind = "BlockProducerStatsAttemptsEmpty",
                    summary = "attempts are empty when they aren't expected to be",
                    update_kind = kind);
            }
            Some(mut attempt) => {
                let was_correct_state = with(&mut attempt);

                if !was_correct_state {
                    openmina_core::log::error!(openmina_core::log::system_time();
                        kind = "BlockProducerStatsAttemptUnexpectedState",
                        summary = format!("update kind `{kind}` is not applicable to state: {attempt:?}"));
                }
                self.attempts.push_back(attempt);
            }
        }
    }

    pub fn scheduled(&mut self, time: redux::Timestamp, won_slot: &BlockProducerWonSlot) {
        if self.attempts.len() >= MAX_HISTORY {
            self.attempts.pop_front();
        }
        self.attempts.push_back(BlockProductionAttempt {
            won_slot: won_slot.into(),
            block: None,
            times: BlockProductionTimes {
                scheduled: time,
                staged_ledger_diff_create_start: None,
                staged_ledger_diff_create_end: None,
                produced: None,
                proof_create_start: None,
                proof_create_end: None,
                proof_create_error: None,
                block_apply_start: None,
                block_apply_end: None,
                committed: None,
                discarded: None,
            },
            status: BlockProductionStatus::Scheduled,
        });
    }

    pub fn staged_ledger_diff_create_start(&mut self, time: redux::Timestamp) {
        self.update(
            "staged_ledger_diff_create_start",
            move |attempt| match attempt.status {
                BlockProductionStatus::Scheduled => {
                    attempt.status = BlockProductionStatus::StagedLedgerDiffCreatePending;
                    attempt.times.staged_ledger_diff_create_start = Some(time);
                    true
                }
                _ => false,
            },
        );
    }

    pub fn staged_ledger_diff_create_end(&mut self, time: redux::Timestamp) {
        self.update(
            "staged_ledger_diff_create_end",
            move |attempt| match attempt.status {
                BlockProductionStatus::StagedLedgerDiffCreatePending => {
                    attempt.status = BlockProductionStatus::StagedLedgerDiffCreateSuccess;
                    attempt.times.staged_ledger_diff_create_end = Some(time);
                    true
                }
                _ => false,
            },
        );
    }

    pub fn produced(
        &mut self,
        time: redux::Timestamp,
        block_hash: &BlockHash,
        block: &BlockWithoutProof,
    ) {
        self.update("produced", move |attempt| match attempt.status {
            BlockProductionStatus::StagedLedgerDiffCreateSuccess => {
                attempt.status = BlockProductionStatus::Produced;
                attempt.times.produced = Some(time);
                attempt.block = Some((block_hash, block).into());
                true
            }
            _ => false,
        });
    }

    pub fn proof_create_start(&mut self, time: redux::Timestamp) {
        self.update("proof_create_start", move |attempt| match attempt.status {
            BlockProductionStatus::Produced => {
                attempt.status = BlockProductionStatus::ProofCreatePending;
                attempt.times.proof_create_start = Some(time);
                true
            }
            _ => false,
        });
    }

    pub fn proof_create_end(&mut self, time: redux::Timestamp) {
        self.update("proof_create_end", move |attempt| match attempt.status {
            BlockProductionStatus::ProofCreatePending => {
                attempt.status = BlockProductionStatus::ProofCreateSuccess;
                attempt.times.proof_create_end = Some(time);
                true
            }
            _ => false,
        });
    }

    pub fn proof_create_error(&mut self, time: redux::Timestamp, error: String) {
        self.update("proof_create_error", move |attempt| match attempt.status {
            BlockProductionStatus::ProofCreatePending => {
                attempt.status = BlockProductionStatus::ProofCreateError { error };
                attempt.times.proof_create_error = Some(time);
                true
            }
            _ => false,
        });
    }

    pub fn block_apply_start(&mut self, time: redux::Timestamp, hash: &BlockHash) {
        if !self.is_our_just_produced_block(hash) {
            return;
        }

        self.update("block_apply_start", move |attempt| match attempt.status {
            BlockProductionStatus::ProofCreateSuccess => {
                attempt.status = BlockProductionStatus::BlockApplyPending;
                attempt.times.block_apply_start = Some(time);
                true
            }
            _ => false,
        });
    }

    pub fn block_apply_end(&mut self, time: redux::Timestamp, hash: &BlockHash) {
        if !self.latest_attempt_block_hash_matches(hash) {
            return;
        }

        self.update("block_apply_end", move |attempt| match attempt.status {
            BlockProductionStatus::BlockApplyPending => {
                attempt.status = BlockProductionStatus::BlockApplySuccess;
                attempt.times.block_apply_end = Some(time);
                true
            }
            _ => false,
        });
    }

    pub fn committed(&mut self, time: redux::Timestamp, hash: &BlockHash) {
        if !self.latest_attempt_block_hash_matches(hash) {
            return;
        }

        self.update("committed", move |attempt| match attempt.status {
            BlockProductionStatus::BlockApplySuccess => {
                attempt.status = BlockProductionStatus::Committed;
                attempt.times.committed = Some(time);
                true
            }
            _ => false,
        });
    }

    pub fn discarded(&mut self, time: redux::Timestamp, reason: BlockProducerWonSlotDiscardReason) {
        self.update("discarded", move |attempt| {
            attempt.status = BlockProductionStatus::Discarded {
                discard_reason: reason,
            };
            attempt.times.discarded = Some(time);
            true
        });
    }

    /// Returns `true` if this is a block we just produced
    pub fn is_our_just_produced_block(&self, hash: &BlockHash) -> bool {
        // For the block to be ours:
        // - we must have an attempt to produce a block
        // - we must have just produced the proof for that block
        // - the hash must match
        if let Some(attempt) = self.attempts.back() {
            match (&attempt.status, attempt.block.as_ref()) {
                (BlockProductionStatus::ProofCreateSuccess, Some(block)) => &block.hash == hash,
                _ => false,
            }
        } else {
            false
        }
    }

    /// In case a new run, when the current epoch has less than `slots_per_epoch` slots to evaluate.
    pub fn new_epoch_evaluation(&mut self, epoch: u32, remaining_slots: u32) {
        self.vrf_evaluator.insert(
            epoch,
            VrfEvaluatorStats {
                total_slots: remaining_slots,
                evaluated_slots: 0,
            },
        );
    }

    pub fn increment_slot_evaluated(&mut self, epoch: u32) {
        self.vrf_evaluator
            .entry(epoch)
            .and_modify(|v| v.evaluated_slots = v.evaluated_slots.checked_add(1).expect("overflow"))
            .or_insert_with(|| VrfEvaluatorStats {
                evaluated_slots: 1,
                ..Default::default()
            });
    }
}

impl From<&BlockProducerWonSlot> for BlockProductionAttemptWonSlot {
    fn from(won_slot: &BlockProducerWonSlot) -> Self {
        Self {
            slot_time: won_slot.slot_time,
            global_slot: won_slot.global_slot(),
            epoch: won_slot.epoch(),
            delegator: won_slot.delegator.clone(),
            value_with_threshold: won_slot.value_with_threshold,
        }
    }
}

impl From<(&BlockHash, &BlockWithoutProof)> for ProducedBlock {
    fn from((block_hash, block): (&BlockHash, &BlockWithoutProof)) -> Self {
        Self {
            hash: block_hash.clone(),
            height: block
                .protocol_state
                .body
                .consensus_state
                .blockchain_length
                .as_u32(),
            transactions: block.into(),
            completed_works_count: block.body.completed_works_count(),
            coinbase: if block.body.has_coinbase() {
                openmina_core::constants::constraint_constants().coinbase_amount
            } else {
                0
            },
            fees: block.body.fees_sum(),
            snark_fees: block.body.snark_fees_sum(),
        }
    }
}

impl From<&BlockWithoutProof> for ProducedBlockTransactions {
    fn from(block: &BlockWithoutProof) -> Self {
        block
            .body
            .commands_iter()
            .fold(Self::default(), |mut res, cmd| {
                match &cmd.data {
                    v2::MinaBaseUserCommandStableV2::SignedCommand(v) => match &v.payload.body {
                        v2::MinaBaseSignedCommandPayloadBodyStableV2::Payment(_) => {
                            res.payments = res.payments.checked_add(1).expect("overflow")
                        }
                        v2::MinaBaseSignedCommandPayloadBodyStableV2::StakeDelegation(_) => {
                            res.delegations = res.delegations.checked_add(1).expect("overflow")
                        }
                    },
                    v2::MinaBaseUserCommandStableV2::ZkappCommand(_) => {
                        res.zkapps = res.zkapps.checked_add(1).expect("overflow")
                    }
                }
                res
            })
    }
}
