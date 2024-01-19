use std::collections::BTreeMap;
use std::sync::Arc;

use mina_p2p_messages::v2::LedgerHash;
use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};

use crate::block_producer::BlockProducerWonSlot;

use super::{DelegatorTable, VrfEvaluatorInput, VrfWonSlotWithHash};

// TODO(adonagy): consodilate types, make more clear
// pub type AccountAddressAndBalance = (String, u64);

// TODO(adonagy): cleanup state for production version.
//
// We should try to make impossible states irrepresentable.
// Some examples:
// - status can be: `DataPending`, yet `epoch_data.delegator_table` can
//   be set.
// - `current_epoch_data` and/or `next_epoch_data` might not be set, but
//   we might be in a state which expects those to be set.
// etc...
//
// Sure, enabling conditions, reducers and effects might help mitigate
// that, but type system won't help us and it will be very easy to make
// mistakes.
// Also as a reader, state has no contraints so I can't deduce much from it.
// When making changes, I'll have to review all the parts to make sure
// hidden link between `status` and the rest of the data is being enforced
// somewhere.
//
// For the very least a most of this state should be moved inside the `status`.
// That way there is a direct link between `status` and underlying data.
// Only thing I'd leave outside the `status` from below, is `won_slots`,
// since we need to accumulate that.
//
// Even better than that would be to have 2 enums. Main one for preparing
// data necessary for evaluation (building delegatee table), and once
// we are done there, have a nested state for slot evaluation.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerVrfEvaluatorState {
    pub status: BlockProducerVrfEvaluatorStatus,
    pub won_slots: BTreeMap<u32, VrfWonSlotWithHash>,
    pub latest_evaluated_slot: u32,
    pub genesis_timestamp: redux::Timestamp,
    last_evaluated_epoch: Option<u32>,
    last_block_heights_in_epoch: BTreeMap<u32, u32>,
}

impl BlockProducerVrfEvaluatorState {
    pub fn new(now: redux::Timestamp) -> Self {
        Self {
            status: BlockProducerVrfEvaluatorStatus::Idle { time: now },
            won_slots: Default::default(),
            latest_evaluated_slot: Default::default(),
            genesis_timestamp: redux::Timestamp::ZERO,
            last_evaluated_epoch: Default::default(),
            last_block_heights_in_epoch: Default::default(),
        }
    }

    pub fn next_won_slot(
        &self,
        cur_global_slot: u32,
        best_tip: &ArcBlockWithHash,
    ) -> Option<BlockProducerWonSlot> {
        self.won_slots
            .range(cur_global_slot..)
            .map(|(_, won_slot)| {
                BlockProducerWonSlot::from_vrf_won_slot(won_slot, best_tip.genesis_timestamp())
            })
            .find(|won_slot| won_slot > best_tip)
    }

    pub fn last_evaluated_epoch(&self) -> Option<u32> {
        self.last_evaluated_epoch
    }

    pub fn add_last_height(&mut self, epoch: u32, height: u32) {
        self.last_block_heights_in_epoch.insert(epoch, height);
    }

    pub fn last_height(&self, epoch: u32) -> Option<u32> {
        self.last_block_heights_in_epoch.get(&epoch).copied()
    }

    pub fn last_evaluated_global_slot(&self) -> u32 {
        self.latest_evaluated_slot
    }

    pub fn set_last_evaluated_global_slot(&mut self, global_slot: &u32) {
        if self.status.is_evaluating() {
            self.latest_evaluated_slot = *global_slot
        }
    }

    pub fn initialize_evaluator(&mut self, epoch: u32, last_height: u32) {
        if !self.status.is_initialized() {
            self.last_block_heights_in_epoch.insert(epoch, last_height);
        }
    }

