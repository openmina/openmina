use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;
use std::str::FromStr;

use mina_p2p_messages::v2::EpochSeed;
use openmina_node_account::AccountSecretKey;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;
use vrf::{VrfEvaluationInput, VrfEvaluationOutput, VrfWonSlot};

use crate::epoch::{EpochStorage, EpochData};
use crate::ledger::Ledger;

pub struct Evaluator {
    key: AccountSecretKey,
    storage: EpochStorage,
    receiver: UnboundedReceiver<EpochInit>,
    // ledgers_dir ?
}

impl Evaluator {
    pub fn spawn_new(
        key: AccountSecretKey,
        storage: EpochStorage,
        receiver: UnboundedReceiver<EpochInit>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            Self {
                key,
                storage,
                receiver
            }.run().await
        })
    }

    async fn run(&mut self) {
        if let Some(init) = self.receiver.recv().await {
            let (start, end) = init.bounds;
            println!("Evaluating slots: {start} - {end}");
            println!("PWD: {}", std::env::current_dir().unwrap().display());  
            let ledger = Ledger::load_from_file(init.ledger_path).unwrap();
            let total_currency = ledger.total_currency();

            let pub_key = self.key.public_key().to_string();

            let delegates = ledger.gather_producer_and_delegates(&self.key.public_key().to_string());

            let epoch_seed = EpochSeed::from_str(&init.seed).unwrap();

            let mut won_slots: BTreeMap<u32, VrfWonSlot> = BTreeMap::new();

            for global_slot in start..end {
                for (index, delegate) in &delegates {
                    let vrf_input = VrfEvaluationInput::new(
                        self.key.clone().into(),
                        epoch_seed.clone(),
                        pub_key.clone(),
                        global_slot,
                        (*index).into(),
                        delegate.balance.clone().into(),
                        total_currency.clone(),
                    );

                    if let Ok(VrfEvaluationOutput::SlotWon(won_slot)) = vrf::evaluate_vrf(vrf_input) {
                        println!("Won slot: {global_slot}");
                        won_slots.insert(won_slot.global_slot, won_slot);
                        break;
                    }
                }
            }
            let epoch_data = EpochData::new(init.epoch_number, won_slots);
            self.storage.insert(init.epoch_number, epoch_data);
        }
    }
}

#[derive(Debug, Clone)]
pub struct EpochInit {
    epoch_number: u32,
    ledger_path: PathBuf,
    seed: String,
    bounds: (u32, u32),
}

impl EpochInit {
    pub fn new(epoch_number: u32, ledger_path: PathBuf, seed: String, bounds: (u32, u32)) -> Self {
        Self {
            epoch_number,
            ledger_path,
            seed,
            bounds,
        }
    }
}
