use std::collections::BTreeMap;
use std::sync::Arc;

use mina_p2p_messages::v2::LedgerHash;
use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};

use crate::block_producer::BlockProducerWonSlot;

use super::{DelegatorTable, VrfWonSlotWithHash};

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
    pub current_epoch_data: Option<EpochData>,
    pub next_epoch_data: Option<EpochData>,
    // TODO(adonagy): move to block producer state probably
    pub current_epoch: Option<u32>,
    pub current_best_tip_slot: u32,
    pub latest_evaluated_slot: u32,
    pub last_possible_evaluation_slot: u32,
    pub genesis_timestamp: redux::Timestamp,
}

impl BlockProducerVrfEvaluatorState {
    pub fn new(now: redux::Timestamp) -> Self {
        Self {
            status: BlockProducerVrfEvaluatorStatus::Idle { time: now },
            won_slots: Default::default(),
            current_epoch_data: Default::default(),
            next_epoch_data: Default::default(),
            current_epoch: None,
            current_best_tip_slot: Default::default(),
            latest_evaluated_slot: Default::default(),
            last_possible_evaluation_slot: Default::default(),
            genesis_timestamp: redux::Timestamp::ZERO,
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
            .filter(|won_slot| won_slot > best_tip)
            .next()
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
pub enum BlockProducerVrfEvaluatorStatus {
    Idle {
        time: redux::Timestamp,
    },
    EpochChanged {
        time: redux::Timestamp,
    },
    DataPending {
        time: redux::Timestamp,
    },
    DataSuccess {
        time: redux::Timestamp,
    },
    DataFail {
        time: redux::Timestamp,
    },
    SlotsRequested {
        time: redux::Timestamp,
        global_slot: u32,
        staking_ledger_hash: LedgerHash,
    },
    SlotsReceived {
        time: redux::Timestamp,
        global_slot: u32,
        staking_ledger_hash: LedgerHash,
    },
}

impl BlockProducerVrfEvaluatorStatus {
    pub fn matches_requested_slot(
        &self,
        expected_global_slot: u32,
        expected_staking_ledger_hash: &LedgerHash,
    ) -> bool {
        match self {
            Self::SlotsRequested {
                global_slot,
                staking_ledger_hash,
                ..
            } => {
                &expected_global_slot == global_slot
                    && expected_staking_ledger_hash == staking_ledger_hash
            }
            _ => false,
        }
    }
}
