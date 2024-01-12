use std::collections::BTreeMap;

use ledger::AccountIndex;
use serde::{Deserialize, Serialize};
use vrf::VrfWonSlot;

use crate::account::AccountPublicKey;
use crate::BlockProducerConfig;

// TODO(adonagy): consodilate types, make more clear
// pub type AccountAddressAndBalance = (String, u64);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockProducerVrfEvaluatorState {
    pub status: BlockProducerVrfEvaluatorStatus,
    pub won_slots: BTreeMap<u32, VrfWonSlot>,
    pub current_epoch_data: EpochData,
    pub next_epoch_data: EpochData,
    pub producer_pub_key: String,
    // TODO(adonagy): move to block producer state probably
    pub current_epoch: Option<u32>,
    pub current_best_tip_slot: u32,
    pub latest_evaluated_slot: u32,
    pub last_possible_evaluation_slot: u32,
    pub genesis_timestamp: redux::Timestamp,
}

impl BlockProducerVrfEvaluatorState {
    pub fn new(now: redux::Timestamp, config: BlockProducerConfig) -> Self {
        let producer_pub_key = config.pub_key.to_string();
        Self {
            status: BlockProducerVrfEvaluatorStatus::Idle { time: now },
            won_slots: Default::default(),
            current_epoch_data: Default::default(),
            next_epoch_data: Default::default(),
            producer_pub_key,
            current_epoch: None,
            current_best_tip_slot: Default::default(),
            latest_evaluated_slot: Default::default(),
            last_possible_evaluation_slot: Default::default(),
            genesis_timestamp: redux::Timestamp::ZERO,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EpochData {
    pub seed: String,
    pub ledger: String,
    pub delegator_table: BTreeMap<AccountIndex, (AccountPublicKey, u64)>,
    pub total_currency: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BlockProducerVrfEvaluatorStatus {
    Idle { time: redux::Timestamp },
    EpochChanged { time: redux::Timestamp },
    DataPending { time: redux::Timestamp },
    DataSuccess { time: redux::Timestamp },
    DataFail { time: redux::Timestamp },
    SlotsRequested { time: redux::Timestamp },
    SlotsReceived { time: redux::Timestamp },
}