    pub fn set_last_evaluated_epoch(&mut self) -> bool {
        match self.status {
            BlockProducerVrfEvaluatorStatus::EpochEvaluationPending { epoch_number, .. } => {
                self.last_evaluated_epoch = Some(epoch_number);
                true
            }
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EpochData {
    pub seed: String,
    pub ledger: LedgerHash,
    pub delegator_table: Arc<DelegatorTable>,
    pub total_currency: u64,
}

impl EpochData {
    pub fn new(seed: String, ledger: LedgerHash, total_currency: u64) -> Self {
        Self {
            seed,
            ledger,
            total_currency,
            delegator_table: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PendingEvaluation {
    pub epoch_number: u32,
    pub epoch_data: EpochData,
    pub latest_evaluated_slot: u32,
    pub epoch_context: EpochContext,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BlockProducerVrfEvaluatorStatus {
    Idle {
        time: redux::Timestamp,
    },
    EvaluatorInitialisationPending {
        time: redux::Timestamp,
    },
    EvaluatorInitialized {
        time: redux::Timestamp,
    },
    CanEvaluateVrf {
        time: redux::Timestamp,
        current_epoch_number: u32,
        is_current_epoch_evaluated: bool,
        is_next_epoch_evaluated: bool,
    },
    EpochEvaluationInit {
        time: redux::Timestamp,
        epoch_context: EpochContext,
    },
    EpochDelegatorTablePending {
        time: redux::Timestamp,
        epoch_number: u32,
        staking_epoch_ledger_hash: LedgerHash,
        epoch_context: EpochContext,
    },
    EpochDelegatorTableSuccess {
        time: redux::Timestamp,
        epoch_number: u32,
        staking_epoch_ledger_hash: LedgerHash,
        epoch_context: EpochContext,
    },
    EpochEvaluationPending {
        time: redux::Timestamp,
        epoch_number: u32,
        epoch_data: EpochData,
        latest_evaluated_slot: u32,
        epoch_context: EpochContext,
    },
    EpochEvaluationSuccess {
        time: redux::Timestamp,
        epoch_number: u32,
        epoch_context: EpochContext,
    },
    WaitingForEvaluation {
        time: redux::Timestamp,
        current_epoch_number: u32,
        current_best_tip_height: u32,
        current_best_tip_slot: u32,
        current_best_tip_global_slot: u32,
        last_epoch_block_height: Option<u32>,
        transition_frontier_size: u32,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EpochContext {
    Current,
    Next,
    Waiting,
}

impl std::fmt::Display for EpochContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Current => write!(f, "Current"),
            Self::Next => write!(f, "Next"),
            Self::Waiting => write!(f, "Waiting"),
        }
    }
}

impl BlockProducerVrfEvaluatorStatus {
    pub fn epoch_context(&self) -> EpochContext {
        match self {
            Self::Idle { .. } => EpochContext::Current,
            Self::EpochEvaluationInit { epoch_context, .. }
            | Self::EpochDelegatorTablePending { epoch_context, .. }
            | Self::EpochDelegatorTableSuccess { epoch_context, .. }
            | Self::EpochEvaluationPending { epoch_context, .. } => epoch_context.clone(),
            Self::CanEvaluateVrf { .. } | Self::WaitingForEvaluation { .. } => {
                EpochContext::Waiting
            }
            // TODO(adonagy): remove once the old states are cleaned up
            _ => panic!("this: {:?}", self),
        }
    }

    pub fn is_initialized(&self) -> bool {
        !matches!(self, Self::Idle { .. })
    }

    pub fn is_evaluating_current_epoch(&self) -> bool {
        matches!(
            self,
            Self::EpochEvaluationPending {
                epoch_context: EpochContext::Current,
                ..
            }
        )
    }

    pub fn is_evaluating_next_epoch(&self) -> bool {
        matches!(
            self,
            Self::EpochEvaluationPending {
                epoch_context: EpochContext::Next,
                ..
            }
        )
    }

    pub fn is_evaluating(&self) -> bool {
        self.is_evaluating_current_epoch() | self.is_evaluating_next_epoch()
    }

    pub fn is_evaluator_ready(&self, last_evaluated_epoch: Option<u32>) -> bool {
        match self {
            Self::EvaluatorInitialized { .. }
            | Self::EpochEvaluationSuccess {
                epoch_context: EpochContext::Current,
                ..
            } => true,
            Self::WaitingForEvaluation {
                transition_frontier_size,
                current_best_tip_height,
                last_epoch_block_height: Some(last_epoch_block_height),
                ..
            } => {
                // predicate 1: only start the next epoch evaluation if the last evaluated epoch is the current epoch
                // predicate 2: and there were at least k (k == 290 == transition_frontier_size) blocks created in the current epoch
                // so the next_epoch_ledger (staking ledger for the next epoch) is "stabilized" (was in the root of the transition frontier)

                // TODO(adonagy): edge case when the current epoch won't have enough slots filled, this is an extreme and should it happen the network has
                // some serious issues. Handle it as well?
                !self.is_next_epoch_evaluated(last_evaluated_epoch)
                    && (current_best_tip_height.saturating_sub(*last_epoch_block_height))
                        > *transition_frontier_size
            }
            _ => false,
        }
    }

    pub fn epoch_to_evaluate(&self) -> EpochContext {
        match self {
            Self::CanEvaluateVrf {
                is_current_epoch_evaluated,
                ..
            } => {
                if !is_current_epoch_evaluated {
                    EpochContext::Current
                } else {
                    EpochContext::Next
                }
            }
            _ => EpochContext::Waiting,
        }
    }

    pub fn is_current_epoch_evaluated(&self, last_evaluated_epoch: Option<u32>) -> bool {
        match self {
            BlockProducerVrfEvaluatorStatus::CanEvaluateVrf {
                current_epoch_number,
                ..
            } => last_evaluated_epoch >= Some(*current_epoch_number),
            BlockProducerVrfEvaluatorStatus::EpochEvaluationSuccess {
                epoch_context: EpochContext::Current,
                ..
            }
            | BlockProducerVrfEvaluatorStatus::EpochEvaluationInit {
                epoch_context: EpochContext::Next,
                ..
            }
            | BlockProducerVrfEvaluatorStatus::EpochDelegatorTablePending {
                epoch_context: EpochContext::Next,
                ..
            }
            | BlockProducerVrfEvaluatorStatus::EpochDelegatorTableSuccess {
                epoch_context: EpochContext::Next,
                ..
            }
            | BlockProducerVrfEvaluatorStatus::EpochEvaluationPending {
                epoch_context: EpochContext::Next,
                ..
            }
            | BlockProducerVrfEvaluatorStatus::EpochEvaluationSuccess {
                epoch_context: EpochContext::Next,
                ..
            }
            | BlockProducerVrfEvaluatorStatus::WaitingForEvaluation { .. } => true,
            _ => false,
        }
    }

    pub fn is_next_epoch_evaluated(&self, last_evaluated_epoch: Option<u32>) -> bool {
        match self {
            BlockProducerVrfEvaluatorStatus::CanEvaluateVrf {
                current_epoch_number,
                ..
            }
            | BlockProducerVrfEvaluatorStatus::WaitingForEvaluation {
                current_epoch_number,
                ..
            } => last_evaluated_epoch >= Some(current_epoch_number + 1),
            BlockProducerVrfEvaluatorStatus::EpochEvaluationSuccess {
                epoch_context: EpochContext::Next,
                ..
            } => true,
            _ => false,
        }
    }

    pub fn can_construct_delegator_table(&self) -> bool {
        matches!(self, Self::EpochEvaluationInit { .. })
    }

    pub fn is_delegator_table_requested(&self) -> bool {
        matches!(self, Self::EpochDelegatorTablePending { .. })
    }

    pub fn can_start_current_epoch_evaluation(&self) -> bool {
        matches!(
            self,
            Self::EpochDelegatorTableSuccess {
                epoch_context: EpochContext::Current,
                ..
            }
        )
    }

    pub fn can_start_next_epoch_evaluation(&self) -> bool {
        matches!(
            self,
            Self::EpochDelegatorTableSuccess {
                epoch_context: EpochContext::Next,
                ..
            }
        )
    }

    pub fn is_current_epoch_initiated(&self) -> bool {
        matches!(
            self,
            Self::EpochEvaluationInit {
                epoch_context: EpochContext::Current,
                ..
            }
        )
    }

    pub fn is_next_epoch_initiated(&self) -> bool {
        matches!(
            self,
            Self::EpochEvaluationInit {
                epoch_context: EpochContext::Next,
                ..
            }
        )
    }

    pub fn construct_vrf_input(&self) -> Option<VrfEvaluatorInput> {
        if let Some(pending_evaluation) = self.current_evaluation() {
            Some(VrfEvaluatorInput::new(
                pending_evaluation.epoch_data.seed,
                pending_evaluation.epoch_data.delegator_table,
                pending_evaluation.latest_evaluated_slot + 1,
                pending_evaluation.epoch_data.total_currency,
                pending_evaluation.epoch_data.ledger,
            ))
        } else {
            None
        }
    }

    pub fn current_evaluation(&self) -> Option<PendingEvaluation> {
        match self {
            Self::EpochEvaluationPending {
                epoch_number,
                epoch_data,
                latest_evaluated_slot,
                epoch_context,
                ..
            } => Some(PendingEvaluation {
                epoch_data: epoch_data.clone(),
                epoch_number: *epoch_number,
                latest_evaluated_slot: *latest_evaluated_slot,
                epoch_context: epoch_context.clone(),
            }),
            _ => None,
        }
    }
}
