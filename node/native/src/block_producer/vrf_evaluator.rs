use mina_signer::Keypair;
use node::{block_producer::{BlockProducerEvent, vrf_evaluator::VrfEvaluatorInput}, event_source::Event};
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
        // TODO(adonagy): check correctness of epoch bound calculations
        // const SLOT_PER_EPOCH: u32 = 7140;
        // let epoch_num = vrf_evaluator_input.start_at_slot / SLOT_PER_EPOCH;
        // let end = epoch_num * SLOT_PER_EPOCH + SLOT_PER_EPOCH;

        // println!("[vrf] evaluating: {} - {}", vrf_evaluator_input.start_at_slot, end);

        // let mut batch: Vec<VrfEvaluationOutput> = Vec::new();
        let mut vrf_result = VrfEvaluationOutput::SlotLost(vrf_evaluator_input.global_slot);

        for (index, account) in vrf_evaluator_input.delegatee_table.iter() {
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
        // send the result back to the state machine
        let _ = event_sender.send(
            BlockProducerEvent::VrfEvaluator(BlockProducerVrfEvaluatorEvent::Evaluated(vrf_result))
                .into(),
        );
    }
}

impl node::block_producer::vrf_evaluator::BlockProducerVrfEvaluatorService for NodeService {
    fn evaluate(&mut self, data: VrfEvaluatorInput) {
        if let Some(bp) = self.block_producer.as_mut() {
            // TODO(adonagy): send the data to the vrf_evaluator thread
            let _ = bp.vrf_evaluation_sender.send(data);
        }
    }
}
