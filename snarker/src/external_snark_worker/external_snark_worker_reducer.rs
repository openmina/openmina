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
            ExternalSnarkWorkerAction::Start(_) => {
                self.state = ExternalSnarkWorkerState::Starting;
            }
            ExternalSnarkWorkerAction::Started(_) => {
                self.state = ExternalSnarkWorkerState::Idle;
            }
            ExternalSnarkWorkerAction::StartTimeout(_) => {
                return;
            }
            ExternalSnarkWorkerAction::Kill(_) => {
                self.state = ExternalSnarkWorkerState::Killing;
            }
            ExternalSnarkWorkerAction::Killed(_) => {
                self.state = ExternalSnarkWorkerState::None;
            }
            ExternalSnarkWorkerAction::Error(a) => {
                self.state = ExternalSnarkWorkerState::Error(a.error.clone(), a.permanent);
            }
            ExternalSnarkWorkerAction::SubmitWork(action) => {
                self.state = ExternalSnarkWorkerState::Working(action.job_id.clone(), action.summary.clone());
            }
            ExternalSnarkWorkerAction::WorkResult(action) => {
                let ExternalSnarkWorkerState::Working(job_id, _) = &self.state else {
                    return;
                };
                self.state =
                    ExternalSnarkWorkerState::WorkReady(job_id.clone(), action.result.clone());
            }
            ExternalSnarkWorkerAction::WorkError(action) => {
                let ExternalSnarkWorkerState::Working(job_id, _) = &self.state else {
                    return;
                };
                self.state =
                    ExternalSnarkWorkerState::WorkError(job_id.clone(), action.error.clone());
            }
            ExternalSnarkWorkerAction::WorkTimeout(_) => {
                return;
            }
            ExternalSnarkWorkerAction::CancelWork(_) => {
                let ExternalSnarkWorkerState::Working(job_id, _) = &self.state else {
                    return;
                };
                self.state = ExternalSnarkWorkerState::Cancelling(job_id.clone());
            }
            ExternalSnarkWorkerAction::WorkCancelled(_) => {
                let ExternalSnarkWorkerState::Cancelling(job_id) = &self.state else {
                    return;
                };
                self.state = ExternalSnarkWorkerState::Cancelled(job_id.clone());
            }
            ExternalSnarkWorkerAction::PruneWork(_) => {
                self.state = ExternalSnarkWorkerState::Idle;
            }
        }
        self.timestamp = meta.time();
    }
}
