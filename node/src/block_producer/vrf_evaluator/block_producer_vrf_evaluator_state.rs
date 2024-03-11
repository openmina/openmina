use std::collections::BTreeMap;
use std::sync::Arc;

use mina_p2p_messages::v2::{
    ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
    ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1, LedgerHash,
};
use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};

use crate::block_producer::BlockProducerWonSlot;

use super::{DelegatorTable, VrfEvaluatorInput, VrfWonSlotWithHash};

/// Vrf evaluator sub-state
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
            epoch_context: EpochContext::Waiting,
        }
    }

    /// Determines the position of a slot within an epoch (at the beginning, end, or within the epoch).
    /// This function calculates the position based on the `global_slot` and a predefined number of slots per epoch.
    ///
    /// Arguments:
    /// - `global_slot`: A reference to a 32-bit unsigned integer representing the global slot number.
    ///
    /// Returns:
    /// - `SlotPositionInEpoch`: An enum indicating the slot's position (Beginning, End, or Within).
    pub fn evaluate_epoch_bounds(global_slot: &u32) -> SlotPositionInEpoch {
        const SLOTS_PER_EPOCH: u32 = 7140;

        if global_slot % SLOTS_PER_EPOCH == 0 {
            SlotPositionInEpoch::Beginning
        } else if (global_slot + 1) % SLOTS_PER_EPOCH == 0 {
            SlotPositionInEpoch::End
        } else {
            SlotPositionInEpoch::Within
        }
    }

    /// Fetches the next slot that has been won based on the current global slot and the best tip information.
    /// If there are no future won slots, returns `None`.
    ///
    /// Arguments:
    /// - `cur_global_slot`: The current global slot as a 32-bit unsigned integer.
    /// - `best_tip`: A reference to the `ArcBlockWithHash` representing the current best tip.
    ///
    /// Returns:
    /// - `Option<BlockProducerWonSlot>`: The next won slot, if any, as a `BlockProducerWonSlot` or `None` if there are no more slots won in the future.
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

    /// Retrieves the current epoch context.
    ///
    /// Returns:
    /// - `&EpochContext`: A reference to the `EpochContext`
    pub fn epoch_context(&self) -> &EpochContext {
        &self.epoch_context
    }

    /// Sets the epoch context based on the current evaluation status and the available data.
    /// This method updates the `epoch_context` field in the `ReadinessCheck` state
    pub fn set_epoch_context(&mut self) {
        // guard the epoch context change and permit it only if in the ReadinessCheck state
        if let BlockProducerVrfEvaluatorStatus::ReadinessCheck {
            last_epoch_block_height,
            current_best_tip_height,
            transition_frontier_size,
            current_epoch_number,
            staking_epoch_data,
            next_epoch_data,
            ..
        } = self.status.clone()
        {
            if !self.is_epoch_evaluated(current_epoch_number) {
                self.epoch_context = EpochContext::Current(staking_epoch_data.into())
            } else if !self.is_epoch_evaluated(current_epoch_number + 1) {
                if let Some(last_epoch_block_height) = last_epoch_block_height {
                    if (last_epoch_block_height + transition_frontier_size)
                        >= current_best_tip_height
                    {
                        self.epoch_context = EpochContext::Next(next_epoch_data.into())
                    } else {
                        self.epoch_context = EpochContext::Waiting
                    }
                } else {
                    // if last_epoch_block_height is not set, we are still in genesis epoch, Next epoch evaluation is possible
                    self.epoch_context = EpochContext::Next(next_epoch_data.into())
                }
            } else {
                self.epoch_context = EpochContext::Waiting
            }
        }
    }

    /// Determines if a given epoch number has already been evaluated.
    ///
    /// Arguments:
    /// - `epoch_number`: The epoch number as a 32-bit unsigned integer to check.
    ///
    /// Returns:
    /// - `bool`: `true` if the epoch has already been evaluated, otherwise `false`.
    pub fn is_epoch_evaluated(&self, epoch_number: u32) -> bool {
        if let Some(last_evaluated_epoch) = self.last_evaluated_epoch {
            last_evaluated_epoch >= epoch_number
        } else {
            false
        }
    }

    /// Retrieves the number of the last evaluated epoch, if any.
    ///
    /// Returns:
    /// - `Option<u32>`: The epoch number of the last evaluated epoch, or `None` if no epoch has been evaluated yet.

    pub fn last_evaluated_epoch(&self) -> Option<u32> {
        self.last_evaluated_epoch
    }

    /// Adds or updates the highest block height reached within a specified epoch.
    ///
    /// Arguments:
    /// - `epoch`: The epoch number as a 32-bit unsigned integer.
    /// - `height`: The block height as a 32-bit unsigned integer.
    pub fn add_last_height(&mut self, epoch: u32, height: u32) {
        self.last_block_heights_in_epoch.insert(epoch, height);
    }

    /// Retrieves the highest block height reached in a specified epoch, if available.
    ///
    /// Arguments:
    /// - `epoch`: The epoch number as a 32-bit unsigned integer.
    ///
    /// Returns:
    /// - `Option<u32>`: The highest block height within the specified epoch, or `None` if not available.
    pub fn last_height(&self, epoch: u32) -> Option<u32> {
        self.last_block_heights_in_epoch.get(&epoch).copied()
    }

    /// TODO: remove, not needed anymore
    pub fn latest_evaluated_global_slot(&self) -> u32 {
        self.latest_evaluated_slot
    }

    /// Returns `true` if the evaluator is in the `ReadinessCheck` state, otherwise `false`.
    pub fn is_readiness_check(&self) -> bool {
        matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::ReadinessCheck { .. }
        )
    }

    /// Returns `true` if the evaluator is in the `EpochBoundsCheck` state, otherwise `false`.
    pub fn is_epoch_bound_evaluated(&self) -> bool {
        matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::EpochBoundsCheck { .. }
        )
    }

    /// Returns true if the evaluator is in a state that can perform a `ReadinessCheck``, otherwise `false`.
    pub fn can_check_next_evaluation(&self) -> bool {
        matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::WaitingForNextEvaluation { .. }
                | BlockProducerVrfEvaluatorStatus::EpochEvaluationSuccess { .. }
                | BlockProducerVrfEvaluatorStatus::InitialisationComplete { .. }
        )
    }

    /// Returns `true` if the evaluator is in the `ReadyToEvaluate` state, otherwise `false`.
    pub fn can_construct_delegator_table(&self) -> bool {
        matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::ReadyToEvaluate { .. }
        )
    }

    /// Returns `true` if the evaluator is in the `EpochDelegatorTablePending` state, otherwise `false`.
    pub fn is_delegator_table_requested(&self) -> bool {
        matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::EpochDelegatorTablePending { .. }
        )
    }

    /// Returns `true` if the evaluator is in the `EpochDelegatorTableSuccess` state, otherwise `false`.
    pub fn is_delegator_table_constructed(&self) -> bool {
        matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::EpochDelegatorTableSuccess { .. }
        )
    }

    /// TODO: redundant fn, remove?
    pub fn can_start_epoch_evaluation(&self) -> bool {
        matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::EpochDelegatorTableSuccess { .. }
        )
    }

    /// Returns `true` if the evaluator is in any of the following states: `SlotEvaluationPending`,
    /// `SlotEvaluationReceived`, `EpochEvaluationPending`, `EpochBoundsCheck`, otherwise `false` .
    pub fn is_evaluating(&self) -> bool {
        matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::SlotEvaluationPending { .. }
                | BlockProducerVrfEvaluatorStatus::SlotEvaluationReceived { .. }
                | BlockProducerVrfEvaluatorStatus::EpochEvaluationPending { .. }
                | BlockProducerVrfEvaluatorStatus::EpochBoundsCheck { .. }
        )
    }

    // TODO(adonagy): review this, might be redundant and not needed anymore
    pub fn is_evaluation_pending(&self) -> bool {
        matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::EpochEvaluationPending { .. }
        )
    }

    /// Returns `true` if the evaluator is in the `InitialSlotSelection` state, otherwise `false`.
    pub fn is_slot_selection(&self) -> bool {
        matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::InitialSlotSelection { .. }
        )
    }

    /// Returns `true` if the evaluator is in the `SlotEvaluationPending` state, otherwise `false`.
    pub fn is_slot_requested(&self) -> bool {
        matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::SlotEvaluationPending { .. }
        )
    }

    /// Returns `true` if the evaluator is in the `SlotEvaluationReceived` state, otherwise `false`.
    pub fn is_slot_evaluated(&self) -> bool {
        matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::SlotEvaluationReceived { .. }
        )
    }

    pub fn set_latest_evaluated_global_slot(&mut self, global_slot: &u32) {
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
                EpochContext::Current(_) => self.last_evaluated_epoch = Some(*epoch_number),
                EpochContext::Next(_) => self.last_evaluated_epoch = Some(epoch_number + 1),
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

    pub fn get_epoch_bound_from_check(&self) -> Option<SlotPositionInEpoch> {
        if let BlockProducerVrfEvaluatorStatus::EpochBoundsCheck {
            epoch_current_bound,
            ..
        } = &self.status
        {
            Some(epoch_current_bound.clone())
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

impl From<ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1> for EpochData {
    fn from(value: ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1) -> Self {
        Self {
            seed: value.seed.to_string(),
            ledger: value.ledger.hash,
            delegator_table: Default::default(),
            total_currency: value.ledger.total_currency.as_u64(),
        }
    }
}

impl From<ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1> for EpochData {
    fn from(value: ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1) -> Self {
        Self {
            seed: value.seed.to_string(),
            ledger: value.ledger.hash,
            delegator_table: Default::default(),
            total_currency: value.ledger.total_currency.as_u64(),
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
        // next_vrf_input: Option<VrfEvaluatorInput>,
    },
    /// A slot was sent for evaluation to the vrf evaluator service
    SlotEvaluationPending {
        time: redux::Timestamp,
        global_slot: u32,
    },
    SlotEvaluationReceived {
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
        staking_epoch_data: ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
        next_epoch_data: ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
    },
    EpochBoundsCheck {
        time: redux::Timestamp,
        epoch_number: u32,
        latest_evaluated_global_slot: u32,
        epoch_current_bound: SlotPositionInEpoch,
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
            Self::EpochBoundsCheck { .. } => write!(f, "EpochBoundsCheck"),
            Self::SlotEvaluationReceived { .. } => write!(f, "SlotEvaluationReceived"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EpochContext {
    Current(EpochData),
    Next(EpochData),
    Waiting,
}

/// Represents the position of the slot in the epoch
/// `Beginning` -> First slot of an epoch
/// `Within` -> The slot is not at the epoch bounds, i.e. is withing the epoch
/// `End` -> The last slot of an epoch
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SlotPositionInEpoch {
    Beginning,
    Within,
    End,
}

impl std::fmt::Display for EpochContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Current(_) => write!(f, "Current"),
            Self::Next(_) => write!(f, "Next"),
            Self::Waiting => write!(f, "Waiting"),
        }
    }
}

impl EpochContext {
    pub fn get_epoch_data(&self) -> Option<EpochData> {
        match self {
            EpochContext::Current(epoch_data) | EpochContext::Next(epoch_data) => {
                Some(epoch_data.clone())
            }
            EpochContext::Waiting => None,
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
    use std::{collections::BTreeMap, str::FromStr, sync::Mutex};

    use lazy_static::lazy_static;
    use mina_p2p_messages::{
        bigint::BigInt,
        number::Number,
        v2::{
            ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
            ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
            CurrencyAmountStableV1, LedgerHash, MinaBaseEpochLedgerValueStableV1,
            MinaBaseEpochSeedStableV1, StateHash, UnsignedExtendedUInt32StableV1,
            UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
        },
    };

    use crate::block_producer::vrf_evaluator::{
        BlockProducerVrfEvaluatorState, BlockProducerVrfEvaluatorStatus, EpochContext,
        SlotPositionInEpoch,
    };

    lazy_static! {
        static ref DUMMY_STAKING_EPOCH_DATA: ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1 = {
            ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1 {
                ledger: MinaBaseEpochLedgerValueStableV1 {
                    hash: LedgerHash::from_str("jxTAZfKKDxoX4vtt68pQCWooXoVLjnfBpusaMwewrcZxsL3uWp6").unwrap(),
                    total_currency: CurrencyAmountStableV1(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(Number(10000000))),
                },
                // seed: MinaBaseEpochSeedStableV1(BigInt::from(0))
                seed: MinaBaseEpochSeedStableV1(BigInt::zero()).into(),
                start_checkpoint: StateHash::from_str("3NKxUSAJE3wqJkrtBhMYhwzrMq3B5sKjPJQRyXz1YrPWA7761opD").unwrap(),
                lock_checkpoint: StateHash::from_str("3NKxUSAJE3wqJkrtBhMYhwzrMq3B5sKjPJQRyXz1YrPWA7761opD").unwrap(),
                epoch_length: UnsignedExtendedUInt32StableV1(Number(7140))
            }
        };
        static ref DUMMY_NEXT_EPOCH_DATA: ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1 = {
            ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1 {
                ledger: MinaBaseEpochLedgerValueStableV1 {
                    hash: LedgerHash::from_str("jxTAZfKKDxoX4vtt68pQCWooXoVLjnfBpusaMwewrcZxsL3uWp6").unwrap(),
                    total_currency: CurrencyAmountStableV1(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(Number(10000000))),
                },
                // seed: MinaBaseEpochSeedStableV1(BigInt::from(0))
                seed: MinaBaseEpochSeedStableV1(BigInt::zero()).into(),
                start_checkpoint: StateHash::from_str("3NKxUSAJE3wqJkrtBhMYhwzrMq3B5sKjPJQRyXz1YrPWA7761opD").unwrap(),
                lock_checkpoint: StateHash::from_str("3NKxUSAJE3wqJkrtBhMYhwzrMq3B5sKjPJQRyXz1YrPWA7761opD").unwrap(),
                epoch_length: UnsignedExtendedUInt32StableV1(Number(7140))
            }
        };
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
                    staking_epoch_data: DUMMY_STAKING_EPOCH_DATA.to_owned(),
                    next_epoch_data: DUMMY_NEXT_EPOCH_DATA.to_owned(),
                },
                won_slots: BTreeMap::new(),
                latest_evaluated_slot: 0,
                genesis_timestamp: redux::Timestamp::global_now(),
                last_evaluated_epoch: None,
                last_block_heights_in_epoch: BTreeMap::new(),
                pending_evaluation: None,
                epoch_context: EpochContext::Current(DUMMY_STAKING_EPOCH_DATA.to_owned().into()),
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
                    staking_epoch_data: DUMMY_STAKING_EPOCH_DATA.to_owned(),
                    next_epoch_data: DUMMY_NEXT_EPOCH_DATA.to_owned(),
                },
                won_slots: BTreeMap::new(),
                latest_evaluated_slot: 7139,
                genesis_timestamp: redux::Timestamp::global_now(),
                last_evaluated_epoch: Some(0),
                last_block_heights_in_epoch: BTreeMap::new(),
                pending_evaluation: None,
                epoch_context: EpochContext::Current(DUMMY_STAKING_EPOCH_DATA.to_owned().into()),
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
                    staking_epoch_data: DUMMY_STAKING_EPOCH_DATA.to_owned(),
                    next_epoch_data: DUMMY_NEXT_EPOCH_DATA.to_owned(),
                },
                won_slots: BTreeMap::new(),
                latest_evaluated_slot: 14279,
                genesis_timestamp: redux::Timestamp::global_now(),
                last_evaluated_epoch: Some(1),
                last_block_heights_in_epoch: BTreeMap::new(),
                pending_evaluation: None,
                epoch_context: EpochContext::Current(DUMMY_STAKING_EPOCH_DATA.to_owned().into()),
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
                    staking_epoch_data: DUMMY_STAKING_EPOCH_DATA.to_owned(),
                    next_epoch_data: DUMMY_NEXT_EPOCH_DATA.to_owned(),
                },
                won_slots: BTreeMap::new(),
                latest_evaluated_slot: 0,
                genesis_timestamp: redux::Timestamp::global_now(),
                last_evaluated_epoch: None,
                last_block_heights_in_epoch: BTreeMap::new(),
                pending_evaluation: None,
                epoch_context: EpochContext::Current(DUMMY_STAKING_EPOCH_DATA.to_owned().into()),
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
                    staking_epoch_data: DUMMY_STAKING_EPOCH_DATA.to_owned(),
                    next_epoch_data: DUMMY_NEXT_EPOCH_DATA.to_owned(),
                },
                won_slots: BTreeMap::new(),
                latest_evaluated_slot: 21419,
                genesis_timestamp: redux::Timestamp::global_now(),
                last_evaluated_epoch: Some(2),
                last_block_heights_in_epoch: BTreeMap::new(),
                pending_evaluation: None,
                epoch_context: EpochContext::Current(DUMMY_STAKING_EPOCH_DATA.to_owned().into()),
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
        test_set_epoch_context(
            &mut vrf_evaluator_state,
            EpochContext::Current(DUMMY_STAKING_EPOCH_DATA.to_owned().into()),
        )
    }

    #[test]
    fn correctly_switch_to_next_epoch() {
        let mut vrf_evaluator_state = GENESIS_EPOCH_CURRENT_EPOCH_EVALUATED.lock().unwrap();
        test_set_epoch_context(
            &mut vrf_evaluator_state,
            EpochContext::Next(DUMMY_NEXT_EPOCH_DATA.to_owned().into()),
        )
    }

    #[test]
    fn correctly_switch_to_waiting() {
        let mut vrf_evaluator_state = GENESIS_EPOCH_NEXT_EPOCH_EVALUATED.lock().unwrap();
        test_set_epoch_context(&mut vrf_evaluator_state, EpochContext::Waiting)
    }

    #[test]
    fn generic_epoch_set_epoch_context_on_startup() {
        let mut vrf_evaluator_state = SECOND_EPOCH_STARTUP.lock().unwrap();
        test_set_epoch_context(
            &mut vrf_evaluator_state,
            EpochContext::Current(DUMMY_STAKING_EPOCH_DATA.to_owned().into()),
        )
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
            staking_epoch_data: DUMMY_STAKING_EPOCH_DATA.to_owned(),
            next_epoch_data: DUMMY_NEXT_EPOCH_DATA.to_owned(),
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
            staking_epoch_data: DUMMY_STAKING_EPOCH_DATA.to_owned(),
            next_epoch_data: DUMMY_NEXT_EPOCH_DATA.to_owned(),
        };

        test_set_epoch_context(
            &mut vrf_evaluator_state,
            EpochContext::Next(DUMMY_NEXT_EPOCH_DATA.to_owned().into()),
        );
    }

    #[test]
    fn test_evaluate_epoch_bounds() {
        const GENESIS_EPOCH_BEGINNING: u32 = 0;
        const GENESIS_EPOCH_WITHIN: u32 = 2000;
        const GENESIS_EPOCH_END: u32 = 7139;

        const BEGINNING: u32 = 7140;
        const WITHIN: u32 = 7500;
        const END: u32 = 14279;

        let res = BlockProducerVrfEvaluatorState::evaluate_epoch_bounds(&GENESIS_EPOCH_BEGINNING);
        assert!(matches!(res, SlotPositionInEpoch::Beginning));
        let res = BlockProducerVrfEvaluatorState::evaluate_epoch_bounds(&GENESIS_EPOCH_WITHIN);
        assert!(matches!(res, SlotPositionInEpoch::Within));
        let res = BlockProducerVrfEvaluatorState::evaluate_epoch_bounds(&GENESIS_EPOCH_END);
        assert!(matches!(res, SlotPositionInEpoch::End));

        let res = BlockProducerVrfEvaluatorState::evaluate_epoch_bounds(&BEGINNING);
        assert!(matches!(res, SlotPositionInEpoch::Beginning));

        let res = BlockProducerVrfEvaluatorState::evaluate_epoch_bounds(&WITHIN);
        assert!(matches!(res, SlotPositionInEpoch::Within));

        let res = BlockProducerVrfEvaluatorState::evaluate_epoch_bounds(&END);
        assert!(matches!(res, SlotPositionInEpoch::End));
    }
}
