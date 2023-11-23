pub trait BlockProducerVrfEvaluatorService: redux::Service {
    fn evaluate(&mut self, data: ());
}
