use crate::block_producer::vrf_evaluator::VrfEvaluatorInput;

pub trait BlockProducerVrfEvaluatorService: redux::Service {
    fn evaluate(&mut self, data: VrfEvaluatorInput);
}
