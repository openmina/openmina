use openmina_core::snark::Snark;
use redux::Timestamp;

use super::{
    available_job_to_snark_worker_spec,
    external_snark_worker_state::{ExternalSnarkWorker, ExternalSnarkWorkerState},
    ExternalSnarkWorkerAction, ExternalSnarkWorkerActionWithMetaRef, ExternalSnarkWorkerWorkError,
    ExternalSnarkWorkers,
};
use crate::{
    external_snark_worker_effectful::ExternalSnarkWorkerEffectfulAction, p2p_ready,
    SnarkPoolAction, Substate,
};

impl ExternalSnarkWorkers {
    pub fn reducer(
        state_context: Substate<ExternalSnarkWorkers>,
        action: ExternalSnarkWorkerActionWithMetaRef<'_>,
    ) {
        ExternalSnarkWorker::reducer(Substate::from_compatible_substate(state_context), action);
    }
}

impl ExternalSnarkWorker {
    pub fn reducer(
        mut state_context: Substate<ExternalSnarkWorker>,
        action: ExternalSnarkWorkerActionWithMetaRef<'_>,
    ) {
        let Ok(worker_state) = state_context.get_substate_mut() else {
            return;
        };
        let (action, meta) = action.split();
        match action {
            ExternalSnarkWorkerAction::Start => {
                worker_state.state = ExternalSnarkWorkerState::Starting;
                worker_state.update_timestamp(meta.time());

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let Some(config) = &state.config.snarker else {
                    return;
                };

                let public_key = config.public_key.clone().into();
                let fee = config.fee.clone();

                dispatcher.push(ExternalSnarkWorkerEffectfulAction::Start { public_key, fee });
            }
            ExternalSnarkWorkerAction::Started => {
                worker_state.state = ExternalSnarkWorkerState::Idle;
                worker_state.update_timestamp(meta.time());

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(SnarkPoolAction::AutoCreateCommitment);
            }
            ExternalSnarkWorkerAction::StartTimeout { .. } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(ExternalSnarkWorkerAction::Error {
                    error: super::ExternalSnarkWorkerError::StartTimeout,
                    permanent: true,
                });
            }
            ExternalSnarkWorkerAction::Kill => {
                worker_state.state = ExternalSnarkWorkerState::Killing;
                worker_state.update_timestamp(meta.time());

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(ExternalSnarkWorkerEffectfulAction::Kill);
            }
            ExternalSnarkWorkerAction::Killed => {
                worker_state.state = ExternalSnarkWorkerState::None;
                worker_state.update_timestamp(meta.time());
            }
            ExternalSnarkWorkerAction::Error { error, permanent } => {
                worker_state.state = ExternalSnarkWorkerState::Error(error.clone(), *permanent);
                worker_state.update_timestamp(meta.time());

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(ExternalSnarkWorkerAction::Kill);
            }
            ExternalSnarkWorkerAction::SubmitWork { job_id, summary } => {
                worker_state.state =
                    ExternalSnarkWorkerState::Working(job_id.clone(), summary.clone());
                worker_state.update_timestamp(meta.time());

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let Some(job) = state.snark_pool.get(job_id) else {
                    return;
                };

                match available_job_to_snark_worker_spec(
                    job.job.clone(),
                    &state.transition_frontier,
                ) {
                    Ok(spec) => {
                        dispatcher.push(ExternalSnarkWorkerEffectfulAction::SubmitWork {
                            spec: Box::new(spec),
                        });
                    }
                    Err(err) => {
                        dispatcher.push(ExternalSnarkWorkerAction::WorkError {
                            error: ExternalSnarkWorkerWorkError::WorkSpecError(err),
                        });
                    }
                }
            }
            ExternalSnarkWorkerAction::WorkResult { result } => {
                let ExternalSnarkWorkerState::Working(job_id, _) = &worker_state.state else {
                    return;
                };
                worker_state.state =
                    ExternalSnarkWorkerState::WorkReady(job_id.clone(), result.clone());
                worker_state.update_timestamp(meta.time());

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let Some(config) = &state.config.snarker else {
                    return;
                };
                let p2p = p2p_ready!(state.p2p, meta.time());
                let snarker = config.public_key.clone().into();
                let fee = config.fee.clone();
                let snark = Snark {
                    snarker,
                    fee,
                    proofs: result.clone(),
                };
                let sender = p2p.my_id();
                // Directly add snark to the snark pool as it's produced by us.
                dispatcher.push(SnarkPoolAction::WorkAdd { snark, sender });
                dispatcher.push(ExternalSnarkWorkerAction::PruneWork);
            }
            ExternalSnarkWorkerAction::WorkError { error } => {
                let ExternalSnarkWorkerState::Working(job_id, _) = &worker_state.state else {
                    return;
                };
                worker_state.state =
                    ExternalSnarkWorkerState::WorkError(job_id.clone(), error.clone());
                worker_state.update_timestamp(meta.time());

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(ExternalSnarkWorkerAction::PruneWork);
            }
            ExternalSnarkWorkerAction::WorkTimeout { .. } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(ExternalSnarkWorkerAction::CancelWork);
            }
            ExternalSnarkWorkerAction::CancelWork => {
                let ExternalSnarkWorkerState::Working(job_id, _) = &worker_state.state else {
                    return;
                };
                worker_state.state = ExternalSnarkWorkerState::Cancelling(job_id.clone());
                worker_state.update_timestamp(meta.time());

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(ExternalSnarkWorkerEffectfulAction::CancelWork);
            }
            ExternalSnarkWorkerAction::WorkCancelled => {
                let ExternalSnarkWorkerState::Cancelling(job_id) = &worker_state.state else {
                    return;
                };
                worker_state.state = ExternalSnarkWorkerState::Cancelled(job_id.clone());
                worker_state.update_timestamp(meta.time());

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(ExternalSnarkWorkerAction::PruneWork);
            }
            ExternalSnarkWorkerAction::PruneWork => {
                worker_state.state = ExternalSnarkWorkerState::Idle;
                worker_state.update_timestamp(meta.time());

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(SnarkPoolAction::AutoCreateCommitment);
            }
        }
    }

    fn update_timestamp(&mut self, time: Timestamp) {
        self.timestamp = time;
    }
}
