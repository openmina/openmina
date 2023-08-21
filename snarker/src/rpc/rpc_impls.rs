use crate::external_snark_worker::ExternalSnarkWorkerState;

use super::{RpcSnarkWorker, RpcSnarkWorkerStatus};

impl From<ExternalSnarkWorkerState> for RpcSnarkWorker {
    fn from(source: ExternalSnarkWorkerState) -> Self {
        Self {
            time: None,
            id: Some("single".into()),
            status: source.into(),
        }
    }
}

impl From<ExternalSnarkWorkerState> for RpcSnarkWorkerStatus {
    fn from(source: ExternalSnarkWorkerState) -> Self {
        match source {
            ExternalSnarkWorkerState::None => RpcSnarkWorkerStatus::None,
            ExternalSnarkWorkerState::Starting => RpcSnarkWorkerStatus::Starting,
            ExternalSnarkWorkerState::Idle => RpcSnarkWorkerStatus::Idle,
            ExternalSnarkWorkerState::Working(job_id) => RpcSnarkWorkerStatus::Working { job_id },
            ExternalSnarkWorkerState::WorkReady(job_id, _) => {
                RpcSnarkWorkerStatus::WorkReady { job_id }
            }
            ExternalSnarkWorkerState::WorkError(job_id, error) => {
                RpcSnarkWorkerStatus::WorkError { job_id, error }
            }
            ExternalSnarkWorkerState::Cancelling(job_id) => {
                RpcSnarkWorkerStatus::Cancelling { job_id }
            }
            ExternalSnarkWorkerState::Cancelled(job_id) => {
                RpcSnarkWorkerStatus::Cancelled { job_id }
            }
            ExternalSnarkWorkerState::Killing => RpcSnarkWorkerStatus::Killing,
            ExternalSnarkWorkerState::Error(error, permanent) => RpcSnarkWorkerStatus::Error { error, permanent },
        }
    }
}
