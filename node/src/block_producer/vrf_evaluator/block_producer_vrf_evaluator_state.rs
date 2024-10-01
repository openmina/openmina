use std::collections::BTreeMap;
use std::sync::Arc;

use mina_p2p_messages::v2;
use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};

use crate::{account::AccountPublicKey, block_producer::BlockProducerWonSlot};

use super::{DelegatorTable, VrfEvaluatorInput, VrfWonSlotWithHash};

pub const SLOTS_PER_EPOCH: u32 = 7140;
/// Vrf evaluator sub-state
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerVrfEvaluatorState {
    pub status: BlockProducerVrfEvaluatorStatus,
    pub won_slots: BTreeMap<u32, VrfWonSlotWithHash>,
    pub latest_evaluated_slot: u32,
    pub genesis_timestamp: redux::Timestamp,
    last_evaluated_epoch: Option<u32>,
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
            best_tip_epoch,
            root_block_epoch,
            current_epoch,
            staking_epoch_data,
            next_epoch_data,
            ..
        } = self.status.clone()
        {
            // handle edge cases when the genesis injection triggers a best tip update
            // do not start evaluating if the true current epoch is greater than the best tip epoch
            if let Some(current_epoch) = current_epoch {
                if current_epoch > best_tip_epoch {
                    self.epoch_context = EpochContext::Waiting;
                    return;
                }
            } else {
                self.epoch_context = EpochContext::Waiting;
                return;
            }

            if !self.is_epoch_evaluated(best_tip_epoch) {
                self.epoch_context = EpochContext::Current((*staking_epoch_data).into())
            } else if !self.is_epoch_evaluated(best_tip_epoch + 1) {
                if root_block_epoch == best_tip_epoch {
                    self.epoch_context = EpochContext::Next((*next_epoch_data).into())
                } else {
                    self.epoch_context = EpochContext::Waiting
                }
            } else {
                self.epoch_context = EpochContext::Waiting
            }
        }
    }

    pub fn is_idle(&self) -> bool {
        matches!(self.status, BlockProducerVrfEvaluatorStatus::Idle { .. })
    }

    pub fn is_initialized(&self) -> bool {
        !matches!(
            self.status,
            BlockProducerVrfEvaluatorStatus::Idle { .. }
                | BlockProducerVrfEvaluatorStatus::InitialisationPending { .. }
        )
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
                | BlockProducerVrfEvaluatorStatus::EpochEvaluationInterrupted { .. }
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

    pub fn initialize_evaluator(&mut self, _epoch: u32, _last_height: u32) {}

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

    pub fn currently_evaluated_epoch(&self) -> Option<u32> {
        self.pending_evaluation.as_ref().map(|pe| pe.epoch_number)
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

    pub fn retention_slot(&self, current_epoch_number: &u32) -> u32 {
        const PAST_EPOCHS_TO_KEEP: u32 = 2;
        let cutoff_epoch = current_epoch_number.saturating_sub(PAST_EPOCHS_TO_KEEP);
        (cutoff_epoch * SLOTS_PER_EPOCH).saturating_sub(1)
    }

    pub fn cleanup_old_won_slots(&mut self, current_epoch_number: &u32) {
        let cutoff_slot = self.retention_slot(current_epoch_number);
        self.won_slots
            .retain(|global_slot, _| cutoff_slot < *global_slot);
    }

    /// If we need to construct delegator table, get it's inputs.
    pub fn vrf_delegator_table_inputs(&self) -> Option<(&v2::LedgerHash, &AccountPublicKey)> {
        match &self.status {
            BlockProducerVrfEvaluatorStatus::EpochDelegatorTablePending {
                staking_epoch_ledger_hash,
                producer,
                ..
            } => Some((staking_epoch_ledger_hash, producer)),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EpochData {
    pub seed: v2::EpochSeed,
    pub ledger: v2::LedgerHash,
    pub delegator_table: Arc<DelegatorTable>,
    pub total_currency: u64,
}

impl EpochData {
    pub fn new(seed: v2::EpochSeed, ledger: v2::LedgerHash, total_currency: u64) -> Self {
        Self {
            seed,
            ledger,
            total_currency,
            delegator_table: Default::default(),
        }
    }
}

impl From<v2::ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1> for EpochData {
    fn from(
        value: v2::ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
    ) -> Self {
        Self {
            seed: value.seed,
            ledger: value.ledger.hash,
            delegator_table: Default::default(),
            total_currency: value.ledger.total_currency.as_u64(),
        }
    }
}

impl From<v2::ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1> for EpochData {
    fn from(value: v2::ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1) -> Self {
        Self {
            seed: value.seed,
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
        best_tip_epoch: u32,
        is_current_epoch_evaluated: bool,
        is_next_epoch_evaluated: bool,

        best_tip_slot: u32,
        best_tip_global_slot: u32,
        next_epoch_first_slot: u32,
        staking_epoch_data: EpochData,
        producer: AccountPublicKey,
    },
    /// Waiting for delegator table building
    EpochDelegatorTablePending {
        time: redux::Timestamp,
        staking_epoch_ledger_hash: v2::LedgerHash,

        best_tip_epoch: u32,
        best_tip_slot: u32,
        best_tip_global_slot: u32,
        next_epoch_first_slot: u32,
        staking_epoch_data: EpochData,
        producer: AccountPublicKey,
    },
    /// Delegator table built successfully
    EpochDelegatorTableSuccess {
        time: redux::Timestamp,
        staking_epoch_ledger_hash: v2::LedgerHash,

        best_tip_epoch: u32,
        best_tip_slot: u32,
        best_tip_global_slot: u32,
        next_epoch_first_slot: u32,
        staking_epoch_data: EpochData,
        producer: AccountPublicKey,
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
    WaitingForNextEvaluation { time: redux::Timestamp },
    /// Checking whether the evaluator is abble to evaluate the epoch
    /// Note: The current epoch can be allways evaluated right away
    ReadinessCheck {
        time: redux::Timestamp,
        current_epoch: Option<u32>,
        best_tip_epoch: u32,
        root_block_epoch: u32,
        is_current_epoch_evaluated: bool,
        is_next_epoch_evaluated: bool,
        last_evaluated_epoch: Option<u32>,
        staking_epoch_data:
            Box<v2::ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1>,
        next_epoch_data: Box<v2::ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1>,
    },
    EpochBoundsCheck {
        time: redux::Timestamp,
        epoch_number: u32,
        latest_evaluated_global_slot: u32,
        epoch_current_bound: SlotPositionInEpoch,
    },
    EpochEvaluationInterrupted {
        time: redux::Timestamp,
        reason: InterruptReason,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum InterruptReason {
    BestTipWithHigherEpoch,
}

impl std::fmt::Display for InterruptReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BestTipWithHigherEpoch => write!(f, "Received best tip with higher epoch"),
        }
    }
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
            Self::EpochEvaluationInterrupted { .. } => write!(f, "EpochEvaluationInterrupted"),
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

// impl BlockProducerVrfEvaluatorStatus {
//     pub fn is_initialized(&self) -> bool {
//         !matches!(self, Self::Idle { .. })
//     }
// }

// TODO(adonagy): rework the tests so they track the situation with the genesis best tip update
#[cfg(test)]
mod test {
    use std::{collections::BTreeMap, str::FromStr, sync::Mutex};

    use lazy_static::lazy_static;
    use ledger::AccountIndex;
    use mina_p2p_messages::{
        bigint::BigInt,
        number::Number,
        v2::{
            ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
            ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
            CurrencyAmountStableV1, EpochSeed, LedgerHash, MinaBaseEpochLedgerValueStableV1,
            MinaBaseEpochSeedStableV1, StateHash, UnsignedExtendedUInt32StableV1,
            UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
        },
    };
    use openmina_node_account::AccountSecretKey;
    use vrf::VrfWonSlot;

    use crate::block_producer::vrf_evaluator::{
        BlockProducerVrfEvaluatorState, BlockProducerVrfEvaluatorStatus, EpochContext,
        SlotPositionInEpoch, VrfWonSlotWithHash,
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
                    current_epoch: Some(0), // TODO(adonagy)
                    root_block_epoch: 0,
                    best_tip_epoch: 0,
                    is_current_epoch_evaluated: false,
                    is_next_epoch_evaluated: false,
                    last_evaluated_epoch: None,
                    staking_epoch_data: Box::new(DUMMY_STAKING_EPOCH_DATA.to_owned()),
                    next_epoch_data: Box::new(DUMMY_NEXT_EPOCH_DATA.to_owned()),
                },
                won_slots: BTreeMap::new(),
                latest_evaluated_slot: 0,
                genesis_timestamp: redux::Timestamp::global_now(),
                last_evaluated_epoch: None,
                pending_evaluation: None,
                epoch_context: EpochContext::Current(DUMMY_STAKING_EPOCH_DATA.to_owned().into()),
            };
            Mutex::new(state)
        };
        static ref GENESIS_EPOCH_CURRENT_EPOCH_EVALUATED: Mutex<BlockProducerVrfEvaluatorState> = {
            let state = BlockProducerVrfEvaluatorState {
                status: BlockProducerVrfEvaluatorStatus::ReadinessCheck {
                    time: redux::Timestamp::global_now(),
                    current_epoch: Some(0), // TODO(adonagy)
                    root_block_epoch: 0,
                    best_tip_epoch: 0,
                    is_current_epoch_evaluated: true,
                    is_next_epoch_evaluated: false,
                    last_evaluated_epoch: Some(0),
                    staking_epoch_data: Box::new(DUMMY_STAKING_EPOCH_DATA.to_owned()),
                    next_epoch_data: Box::new(DUMMY_NEXT_EPOCH_DATA.to_owned()),
                },
                won_slots: BTreeMap::new(),
                latest_evaluated_slot: 7139,
                genesis_timestamp: redux::Timestamp::global_now(),
                last_evaluated_epoch: Some(0),
                pending_evaluation: None,
                epoch_context: EpochContext::Current(DUMMY_STAKING_EPOCH_DATA.to_owned().into()),
            };
            Mutex::new(state)
        };
        static ref GENESIS_EPOCH_NEXT_EPOCH_EVALUATED: Mutex<BlockProducerVrfEvaluatorState> = {
            let state = BlockProducerVrfEvaluatorState {
                status: BlockProducerVrfEvaluatorStatus::ReadinessCheck {
                    time: redux::Timestamp::global_now(),
                    current_epoch: Some(0), // TODO(adonagy)
                    root_block_epoch: 0,
                    best_tip_epoch: 0,
                    is_current_epoch_evaluated: true,
                    is_next_epoch_evaluated: true,
                    last_evaluated_epoch: Some(1),
                    staking_epoch_data: Box::new(DUMMY_STAKING_EPOCH_DATA.to_owned()),
                    next_epoch_data: Box::new(DUMMY_NEXT_EPOCH_DATA.to_owned()),
                },
                won_slots: BTreeMap::new(),
                latest_evaluated_slot: 14279,
                genesis_timestamp: redux::Timestamp::global_now(),
                last_evaluated_epoch: Some(1),
                pending_evaluation: None,
                epoch_context: EpochContext::Current(DUMMY_STAKING_EPOCH_DATA.to_owned().into()),
            };
            Mutex::new(state)
        };
        static ref SECOND_EPOCH_STARTUP: Mutex<BlockProducerVrfEvaluatorState> = {
            let state = BlockProducerVrfEvaluatorState {
                status: BlockProducerVrfEvaluatorStatus::ReadinessCheck {
                    time: redux::Timestamp::global_now(),
                    current_epoch: Some(2), // TODO(adonagy)
                    root_block_epoch: 2,
                    best_tip_epoch: 2,
                    is_current_epoch_evaluated: false,
                    is_next_epoch_evaluated: false,
                    last_evaluated_epoch: None,
                    staking_epoch_data: Box::new(DUMMY_STAKING_EPOCH_DATA.to_owned()),
                    next_epoch_data: Box::new(DUMMY_NEXT_EPOCH_DATA.to_owned()),
                },
                won_slots: BTreeMap::new(),
                latest_evaluated_slot: 0,
                genesis_timestamp: redux::Timestamp::global_now(),
                last_evaluated_epoch: None,
                pending_evaluation: None,
                epoch_context: EpochContext::Current(DUMMY_STAKING_EPOCH_DATA.to_owned().into()),
            };
            Mutex::new(state)
        };
        static ref SECOND_EPOCH_CURRENT_EPOCH_EVALUATED_WAIT_FOR_NEXT: Mutex<BlockProducerVrfEvaluatorState> = {
            let state = BlockProducerVrfEvaluatorState {
                status: BlockProducerVrfEvaluatorStatus::ReadinessCheck {
                    time: redux::Timestamp::global_now(),
                    current_epoch: Some(2), // TODO(adonagy)
                    root_block_epoch: 1,
                    best_tip_epoch: 2,
                    is_current_epoch_evaluated: true,
                    is_next_epoch_evaluated: false,
                    last_evaluated_epoch: Some(2),
                    staking_epoch_data: Box::new(DUMMY_STAKING_EPOCH_DATA.to_owned()),
                    next_epoch_data: Box::new(DUMMY_NEXT_EPOCH_DATA.to_owned()),
                },
                won_slots: BTreeMap::new(),
                latest_evaluated_slot: 21419,
                genesis_timestamp: redux::Timestamp::global_now(),
                last_evaluated_epoch: Some(2),
                pending_evaluation: None,
                epoch_context: EpochContext::Current(DUMMY_STAKING_EPOCH_DATA.to_owned().into()),
            };
            Mutex::new(state)
        };
    }

    #[test]
    fn correctly_set_epoch_context_on_startup() {
        let mut vrf_evaluator_state = GENESIS_EPOCH_FIRST_SLOT.lock().unwrap();
        vrf_evaluator_state.set_epoch_context();
        assert!(matches!(
            vrf_evaluator_state.epoch_context(),
            EpochContext::Current(_)
        ));
    }

    #[test]
    fn correctly_switch_to_next_epoch() {
        let mut vrf_evaluator_state = GENESIS_EPOCH_CURRENT_EPOCH_EVALUATED.lock().unwrap();
        vrf_evaluator_state.set_epoch_context();
        assert!(matches!(
            vrf_evaluator_state.epoch_context(),
            EpochContext::Next(_)
        ));
    }

    #[test]
    fn correctly_switch_to_waiting() {
        let mut vrf_evaluator_state = GENESIS_EPOCH_NEXT_EPOCH_EVALUATED.lock().unwrap();
        vrf_evaluator_state.set_epoch_context();
        assert!(matches!(
            vrf_evaluator_state.epoch_context(),
            EpochContext::Waiting
        ));
    }

    #[test]
    fn generic_epoch_set_epoch_context_on_startup() {
        let mut vrf_evaluator_state = SECOND_EPOCH_STARTUP.lock().unwrap();
        vrf_evaluator_state.set_epoch_context();
        assert!(matches!(
            vrf_evaluator_state.epoch_context(),
            EpochContext::Current(_)
        ));
    }

    #[test]
    fn generic_epoch_correctly_switch_to_next_epoch() {
        // Staking ledger not yet materialized (wait k=290 blocks)
        let mut vrf_evaluator_state = SECOND_EPOCH_CURRENT_EPOCH_EVALUATED_WAIT_FOR_NEXT
            .lock()
            .unwrap();
        vrf_evaluator_state.set_epoch_context();
        assert!(matches!(
            vrf_evaluator_state.epoch_context(),
            EpochContext::Waiting
        ));

        // Epoch has changed but the root is still from the previous epoch.
        // Next epoch must not be evaluated yet.
        vrf_evaluator_state.status = BlockProducerVrfEvaluatorStatus::ReadinessCheck {
            time: redux::Timestamp::global_now(),
            current_epoch: Some(2), // TODO(adonagy)
            root_block_epoch: 1,
            best_tip_epoch: 2,
            is_current_epoch_evaluated: true,
            is_next_epoch_evaluated: false,
            last_evaluated_epoch: Some(2),
            staking_epoch_data: Box::new(DUMMY_STAKING_EPOCH_DATA.to_owned()),
            next_epoch_data: Box::new(DUMMY_NEXT_EPOCH_DATA.to_owned()),
        };

        vrf_evaluator_state.set_epoch_context();
        assert!(matches!(
            vrf_evaluator_state.epoch_context(),
            EpochContext::Waiting
        ));

        // Epoch has changed, and the root is already at that epoch too.
        // Next epoch must be evaluated now.
        vrf_evaluator_state.status = BlockProducerVrfEvaluatorStatus::ReadinessCheck {
            time: redux::Timestamp::global_now(),
            current_epoch: Some(2), // TODO(adonagy)
            root_block_epoch: 2,
            best_tip_epoch: 2,
            is_current_epoch_evaluated: true,
            is_next_epoch_evaluated: false,
            last_evaluated_epoch: Some(2),
            staking_epoch_data: Box::new(DUMMY_STAKING_EPOCH_DATA.to_owned()),
            next_epoch_data: Box::new(DUMMY_NEXT_EPOCH_DATA.to_owned()),
        };

        vrf_evaluator_state.set_epoch_context();
        assert!(matches!(
            vrf_evaluator_state.epoch_context(),
            EpochContext::Next(_)
        ));
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

    fn generate_slots(
        start_slot: u32,
        end_slot: u32,
    ) -> impl Iterator<Item = (u32, VrfWonSlotWithHash)> {
        let dummy_ledger_hash =
            LedgerHash::from_str("jxTAZfKKDxoX4vtt68pQCWooXoVLjnfBpusaMwewrcZxsL3uWp6").unwrap();

        (start_slot..=end_slot).map(move |slot| {
            let dummy_won_slot = VrfWonSlot {
                producer: AccountSecretKey::genesis_producer().public_key(),
                winner_account: AccountSecretKey::genesis_producer().public_key(),
                vrf_output: Box::new(
                    vrf::genesis_vrf(EpochSeed::from(MinaBaseEpochSeedStableV1(BigInt::zero())))
                        .unwrap(),
                ),
                global_slot: slot,
                account_index: AccountIndex(0),
                value_with_threshold: None,
            };
            (
                slot,
                VrfWonSlotWithHash::new(dummy_won_slot, dummy_ledger_hash.clone()),
            )
        })
    }

    #[test]
    fn test_cleanup_old_won_slots() {
        // arbitrary, need it just to fill it with some slots
        let mut vrf_evaluator_state = SECOND_EPOCH_STARTUP.lock().unwrap();

        vrf_evaluator_state
            .won_slots
            .extend(generate_slots(1, 28560));

        assert_eq!(28560, vrf_evaluator_state.won_slots.len());

        // should keep all the slots
        vrf_evaluator_state.cleanup_old_won_slots(&1);
        assert_eq!(
            1,
            *vrf_evaluator_state.won_slots.first_key_value().unwrap().0,
            "First retained slot should be the first slot of the epoch, 7140"
        );

        // epoch 1 slots should be discarded
        vrf_evaluator_state.cleanup_old_won_slots(&3);
        assert_eq!(
            7140,
            *vrf_evaluator_state.won_slots.first_key_value().unwrap().0,
            "First retained slot should be the first slot of the epoch, 7140"
        );

        // epoch 2 slots should be discarded
        vrf_evaluator_state.cleanup_old_won_slots(&4);
        assert_eq!(
            14280,
            *vrf_evaluator_state.won_slots.first_key_value().unwrap().0,
            "First retained slot should be the first slot of the epoch, 142180"
        );
    }
}
