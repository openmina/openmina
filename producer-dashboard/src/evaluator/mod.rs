use std::str::FromStr;
use std::sync::Arc;

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

    async fn run(&mut self) {
        while let Some(init) = self.receiver.recv().await {
            let (start, end) = init.bounds;
            println!("[evaluator] Evaluating slots: {start} - {end}");
            let total_currency = init.ledger.total_currency();

            let pub_key = self.key.public_key();

            let delegates = Arc::new(init.ledger.gather_producer_and_delegates(&self.key.public_key().to_string()));

            let epoch_seed = Arc::new(EpochSeed::from_str(&init.seed).unwrap());

            let db = Arc::new(self.db.clone());
            let key = Arc::new(self.key.clone());

            let chunk_size: u32 = 50; // Adjust based on your specific use case
            let mut computed_slots = 0;

            for chunk_start in (start..=end).step_by(chunk_size as usize) {
                let chunk_end = (chunk_start + chunk_size - 1).min(end);
                let tasks: Vec<_> = (chunk_start..=chunk_end).map(|global_slot| {
                    let delegates = delegates.clone();
                    let epoch_seed = epoch_seed.clone();
                    let db = db.clone();
                    let key = key.clone();
                    let pub_key = pub_key.clone();
                    let total_currency = total_currency.clone();
                    let genesis_timestamp = init.genesis_timestamp;

                    tokio::spawn(async move {
                        let timestamp = calc_slot_timestamp(genesis_timestamp, global_slot);
                        let mut slot_data = SlotData::new_lost(global_slot, timestamp);

                        for (index, delegate) in delegates.iter() {
                            let vrf_input = VrfEvaluationInput::new(
                                (*key).clone().into(),
                                (*epoch_seed).clone(),
                                pub_key.clone(),
                                global_slot,
                                (*index).into(),
                                delegate.balance.clone().into(),
                                total_currency.clone(),
                            );

                            if let Ok(VrfEvaluationOutput::SlotWon(_)) = vrf::evaluate_vrf(vrf_input) {
                                println!("Won slot: {global_slot}");
                                slot_data = SlotData::new(global_slot, timestamp, None);
                                break;
                            }
                        }

                        db.store_slot(global_slot, &slot_data)
                    })
                }).collect();

                futures::future::join_all(tasks).await;
                computed_slots += chunk_end - chunk_start + 1;
                println!("Computed {} slots so far", computed_slots);
            }

            println!("Finished computing all {} slots", computed_slots);
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
