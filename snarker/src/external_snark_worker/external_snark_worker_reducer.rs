use super::{
    ExternalSnarkWorkerAction, ExternalSnarkWorkerActionWithMetaRef, ExternalSnarkWorkerState,
};

impl ExternalSnarkWorkerState {
    pub fn reducer(&mut self, action: ExternalSnarkWorkerActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            ExternalSnarkWorkerAction::Start(_) => {
                *self = ExternalSnarkWorkerState::Starting;
            },
            ExternalSnarkWorkerAction::Started(_) => {
                *self = ExternalSnarkWorkerState::Idle;
            }
            ExternalSnarkWorkerAction::Kill(_) => {
                *self = ExternalSnarkWorkerState::Killing;
            },
            ExternalSnarkWorkerAction::Killed(_) => {
                *self = ExternalSnarkWorkerState::None;
            },
            ExternalSnarkWorkerAction::Error(a) => {
                *self = ExternalSnarkWorkerState::Error(a.error.clone());
            },
            ExternalSnarkWorkerAction::SubmitWork(action) => {
                *self = ExternalSnarkWorkerState::Working(action.job_id.clone());
            },
            ExternalSnarkWorkerAction::WorkResult(action) => {
                let ExternalSnarkWorkerState::Working(job_id) = self else {
                    return;
                };
                *self = ExternalSnarkWorkerState::WorkReady(job_id.clone(), action.result.clone());
            },
            ExternalSnarkWorkerAction::WorkError(action) => {
                let ExternalSnarkWorkerState::Working(job_id) = self else {
                    return;
                };
                *self = ExternalSnarkWorkerState::WorkError(job_id.clone(), action.error.clone());
            },
            ExternalSnarkWorkerAction::PruneWork(_) => {
                *self = ExternalSnarkWorkerState::Idle;
            }
        }
    }
}
