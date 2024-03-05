use std::collections::BTreeMap;
use std::sync::Arc;

use mina_p2p_messages::v2::LedgerHash;
use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};

use crate::block_producer::BlockProducerWonSlot;

use super::{DelegatorTable, VrfEvaluatorInput, VrfWonSlotWithHash};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerVrfEvaluatorState {
    pub status: BlockProducerVrfEvaluatorStatus,
    pub won_slots: BTreeMap<u32, VrfWonSlotWithHash>,
    pub latest_evaluated_slot: u32,
    pub genesis_timestamp: redux::Timestamp,
    last_evaluated_epoch: Option<u32>,
    last_block_heights_in_epoch: BTreeMap<u32, u32>,
    pending_evaluation: Option<PendingEvaluation>,
    // epoch_context: EpochContext,
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
            pending_evaluation: Default::default(),
            // epoch_context: EpochContext::Current,
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

    // pub fn epoch_context(&self) -> &EpochContext {
    //     &self.epoch_context
    // }

    // pub fn set_epoch_context(&mut self) {

    // }

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
            if let Some(ref mut pending_evaluation) = self.pending_evaluation {
                pending_evaluation.latest_evaluated_slot = *global_slot;
            }
            self.latest_evaluated_slot = *global_slot
        }
    }

    pub fn initialize_evaluator(&mut self, epoch: u32, last_height: u32) {
        if !self.status.is_initialized() {
            self.last_block_heights_in_epoch.insert(epoch, last_height);
        }
    }

    pub fn set_last_evaluated_epoch(&mut self) {
        if let BlockProducerVrfEvaluatorStatus::EpochEvaluationSuccess {
            epoch_number,
            epoch_context,
            ..
        } = &self.status
        {
            match epoch_context {
                EpochContext::Current => self.last_evaluated_epoch = Some(*epoch_number),
                EpochContext::Next => self.last_evaluated_epoch = Some(epoch_number + 1),
                EpochContext::Waiting => {}
            }
        }
    }

    pub fn set_pending_evaluation(&mut self, pending_evaluation: PendingEvaluation) {
        self.pending_evaluation = Some(pending_evaluation)
    }

    pub fn unset_pending_evaluation(&mut self) {
        self.pending_evaluation = None
    }

    pub fn current_evaluation(&self) -> Option<PendingEvaluation> {
        self.pending_evaluation.clone()
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
    /// Initial state
    Idle { time: redux::Timestamp },
    /// Waiting for initialization
    InitialisationPending { time: redux::Timestamp },
    /// Initialization was a success
    InitialisationComplete { time: redux::Timestamp },
    /// Evaluator is ready and able to evaluate an epoch
    ReadyToEvaluate {
        time: redux::Timestamp,
        epoch_context: EpochContext,
        current_epoch_number: u32,
        is_current_epoch_evaluated: bool,
        is_next_epoch_evaluated: bool,
    },
    /// Waiting for delegator table building
    EpochDelegatorTablePending {
        time: redux::Timestamp,
        epoch_number: u32,
        staking_epoch_ledger_hash: LedgerHash,
        epoch_context: EpochContext,
    },
    /// Delegator table built successfully
    EpochDelegatorTableSuccess {
        time: redux::Timestamp,
        epoch_number: u32,
        staking_epoch_ledger_hash: LedgerHash,
        epoch_context: EpochContext,
    },
    /// Epoch evaluation in progress
    EpochEvaluationPending {
        time: redux::Timestamp,
        epoch_number: u32,
        epoch_data: EpochData,
        latest_evaluated_global_slot: u32,
        epoch_context: EpochContext,
    },
    /// A slot was sent for evaluation to the vrf evaluator service
    SlotEvaluationPending {
        time: redux::Timestamp,
        // epoch_number: u32,
        // epoch_data: EpochData,
        // latest_evaluated_global_slot: u32,
        global_slot: u32,
        // staking_epoch_ledger: LedgerHash,
        epoch_context: EpochContext,
    },
    /// The service returned the evaluation succesfully
    EpochEvaluationSuccess {
        time: redux::Timestamp,
        epoch_number: u32,
        epoch_context: EpochContext,
    },
    /// Waiting for the next possible epoch evaluation
    WaitingForNextEvaluation {
        time: redux::Timestamp,
        current_epoch_number: u32,
        current_best_tip_height: u32,
        current_best_tip_slot: u32,
        current_best_tip_global_slot: u32,
        last_epoch_block_height: Option<u32>,
        transition_frontier_size: u32,
    },
    /// Checking whether the evaluator is abble to evaluate the epoch
    /// Note: The current epoch can be allways evaluated right away
    ReadinessCheck {
        time: redux::Timestamp,
        current_epoch_number: u32,
        is_current_epoch_evaluated: bool,
        is_next_epoch_evaluated: bool,
        transition_frontier_size: u32,
        current_best_tip_height: u32,
        last_evaluated_epoch: Option<u32>,
        last_epoch_block_height: Option<u32>,
    },
}

