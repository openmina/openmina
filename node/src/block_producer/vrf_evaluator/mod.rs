//! VRF evaluator module for block production.
//! Responsible for evaluating VRF (Verifiable Random Function) to determine
//! if the node has won a slot and can produce a block.

mod block_producer_vrf_evaluator_state;
pub use block_producer_vrf_evaluator_state::*;

mod block_producer_vrf_evaluator_event;
pub use block_producer_vrf_evaluator_event::*;

mod block_producer_vrf_evaluator_actions;
pub use block_producer_vrf_evaluator_actions::*;

mod block_producer_vrf_evaluator_reducer;

use std::collections::BTreeMap;
use std::sync::Arc;

use ledger::AccountIndex;
use mina_p2p_messages::v2::{EpochSeed, LedgerHash};
use serde::{Deserialize, Serialize};
use vrf::{VrfEvaluationOutput, VrfWonSlot};

use crate::account::AccountPublicKey;

/// Maps account indices to delegator public keys and their stake amounts.
/// Used for VRF evaluation to determine slot winners based on stake.
pub type DelegatorTable = BTreeMap<AccountIndex, (AccountPublicKey, u64)>;

/// Input parameters required for VRF evaluation.
/// Contains all the necessary data to determine if a node has won a slot.
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct VrfEvaluatorInput {
    pub epoch_seed: EpochSeed,
    pub delegator_table: Arc<DelegatorTable>,
    pub global_slot: u32,
    pub total_currency: u64,
    pub staking_ledger_hash: LedgerHash,
}

/// A won slot with the associated staking ledger hash.
/// Combines the VRF output indicating a won slot with the ledger hash used for evaluation.
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

/// VRF evaluation result with the associated staking ledger hash.
/// Contains either a won slot or information about a lost slot.
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct VrfEvaluationOutputWithHash {
    pub evaluation_result: VrfEvaluationOutput,
    pub staking_ledger_hash: LedgerHash,
}

impl std::fmt::Display for VrfEvaluationOutputWithHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ", self.staking_ledger_hash)?;
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
        epoch_seed: EpochSeed,
        delegator_table: Arc<DelegatorTable>,
        global_slot: u32,
        total_currency: u64,
        staking_ledger_hash: LedgerHash,
    ) -> Self {
        Self {
            epoch_seed,
            delegator_table,
            global_slot,
            total_currency,
            staking_ledger_hash,
        }
    }
}
