use std::str::FromStr;

use mina_p2p_messages::v2::EpochSeed;
use openmina_node_account::AccountSecretKey;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;
use vrf::{VrfEvaluationInput, VrfEvaluationOutput};

use crate::{
    evaluator::epoch::SlotData,
    node::{calc_slot_timestamp, epoch_ledgers::Ledger},
    storage::db_sled::Database,
};

pub mod epoch;

pub struct Evaluator {
    key: AccountSecretKey,
    db: Database,
    receiver: UnboundedReceiver<EpochInit>,
}

impl Evaluator {
    pub fn spawn_new(
        key: AccountSecretKey,
        db: Database,
        receiver: UnboundedReceiver<EpochInit>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move { Self { key, db, receiver }.run().await })
    }

    // TODO(adonagy): can be paralellized on slots
    async fn run(&mut self) {
        if let Some(init) = self.receiver.recv().await {
            let (start, end) = init.bounds;
            println!("Evaluating slots: {start} - {end}");
            let total_currency = init.ledger.total_currency();

            let pub_key = self.key.public_key();

            let delegates = init
                .ledger
                .gather_producer_and_delegates(&self.key.public_key().to_string());

            let epoch_seed = EpochSeed::from_str(&init.seed).unwrap();

            for global_slot in start..=end {
                // initially set to lost, the winning will overwrite it
                let timestamp = calc_slot_timestamp(init.genesis_timestamp, global_slot);
                let _ = self
                    .db
                    .store_slot(global_slot, &SlotData::new_lost(global_slot, timestamp));
                for (index, delegate) in &delegates {
                    let vrf_input = VrfEvaluationInput {
                        producer_key: self.key.clone().into(),
                        global_slot,
                        epoch_seed: epoch_seed.clone(),
                        account_pub_key: pub_key.clone(),
                        delegator_index: (*index).into(),
                        delegated_stake: delegate.balance.clone().into(),
                        total_currency: total_currency.clone(),
                    };

                    if let Ok(VrfEvaluationOutput::SlotWon(_)) = vrf::evaluate_vrf(vrf_input) {
                        println!("Won slot: {global_slot}");

                        // TODO(adonagy): handle error
                        let _ = self
                            .db
                            .store_slot(global_slot, &SlotData::new(global_slot, timestamp, None));
                        break;
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct EpochInit {
    _epoch_number: u32,
    ledger: Ledger,
    seed: String,
    bounds: (u32, u32),
    genesis_timestamp: i64,
}

impl EpochInit {
    pub fn new(
        epoch_number: u32,
        ledger: Ledger,
        seed: String,
        bounds: (u32, u32),
        genesis_timestamp: i64,
    ) -> Self {
        Self {
            _epoch_number: epoch_number,
            ledger,
            seed,
            bounds,
            genesis_timestamp,
        }
    }
}
