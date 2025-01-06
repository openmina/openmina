use mina_signer::Keypair;
use node::{
    block_producer::BlockProducerVrfEvaluatorEvent,
    block_producer::{
        vrf_evaluator::{VrfEvaluationOutputWithHash, VrfEvaluatorInput},
        BlockProducerEvent,
    },
    core::channels::mpsc::{UnboundedReceiver, UnboundedSender},
    event_source::Event,
};
use vrf::{VrfEvaluationInput, VrfEvaluationOutput};

use crate::NodeService;

pub fn vrf_evaluator(
    event_sender: UnboundedSender<Event>,
    mut vrf_evaluation_receiver: UnboundedReceiver<VrfEvaluatorInput>,
    keypair: Keypair,
) {
    while let Some(vrf_evaluator_input) = vrf_evaluation_receiver.blocking_recv() {
        // let bytes = serde_json::to_string(&vrf_evaluator_input).unwrap();
        // openmina_core::http::download("vrf.json".to_string(), bytes.as_bytes().to_vec()).unwrap();

        let keypair = &keypair;
        let VrfEvaluatorInput {
            epoch_seed,
            delegator_table,
            global_slot,
            total_currency,
            staking_ledger_hash: _,
        } = &vrf_evaluator_input;

        let vrf_result = delegator_table
            .iter()
            .find_map(|(index, (pub_key, stake))| {
                let vrf_input = VrfEvaluationInput {
                    producer_key: keypair.clone(),
                    global_slot: *global_slot,
                    epoch_seed: epoch_seed.clone(),
                    account_pub_key: pub_key.clone(),
                    delegator_index: *index,
                    delegated_stake: (*stake).into(),
                    total_currency: (*total_currency).into(),
                };

                let vrf_result = vrf::evaluate_vrf(vrf_input).unwrap();

                // the first delegate that won the slot
                if let VrfEvaluationOutput::SlotWon(_) = vrf_result {
                    return Some(vrf_result);
                }
                None
            })
            .unwrap_or(VrfEvaluationOutput::SlotLost(*global_slot));

        let vrf_result_with_hash = VrfEvaluationOutputWithHash::new(
            vrf_result,
            vrf_evaluator_input.staking_ledger_hash.clone(),
        );

        // send the result back to the state machine
        let _ = event_sender.send(
            BlockProducerEvent::VrfEvaluator(BlockProducerVrfEvaluatorEvent::Evaluated(
                vrf_result_with_hash,
            ))
            .into(),
        );
    }
}

impl node::block_producer_effectful::vrf_evaluator_effectful::BlockProducerVrfEvaluatorService
    for NodeService
{
    fn evaluate(&mut self, data: VrfEvaluatorInput) {
        if let Some(bp) = self.block_producer.as_mut() {
            let _ = bp.vrf_evaluation_sender.send(data);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    // use mina_signer::keypair;
    use node::account::AccountSecretKey;

    use super::*;

    #[test]
    #[ignore]
    fn test_vrf() {
        let json = std::fs::read_to_string("/tmp/vrf.json").unwrap();
        let vrf_evaluator_input: VrfEvaluatorInput = serde_json::from_str(&json).unwrap();

        let private = "SOME_KEY";
        let private = AccountSecretKey::from_str(private).unwrap();
        let keypair: Keypair = private.into();

        let VrfEvaluatorInput {
            epoch_seed,
            delegator_table,
            global_slot,
            total_currency,
            staking_ledger_hash: _,
        } = &vrf_evaluator_input;

        let now = std::time::Instant::now();

        let vrf_result = delegator_table
            .iter()
            .map(|(index, (pub_key, stake))| {
                let vrf_input = VrfEvaluationInput {
                    producer_key: keypair.clone(),
                    global_slot: *global_slot,
                    epoch_seed: epoch_seed.clone(),
                    account_pub_key: pub_key.clone(),
                    delegator_index: *index,
                    delegated_stake: (*stake).into(),
                    total_currency: (*total_currency).into(),
                };
                // let now = redux::Instant::now();
                let vrf_result = vrf::evaluate_vrf(vrf_input).unwrap();
                // let elapsed = now.elapsed();
                // let slot = global_slot;
                // eprintln!("vrf::evaluate_vrf: {elapsed:?} slot:{slot:?} index:{index:?}");
                // openmina_core::info!(openmina_core::log::system_time(); "vrf::evaluate_vrf: {elapsed:?} slot:{slot:?} index:{index:?}");

                // nevaluated.fetch_add(1, std::sync::atomic::Ordering::AcqRel);

                // the first delegate that won the slot
                if let VrfEvaluationOutput::SlotWon(_) = vrf_result {
                    return Some(vrf_result);
                }
                None
            })
            .collect::<Vec<_>>();

        let elapsed = now.elapsed();
        let slot = vrf_evaluator_input.global_slot;
        let ndelegator = vrf_evaluator_input.delegator_table.len();
        // let nevaluated = nevaluated.load(std::sync::atomic::Ordering::Relaxed);
        eprintln!("TOTAL vrf::evaluate_vrf: {elapsed:?} slot:{slot:?} ndelegators:{ndelegator:?}");
        dbg!(vrf_result);
    }
}
