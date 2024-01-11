mod block_producer_vrf_evaluator_state;
use ledger::AccountIndex;
use std::collections::BTreeMap;
use crate::account::AccountPublicKey;

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
use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Deserialize, Clone, Serialize)]
pub struct VrfEvaluatorInput {
    pub epoch_seed: String,
    pub delegatee_table: BTreeMap<AccountIndex, (AccountPublicKey, u64)>,
    pub global_slot: u32,
    pub total_currency: u64,
}

impl VrfEvaluatorInput {
    pub fn new(
        epoch_seed: String,
        delegatee_table: BTreeMap<AccountIndex, (AccountPublicKey, u64)>,
        global_slot: u32,
        total_currency: u64,
    ) -> Self {
        Self {
            epoch_seed,
            delegatee_table,
            global_slot,
            total_currency,
        }
    }
}