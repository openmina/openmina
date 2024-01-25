mod block_producer_vrf_evaluator_state;
use crate::account::AccountPublicKey;
use ledger::AccountIndex;
use mina_p2p_messages::v2::LedgerHash;
use std::collections::BTreeMap;
use vrf::{VrfEvaluationOutput, VrfWonSlot};

pub use block_producer_vrf_evaluator_state::*;

mod block_producer_vrf_evaluator_event;
pub use block_producer_vrf_evaluator_event::*;

mod block_producer_vrf_evaluator_actions;
pub use block_producer_vrf_evaluator_actions::*;

mod block_producer_vrf_evaluator_reducer;
pub use block_producer_vrf_evaluator_reducer::*;

mod block_producer_vrf_evaluator_effects;
pub use block_producer_vrf_evaluator_effects::*;

mod block_producer_vrf_evaluator_service;
pub use block_producer_vrf_evaluator_service::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct VrfEvaluatorInput {
    pub epoch_seed: String,
    pub delegatee_table: BTreeMap<AccountIndex, (AccountPublicKey, u64)>,
    pub global_slot: u32,
    pub total_currency: u64,
    pub staking_ledger_hash: LedgerHash,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct VrfWonSlotWithHash {
    pub won_slot: VrfWonSlot,
    pub staking_ledger_hash: LedgerHash,
}

impl VrfWonSlotWithHash {
    pub fn new(won_slot: VrfWonSlot, staking_ledger_hash: LedgerHash) -> Self {
        Self {
            won_slot,
            staking_ledger_hash,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct VrfEvaluationOutputWithHash {
    pub evaluation_result: VrfEvaluationOutput,
    pub staking_ledger_hash: LedgerHash,
}

impl std::fmt::Display for VrfEvaluationOutputWithHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ", self.staking_ledger_hash.to_string())?;
        match &self.evaluation_result {
            VrfEvaluationOutput::SlotWon(won_slot) => {
                write!(f, "SlotWon {}", won_slot.global_slot)
            }
            VrfEvaluationOutput::SlotLost(global_slot) => {
                write!(f, "SlotLost {}", global_slot)
            }
        }
    }
}

impl VrfEvaluationOutputWithHash {
    pub fn new(evaluation_result: VrfEvaluationOutput, staking_ledger_hash: LedgerHash) -> Self {
        Self {
            evaluation_result,
            staking_ledger_hash,
        }
    }
}

impl VrfEvaluatorInput {
    pub fn new(
        epoch_seed: String,
        delegatee_table: BTreeMap<AccountIndex, (AccountPublicKey, u64)>,
        global_slot: u32,
        total_currency: u64,
        staking_ledger_hash: LedgerHash,
    ) -> Self {
        Self {
            epoch_seed,
            delegatee_table,
            global_slot,
            total_currency,
            staking_ledger_hash,
        }
    }
}
