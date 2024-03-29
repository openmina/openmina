use serde::{Deserialize, Serialize};

use super::VrfEvaluationOutputWithHash;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum BlockProducerVrfEvaluatorEvent {
    Evaluated(VrfEvaluationOutputWithHash),
}

impl std::fmt::Display for BlockProducerVrfEvaluatorEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VrfEvaluator, ")?;
        match self {
            Self::Evaluated(vrf_output) => {
                write!(f, "Evaluated, {}", vrf_output)
            }
        }
    }
}
