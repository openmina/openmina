use super::{
    external_snark_worker_state::{ExternalSnarkWorker, ExternalSnarkWorkerState},
    ExternalSnarkWorkerAction, ExternalSnarkWorkerActionWithMetaRef, ExternalSnarkWorkers,
};

impl ExternalSnarkWorkers {
    pub fn reducer(&mut self, action: ExternalSnarkWorkerActionWithMetaRef<'_>) {
        self.0.reducer(action)
    }
}

impl ExternalSnarkWorker {
    pub fn reducer(&mut self, action: ExternalSnarkWorkerActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            ExternalSnarkWorkerAction::Start => {
                self.state = ExternalSnarkWorkerState::Starting;
            }
            ExternalSnarkWorkerAction::Started => {
                self.state = ExternalSnarkWorkerState::Idle;
            }
            ExternalSnarkWorkerAction::StartTimeout { .. } => {
                return;
            }
            ExternalSnarkWorkerAction::Kill => {
                self.state = ExternalSnarkWorkerState::Killing;
            }
            ExternalSnarkWorkerAction::Killed => {
                self.state = ExternalSnarkWorkerState::None;
            }
            ExternalSnarkWorkerAction::Error { error, permanent } => {
                self.state = ExternalSnarkWorkerState::Error(error.clone(), *permanent);
            }
            ExternalSnarkWorkerAction::SubmitWork { job_id, summary } => {
                self.state = ExternalSnarkWorkerState::Working(job_id.clone(), summary.clone());
            }
            ExternalSnarkWorkerAction::WorkResult { result } => {
                let ExternalSnarkWorkerState::Working(job_id, _) = &self.state else {
                    return;
                };
                self.state = ExternalSnarkWorkerState::WorkReady(job_id.clone(), result.clone());
            }
            ExternalSnarkWorkerAction::WorkError { error } => {
                let ExternalSnarkWorkerState::Working(job_id, _) = &self.state else {
                    return;
                };
                self.state = ExternalSnarkWorkerState::WorkError(job_id.clone(), error.clone());
            }
            ExternalSnarkWorkerAction::WorkTimeout { .. } => {
                return;
            }
            ExternalSnarkWorkerAction::CancelWork => {
                let ExternalSnarkWorkerState::Working(job_id, _) = &self.state else {
                    return;
                };
                self.state = ExternalSnarkWorkerState::Cancelling(job_id.clone());
            }
            ExternalSnarkWorkerAction::WorkCancelled => {
                let ExternalSnarkWorkerState::Cancelling(job_id) = &self.state else {
                    return;
                };
                self.state = ExternalSnarkWorkerState::Cancelled(job_id.clone());
            }
            ExternalSnarkWorkerAction::PruneWork => {
                self.state = ExternalSnarkWorkerState::Idle;
            }
        }
        self.timestamp = meta.time();
    }
}
