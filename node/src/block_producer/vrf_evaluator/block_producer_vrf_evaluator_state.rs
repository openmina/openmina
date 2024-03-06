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
    epoch_context: EpochContext,
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
            epoch_context: EpochContext::Current,
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

    pub fn epoch_context(&self) -> &EpochContext {
        &self.epoch_context
    }

    pub fn set_epoch_context(&mut self) {
        // guard the epoch context change and permit it only if in the ReadinessCheck state
        if let BlockProducerVrfEvaluatorStatus::ReadinessCheck {
            last_epoch_block_height,
            current_best_tip_height,
            transition_frontier_size,
            current_epoch_number,
            ..
        } = self.status
        {
            if !self.is_epoch_evaluated(current_epoch_number) {
                self.epoch_context = EpochContext::Current
            } else if !self.is_epoch_evaluated(current_epoch_number + 1) {
                if let Some(last_epoch_block_height) = last_epoch_block_height {
                    if (last_epoch_block_height + transition_frontier_size)
                        >= current_best_tip_height
                    {
                        self.epoch_context = EpochContext::Next
                    } else {
                        self.epoch_context = EpochContext::Waiting
                    }
                } else {
                    // if last_epoch_block_height is not set, we are still in genesis epoch, Next epoch evaluation is possible
                    self.epoch_context = EpochContext::Next
                }
            } else {
                self.epoch_context = EpochContext::Waiting
            }
        }
    }

    pub fn is_epoch_evaluated(&self, epoch_number: u32) -> bool {
        if let Some(last_evaluated_epoch) = self.last_evaluated_epoch {
            last_evaluated_epoch >= epoch_number
        } else {
            false
        }
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

    pub fn is_readiness_check(&self) -> bool {
        matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::ReadinessCheck { .. }
        )
    }

    pub fn is_waiting_for_slot_evaluation(&self) -> bool {
        matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::SlotEvaluationPending { .. }
        )
    }

    pub fn can_check_next_evaluation(&self) -> bool {
        matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::WaitingForNextEvaluation { .. }
                | BlockProducerVrfEvaluatorStatus::EpochEvaluationSuccess { .. }
                | BlockProducerVrfEvaluatorStatus::InitialisationComplete { .. }
        )
    }

    pub fn can_construct_delegator_table(&self) -> bool {
        matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::ReadyToEvaluate { .. }
        )
    }

    pub fn is_delegator_table_requested(&self) -> bool {
        matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::EpochDelegatorTablePending { .. }
        )
    }

    pub fn is_delegator_table_constructed(&self) -> bool {
        matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::EpochDelegatorTableSuccess { .. }
        )
    }

    pub fn can_start_epoch_evaluation(&self) -> bool {
        matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::EpochDelegatorTableSuccess { .. }
        )
    }

    pub fn is_evaluating(&self) -> bool {
        matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::SlotEvaluationPending { .. }
                | BlockProducerVrfEvaluatorStatus::EpochEvaluationPending { .. }
        )
    }

    pub fn is_slot_selection(&self) -> bool {
        matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::InitialSlotSelection { .. }
        )
    }

    pub fn set_last_evaluated_global_slot(&mut self, global_slot: &u32) {
        if self.is_evaluating() {
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
        if let BlockProducerVrfEvaluatorStatus::EpochEvaluationSuccess { epoch_number, .. } =
            &self.status
        {
            match self.epoch_context {
                EpochContext::Current => self.last_evaluated_epoch = Some(*epoch_number),
                EpochContext::Next => self.last_evaluated_epoch = Some(epoch_number + 1),
                EpochContext::Waiting => {}
            }
        }
    }

    pub fn initial_slot(&self) -> Option<u32> {
        if let BlockProducerVrfEvaluatorStatus::InitialSlotSelection { initial_slot, .. } =
            self.status
        {
            Some(initial_slot)
        } else {
            None
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
        current_epoch_number: u32,
        is_current_epoch_evaluated: bool,
        is_next_epoch_evaluated: bool,
    },
    /// Waiting for delegator table building
    EpochDelegatorTablePending {
        time: redux::Timestamp,
        epoch_number: u32,
        staking_epoch_ledger_hash: LedgerHash,
    },
    /// Delegator table built successfully
    EpochDelegatorTableSuccess {
        time: redux::Timestamp,
        epoch_number: u32,
        staking_epoch_ledger_hash: LedgerHash,
    },
    InitialSlotSelection {
        time: redux::Timestamp,
        epoch_number: u32,
        initial_slot: u32,
    },
    /// Epoch evaluation in progress
    EpochEvaluationPending {
        time: redux::Timestamp,
        epoch_number: u32,
        epoch_data: EpochData,
        latest_evaluated_global_slot: u32,
    },
    /// A slot was sent for evaluation to the vrf evaluator service
    SlotEvaluationPending {
        time: redux::Timestamp,
        global_slot: u32,
    },
    /// The service returned the evaluation succesfully
    EpochEvaluationSuccess {
        time: redux::Timestamp,
        epoch_number: u32,
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
            Self::InitialSlotSelection { .. } => write!(f, "StartingSlotSelection"),
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
    pub fn is_initialized(&self) -> bool {
        !matches!(self, Self::Idle { .. })
    }
}

#[cfg(test)]
mod test {
    use std::{collections::BTreeMap, sync::Mutex};

    use lazy_static::lazy_static;

    use crate::block_producer::vrf_evaluator::{
        BlockProducerVrfEvaluatorState, BlockProducerVrfEvaluatorStatus, EpochContext,
    };

    lazy_static! {
        static ref GENESIS_EPOCH_FIRST_SLOT: Mutex<BlockProducerVrfEvaluatorState> = {
            let state = BlockProducerVrfEvaluatorState {
                status: BlockProducerVrfEvaluatorStatus::ReadinessCheck {
                    time: redux::Timestamp::global_now(),
                    current_epoch_number: 0,
                    is_current_epoch_evaluated: false,
                    is_next_epoch_evaluated: false,
                    transition_frontier_size: 290,
                    current_best_tip_height: 1,
                    last_evaluated_epoch: None,
                    last_epoch_block_height: Some(1),
                },
                won_slots: BTreeMap::new(),
                latest_evaluated_slot: 0,
                genesis_timestamp: redux::Timestamp::global_now(),
                last_evaluated_epoch: None,
                last_block_heights_in_epoch: BTreeMap::new(),
                pending_evaluation: None,
                epoch_context: EpochContext::Current,
            };
            Mutex::new(state)
        };
        static ref GENESIS_EPOCH_CURRENT_EPOCH_EVALUATED: Mutex<BlockProducerVrfEvaluatorState> = {
            let state = BlockProducerVrfEvaluatorState {
                status: BlockProducerVrfEvaluatorStatus::ReadinessCheck {
                    time: redux::Timestamp::global_now(),
                    current_epoch_number: 0,
                    is_current_epoch_evaluated: true,
                    is_next_epoch_evaluated: false,
                    transition_frontier_size: 290,
                    current_best_tip_height: 900,
                    last_evaluated_epoch: Some(0),
                    last_epoch_block_height: None,
                },
                won_slots: BTreeMap::new(),
                latest_evaluated_slot: 7139,
                genesis_timestamp: redux::Timestamp::global_now(),
                last_evaluated_epoch: Some(0),
                last_block_heights_in_epoch: BTreeMap::new(),
                pending_evaluation: None,
                epoch_context: EpochContext::Current,
            };
            Mutex::new(state)
        };
        static ref GENESIS_EPOCH_NEXT_EPOCH_EVALUATED: Mutex<BlockProducerVrfEvaluatorState> = {
            let state = BlockProducerVrfEvaluatorState {
                status: BlockProducerVrfEvaluatorStatus::ReadinessCheck {
                    time: redux::Timestamp::global_now(),
                    current_epoch_number: 0,
                    is_current_epoch_evaluated: true,
                    is_next_epoch_evaluated: true,
                    transition_frontier_size: 290,
                    current_best_tip_height: 1500,
                    last_evaluated_epoch: Some(1),
                    last_epoch_block_height: None,
                },
                won_slots: BTreeMap::new(),
                latest_evaluated_slot: 14279,
                genesis_timestamp: redux::Timestamp::global_now(),
                last_evaluated_epoch: Some(1),
                last_block_heights_in_epoch: BTreeMap::new(),
                pending_evaluation: None,
                epoch_context: EpochContext::Current,
            };
            Mutex::new(state)
        };
        static ref SECOND_EPOCH_STARTUP: Mutex<BlockProducerVrfEvaluatorState> = {
            let state = BlockProducerVrfEvaluatorState {
                status: BlockProducerVrfEvaluatorStatus::ReadinessCheck {
                    time: redux::Timestamp::global_now(),
                    current_epoch_number: 2,
                    is_current_epoch_evaluated: false,
                    is_next_epoch_evaluated: false,
                    transition_frontier_size: 290,
                    current_best_tip_height: 15000,
                    last_evaluated_epoch: None,
                    last_epoch_block_height: Some(14500),
                },
                won_slots: BTreeMap::new(),
                latest_evaluated_slot: 0,
                genesis_timestamp: redux::Timestamp::global_now(),
                last_evaluated_epoch: None,
                last_block_heights_in_epoch: BTreeMap::new(),
                pending_evaluation: None,
                epoch_context: EpochContext::Current,
            };
            Mutex::new(state)
        };
        static ref SECOND_EPOCH_CURRENT_EPOCH_EVALUATED_WAIT_FOR_NEXT: Mutex<BlockProducerVrfEvaluatorState> = {
            let state = BlockProducerVrfEvaluatorState {
                status: BlockProducerVrfEvaluatorStatus::ReadinessCheck {
                    time: redux::Timestamp::global_now(),
                    current_epoch_number: 2,
                    is_current_epoch_evaluated: true,
                    is_next_epoch_evaluated: false,
                    transition_frontier_size: 290,
                    current_best_tip_height: 15000,
                    last_evaluated_epoch: Some(2),
                    last_epoch_block_height: Some(14900),
                },
                won_slots: BTreeMap::new(),
                latest_evaluated_slot: 21419,
                genesis_timestamp: redux::Timestamp::global_now(),
                last_evaluated_epoch: Some(2),
                last_block_heights_in_epoch: BTreeMap::new(),
                pending_evaluation: None,
                epoch_context: EpochContext::Current,
            };
            Mutex::new(state)
        };
    }

    fn test_set_epoch_context(state: &mut BlockProducerVrfEvaluatorState, _expected: EpochContext) {
        state.set_epoch_context();
        assert!(matches!(state.epoch_context(), _expected));
    }

    #[test]
    fn correctly_set_epoch_context_on_startup() {
        let mut vrf_evaluator_state = GENESIS_EPOCH_FIRST_SLOT.lock().unwrap();
        test_set_epoch_context(&mut vrf_evaluator_state, EpochContext::Current)
    }

    #[test]
    fn correctly_switch_to_next_epoch() {
        let mut vrf_evaluator_state = GENESIS_EPOCH_CURRENT_EPOCH_EVALUATED.lock().unwrap();
        test_set_epoch_context(&mut vrf_evaluator_state, EpochContext::Next)
    }

    #[test]
    fn correctly_switch_to_waiting() {
        let mut vrf_evaluator_state = GENESIS_EPOCH_NEXT_EPOCH_EVALUATED.lock().unwrap();
        test_set_epoch_context(&mut vrf_evaluator_state, EpochContext::Waiting)
    }

    #[test]
    fn generic_epoch_set_epoch_context_on_startup() {
        let mut vrf_evaluator_state = SECOND_EPOCH_STARTUP.lock().unwrap();
        test_set_epoch_context(&mut vrf_evaluator_state, EpochContext::Current)
    }

    #[test]
    fn generic_epoch_correctly_switch_to_next_epoch() {
        // Staking ledger not yet materialized (wait k=290 blocks)
        let mut vrf_evaluator_state = SECOND_EPOCH_CURRENT_EPOCH_EVALUATED_WAIT_FOR_NEXT
            .lock()
            .unwrap();
        test_set_epoch_context(&mut vrf_evaluator_state, EpochContext::Waiting);

        vrf_evaluator_state.status = BlockProducerVrfEvaluatorStatus::ReadinessCheck {
            time: redux::Timestamp::global_now(),
            current_epoch_number: 2,
            is_current_epoch_evaluated: true,
            is_next_epoch_evaluated: false,
            transition_frontier_size: 290,
            // one block until can evaluate next
            current_best_tip_height: 15189,
            last_evaluated_epoch: Some(2),
            last_epoch_block_height: Some(14900),
        };

        test_set_epoch_context(&mut vrf_evaluator_state, EpochContext::Waiting);

        vrf_evaluator_state.status = BlockProducerVrfEvaluatorStatus::ReadinessCheck {
            time: redux::Timestamp::global_now(),
            current_epoch_number: 2,
            is_current_epoch_evaluated: true,
            is_next_epoch_evaluated: false,
            transition_frontier_size: 290,
            // Best tip at position that the ledger is set and materialized
            current_best_tip_height: 15190,
            last_evaluated_epoch: Some(2),
            last_epoch_block_height: Some(14900),
        };

        test_set_epoch_context(&mut vrf_evaluator_state, EpochContext::Next);
    }
}
