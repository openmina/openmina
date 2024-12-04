use mina_signer::Keypair;
use node::{
    account::AccountPublicKey,
    block_producer::{
        vrf_evaluator::{VrfEvaluationOutputWithHash, VrfEvaluatorInput},
        BlockProducerEvent, BlockProducerVrfEvaluatorEvent,
    },
    core::channels::mpsc::{UnboundedReceiver, UnboundedSender},
    event_source::Event,
};
use vrf::{VrfEvaluationInput, VrfEvaluationOutput};

use crate::NodeService;

use super::VrfEvaluatorBatch;

pub fn vrf_evaluator(
    event_sender: UnboundedSender<Event>,
    mut vrf_evaluation_receiver: UnboundedReceiver<VrfEvaluatorBatch>,
    keypair: Keypair,
) {
    while let Some(vrf_evaluator_batch) = vrf_evaluation_receiver.blocking_recv() {
        let VrfEvaluatorBatch {
            vrf_evaluator_input,
            start_slot,
            batch_size,
        } = vrf_evaluator_batch;

        let (dummy_account_pub_key, dummy_index, dummy_stake) =
            if let Some((index, (pubkey, stake))) =
                vrf_evaluator_input.delegator_table.first_key_value()
            {
                (pubkey.clone(), *index, *stake)
            } else {
                continue;
            };

        // NOTE: to avoid cloning this value is reused, with the following fields updated
        let mut vrf_input = VrfEvaluationInput::new(
            keypair.clone(),                           // Constant
            vrf_evaluator_input.epoch_seed.clone(),    // Constant
            dummy_account_pub_key,                     // Updated
            vrf_evaluator_input.global_slot,           // Updated
            dummy_index,                               // Updated
            dummy_stake.into(),                        // Updated
            vrf_evaluator_input.total_currency.into(), // Constant
        );

        let mut vrf_results = vec![]; // FIXME: capacity

        for global_slot in start_slot..(start_slot.saturating_add(batch_size)) {
            let mut vrf_result = VrfEvaluationOutput::SlotLost(vrf_evaluator_input.global_slot);

            for (index, (pub_key, stake)) in vrf_evaluator_input.delegator_table.iter() {
                // For each iteration thes values change
                vrf_input.global_slot = global_slot;
                vrf_input.account_pub_key = pub_key.clone();
                vrf_input.delegator_index = *index;
                vrf_input.delegated_stake = (*stake).into();

                vrf_result = vrf::evaluate_vrf(&vrf_input).unwrap();

                // the first delegate that won the slot
                if let VrfEvaluationOutput::SlotWon(_) = vrf_result {
                    break;
                }
            }

            vrf_results.push(vrf_result);
        }

        let vrf_result_with_hash = VrfEvaluationOutputWithHash::new(
            vrf_results,
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
    fn evaluate(
        &mut self,
        vrf_evaluator_input: VrfEvaluatorInput,
        start_slot: u32,
        batch_size: u32,
    ) {
        if let Some(bp) = self.block_producer.as_mut() {
            let _ = bp.vrf_evaluation_sender.send(VrfEvaluatorBatch {
                vrf_evaluator_input,
                start_slot,
                batch_size,
            });
        }
    }
}
