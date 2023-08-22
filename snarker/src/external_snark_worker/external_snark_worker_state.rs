use std::time::Duration;

use redux::Timestamp;
use serde::{Deserialize, Serialize};

use crate::snark_pool::JobSummary;

use super::{ExternalSnarkWorkerError, ExternalSnarkWorkerWorkError, SnarkWorkId, SnarkWorkResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSnarkWorkers(pub(crate) ExternalSnarkWorker);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSnarkWorker {
    pub(crate) state: ExternalSnarkWorkerState,
    pub(crate) timestamp: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExternalSnarkWorkerState {
    None,
    Starting,

    Idle,
    Working(SnarkWorkId, JobSummary),
    WorkReady(SnarkWorkId, SnarkWorkResult),
    WorkError(SnarkWorkId, ExternalSnarkWorkerWorkError),

    Cancelling(SnarkWorkId),
    Cancelled(SnarkWorkId),

    Killing,
    Error(ExternalSnarkWorkerError, bool),
}

impl ExternalSnarkWorkers {
    pub fn new(now: Timestamp) -> Self {
        ExternalSnarkWorkers(ExternalSnarkWorker { state: ExternalSnarkWorkerState::None, timestamp: now })
    }

    pub fn is_idle(&self) -> bool {
        match self.0.state {
            ExternalSnarkWorkerState::Idle => true,
            _ => false,
        }
    }

    pub fn has_idle(&self) -> bool {
        self.available() > 0
    }

    pub fn available(&self) -> usize {
        if matches!(self.0.state, ExternalSnarkWorkerState::Idle) {
            1
        } else {
            0
        }
    }

    pub fn working_job_id(&self) -> Option<&SnarkWorkId> {
        match &self.0.state {
            ExternalSnarkWorkerState::Working(job_id, _) => Some(job_id),
            _ => None,
        }
    }
}
