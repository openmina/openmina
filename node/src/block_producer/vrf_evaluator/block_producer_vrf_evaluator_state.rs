use std::collections::BTreeMap;

use ledger::AccountIndex;
use mina_p2p_messages::v2::LedgerHash;
use serde::{Deserialize, Serialize};
use vrf::VrfWonSlot;

use crate::account::AccountPublicKey;
use crate::BlockProducerConfig;

use super::VrfWonSlotWithHash;

// TODO(adonagy): consodilate types, make more clear
// pub type AccountAddressAndBalance = (String, u64);

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
    pub fn new(now: redux::Timestamp, config: BlockProducerConfig) -> Self {
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EpochData {
    pub seed: String,
    pub ledger: LedgerHash,
    pub delegator_table: BTreeMap<AccountIndex, (AccountPublicKey, u64)>,
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
    pub fn matches_requsted_slot(
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
