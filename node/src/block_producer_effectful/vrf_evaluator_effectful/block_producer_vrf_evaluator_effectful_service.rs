//! Defines the service interface for VRF evaluation in block production.
//! This service is responsible for evaluating slots to determine block production eligibility.

use crate::block_producer::vrf_evaluator::VrfEvaluatorInput;

/// Service interface for VRF evaluation operations.
/// Provides methods for evaluating slots using VRF to determine if the node can produce a block.
pub trait BlockProducerVrfEvaluatorService: redux::Service {
    fn evaluate(&mut self, data: VrfEvaluatorInput);
}
