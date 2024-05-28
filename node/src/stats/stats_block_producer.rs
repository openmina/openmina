use std::collections::VecDeque;

use ledger::AccountIndex;
use mina_p2p_messages::v2;
use serde::{Deserialize, Serialize};

use crate::{
    block_producer::{BlockProducerWonSlot, BlockProducerWonSlotDiscardReason, BlockWithoutProof},
    core::block::BlockHash,
};

const MAX_HISTORY: usize = 2048;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct BlockProducerStats {
    pub(super) attempts: VecDeque<BlockProductionAttempt>,
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
    pub block_apply_start: Option<redux::Timestamp>,
    pub block_apply_end: Option<redux::Timestamp>,
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
    BlockApplyPending,
    BlockApplySuccess,
    Discarded {
        discard_reason: BlockProducerWonSlotDiscardReason,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProducedBlock {
    pub hash: BlockHash,
    pub height: u32,
    pub transactions: ProducedBlockTransactions,
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

impl BlockProducerStats {
    pub fn collect_attempts(&self) -> Vec<BlockProductionAttempt> {
        self.attempts.iter().cloned().collect()
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
                block_apply_start: None,
                block_apply_end: None,
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
        block_hash: &v2::StateHash,
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

    pub fn block_apply_start(&mut self, time: redux::Timestamp, hash: &v2::StateHash) {
        let is_our_block = self
            .attempts
            .back()
            .and_then(|v| v.block.as_ref())
            .map_or(false, |b| &b.hash == hash);
        if !is_our_block {
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

    pub fn block_apply_end(&mut self, time: redux::Timestamp, hash: &v2::StateHash) {
        let is_our_block = self
            .attempts
            .back()
            .and_then(|v| v.block.as_ref())
            .map_or(false, |b| &b.hash == hash);
        if !is_our_block {
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

    pub fn discarded(&mut self, time: redux::Timestamp, reason: BlockProducerWonSlotDiscardReason) {
        self.update("discarded", move |attempt| {
            attempt.status = BlockProductionStatus::Discarded {
                discard_reason: reason,
            };
            attempt.times.discarded = Some(time);
            true
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
            value_with_threshold: won_slot.value_with_threshold.clone(),
        }
    }
}

impl From<(&v2::StateHash, &BlockWithoutProof)> for ProducedBlock {
    fn from((block_hash, block): (&v2::StateHash, &BlockWithoutProof)) -> Self {
        Self {
            hash: block_hash.clone(),
            height: block
                .protocol_state
                .body
                .consensus_state
                .blockchain_length
                .as_u32(),
            transactions: block.into(),
            coinbase: block.body.coinbase_sum(),
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
                            res.payments += 1
                        }
                        v2::MinaBaseSignedCommandPayloadBodyStableV2::StakeDelegation(_) => {
                            res.delegations += 1
                        }
                    },
                    v2::MinaBaseUserCommandStableV2::ZkappCommand(_) => res.zkapps += 1,
                }
                res
            })
    }
}
