use mina_signer::Keypair;
use node::{
    block_producer::{
        vrf_evaluator::{VrfEvaluationOutputWithHash, VrfEvaluatorInput},
        BlockProducerEvent,
    },
    event_source::Event,
};
use openmina_core::channels::mpsc::{UnboundedReceiver, UnboundedSender};
use vrf::{VrfEvaluationInput, VrfEvaluationOutput};

use crate::NodeService;
use node::block_producer::BlockProducerVrfEvaluatorEvent;

pub fn vrf_evaluator(
    event_sender: UnboundedSender<Event>,
    mut vrf_evaluation_receiver: UnboundedReceiver<VrfEvaluatorInput>,
    keypair: Keypair,
) {
    while let Some(vrf_evaluator_input) = vrf_evaluation_receiver.blocking_recv() {
        let mut vrf_result = VrfEvaluationOutput::SlotLost(vrf_evaluator_input.global_slot);

        for (index, account) in vrf_evaluator_input.delegator_table.iter() {
            let vrf_input = VrfEvaluationInput::new(
                keypair.clone(),
                vrf_evaluator_input.epoch_seed.clone(),
                account.0.to_string(),
                vrf_evaluator_input.global_slot,
                index.clone(),
                account.1.into(),
                vrf_evaluator_input.total_currency.into(),
            );
            vrf_result = vrf::evaluate_vrf(vrf_input).unwrap();

            // the first delegate that won the slot
            if let VrfEvaluationOutput::SlotWon(_) = vrf_result {
                break;
            }
        }
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

impl node::block_producer::vrf_evaluator::BlockProducerVrfEvaluatorService for NodeService {
    fn evaluate(&mut self, data: VrfEvaluatorInput) {
        if let Some(bp) = self.block_producer.as_mut() {
            let _ = bp.vrf_evaluation_sender.send(data);
        }
    }
}
