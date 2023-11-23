use serde::{Deserialize, Serialize};

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum BlockProducerVrfEvaluatorEvent {
    Evaluated(()),
}

impl std::fmt::Display for BlockProducerVrfEvaluatorEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VrfEvaluator, ")?;
        match self {
            Self::Evaluated(()) => {
                write!(f, "Evaluated, ()")
            }
        }
    }
}