impl std::fmt::Display for BlockProducerVrfEvaluatorStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle { .. } => write!(f, "Idle"),
            Self::InitialisationPending { .. } => write!(f, "InitialisationPending"),
            Self::InitialisationComplete { .. } => write!(f, "InitialisationComplete"),
            Self::ReadyToEvaluate { .. } => write!(f, "ReadyToEvaluate"),
            Self::EpochDelegatorTablePending { .. } => write!(f, "EpochDelegatorTablePending"),
            Self::EpochDelegatorTableSuccess { .. } => write!(f, "EpochDelegatorTableSuccess"),
            Self::EpochEvaluationPending { .. } => write!(f, "EpochEvaluationPending"),
            Self::SlotEvaluationPending { .. } => write!(f, "SlotEvaluationPending"),
            Self::EpochEvaluationSuccess { .. } => write!(f, "EpochEvaluationSuccess"),
            Self::WaitingForNextEvaluation { .. } => write!(f, "WaitingForNextEvaluation"),
            Self::ReadinessCheck { .. } => write!(f, "ReadinessCheck"),
        }
    }
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
            Self::Idle { .. }
            | Self::InitialisationPending { .. }
            | Self::InitialisationComplete { .. } => EpochContext::Current,
            Self::EpochDelegatorTablePending { epoch_context, .. }
            | Self::EpochDelegatorTableSuccess { epoch_context, .. }
            | Self::EpochEvaluationPending { epoch_context, .. }
            | Self::EpochEvaluationSuccess { epoch_context, .. }
            | Self::SlotEvaluationPending { epoch_context, .. }
            | Self::ReadyToEvaluate { epoch_context, .. } => epoch_context.clone(),
            Self::ReadinessCheck { .. } | Self::WaitingForNextEvaluation { .. } => {
                EpochContext::Waiting
            }
        }
    }

    pub fn is_initialized(&self) -> bool {
        !matches!(self, Self::Idle { .. })
    }

    pub fn is_evaluating_current_epoch(&self) -> bool {
        match self {
            Self::EpochEvaluationPending { epoch_context, .. }
            | Self::SlotEvaluationPending { epoch_context, .. } => {
                matches!(epoch_context, EpochContext::Current)
            }
            _ => false,
        }
    }

    pub fn is_evaluating_next_epoch(&self) -> bool {
        match self {
            Self::EpochEvaluationPending { epoch_context, .. }
            | Self::SlotEvaluationPending { epoch_context, .. } => {
                matches!(epoch_context, EpochContext::Next)
            }
            _ => false,
        }
    }

    pub fn is_evaluating(&self) -> bool {
        self.is_evaluating_current_epoch() | self.is_evaluating_next_epoch()
    }

    pub fn can_evaluate_epoch(&self, last_evaluated_epoch: Option<u32>) -> bool {
        println!("[can_evaluate_epoch] STATE: {}", self);
        match self {
            // Self::InitialisationComplete { .. } => true,
            Self::ReadinessCheck {
                transition_frontier_size,
                current_best_tip_height,
                last_epoch_block_height,
                current_epoch_number,
                ..
            } => {
                // so the next_epoch_ledger (staking ledger for the next epoch) is "stabilized" (was in the root of the transition frontier)

                // TODO(adonagy): edge case when the current epoch won't have enough slots filled, this is an extreme and should it happen the network has
                // some serious issues. Handle it as well?
                println!(
                    "Is current epoch evaluated?: {}",
                    self.is_current_epoch_evaluated(last_evaluated_epoch)
                );
                // predicate 1: if the current epoch is not evaluated, start the evaluation
                if !self.is_current_epoch_evaluated(last_evaluated_epoch)
                    || (!self.is_next_epoch_evaluated(last_evaluated_epoch)
                        && *current_epoch_number == 0)
                {
                    true
                // predicate 2: or only start the next epoch evaluation if the current epoch is evaluated
                // predicate 3: and the next epoch is NOT evaluated
                // predicate 4: and there were at least k (k == 290 == transition_frontier_size) blocks created in the current epoch
                } else if let Some(last_epoch_block_height) = last_epoch_block_height {
                    self.is_current_epoch_evaluated(last_evaluated_epoch)
                        && !self.is_next_epoch_evaluated(last_evaluated_epoch)
                        && (current_best_tip_height.saturating_sub(*last_epoch_block_height))
                            > *transition_frontier_size
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn is_readiness_check(&self) -> bool {
        matches!(self, Self::ReadinessCheck { .. })
    }

    pub fn is_waiting_for_slot_evaluation(&self) -> bool {
        matches!(self, Self::SlotEvaluationPending { .. })
    }

    pub fn can_check_next_evaluation(&self) -> bool {
        matches!(
            self,
            Self::WaitingForNextEvaluation { .. }
                | Self::EpochEvaluationSuccess { .. }
                | Self::InitialisationComplete { .. }
        )
    }

    pub fn epoch_to_evaluate(&self) -> EpochContext {
        match self {
            Self::ReadinessCheck {
                is_current_epoch_evaluated,
                is_next_epoch_evaluated,
                last_evaluated_epoch,
                ..
            } => {
                if !is_current_epoch_evaluated {
                    EpochContext::Current
                } else if !is_next_epoch_evaluated {
                    if self.can_evaluate_epoch(*last_evaluated_epoch) {
                        EpochContext::Next
                    } else {
                        EpochContext::Waiting
                    }
                } else {
                    EpochContext::Waiting
                }
            }
            _ => EpochContext::Waiting,
        }
    }

    pub fn is_current_epoch_evaluated(&self, last_evaluated_epoch: Option<u32>) -> bool {
        match self {
            BlockProducerVrfEvaluatorStatus::ReadinessCheck {
                current_epoch_number,
                ..
            }
            | BlockProducerVrfEvaluatorStatus::ReadyToEvaluate {
                current_epoch_number,
                ..
            } => last_evaluated_epoch >= Some(*current_epoch_number),
            BlockProducerVrfEvaluatorStatus::EpochEvaluationSuccess {
                epoch_context: EpochContext::Current,
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
            | BlockProducerVrfEvaluatorStatus::WaitingForNextEvaluation { .. } => true,
            _ => false,
        }
    }

    pub fn is_next_epoch_evaluated(&self, last_evaluated_epoch: Option<u32>) -> bool {
        match self {
            BlockProducerVrfEvaluatorStatus::ReadinessCheck {
                current_epoch_number,
                ..
            }
            | BlockProducerVrfEvaluatorStatus::ReadyToEvaluate {
                current_epoch_number,
                ..
            }
            | BlockProducerVrfEvaluatorStatus::WaitingForNextEvaluation {
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
        matches!(self, Self::ReadyToEvaluate { .. })
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
}
