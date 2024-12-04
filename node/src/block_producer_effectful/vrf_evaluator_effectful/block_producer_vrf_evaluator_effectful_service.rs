use crate::block_producer::vrf_evaluator::VrfEvaluatorInput;

pub trait BlockProducerVrfEvaluatorService: redux::Service {
    fn evaluate(&mut self, data: VrfEvaluatorInput, start_slot: u32, batch_size: u32);
}
