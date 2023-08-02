use serde::{Serialize, Deserialize};

use super::{SnarkWorkId, ExternalSnarkWorkerError, SnarkWorkResult, ExternalSnarkWorkerWorkError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExternalSnarkWorkerState {
    None,
    Starting,

    Idle,
    Working(SnarkWorkId),
    WorkReady(SnarkWorkId, SnarkWorkResult),
    WorkError(SnarkWorkId, ExternalSnarkWorkerWorkError),

    Killing,
    Error(ExternalSnarkWorkerError),
}

impl ExternalSnarkWorkerState {
    pub fn new() -> Self {
        ExternalSnarkWorkerState::None
    }

    pub fn is_idle(&self) -> bool {
        match self {
            ExternalSnarkWorkerState::Idle => true,
            _ => false,
        }
    }
}
