use openmina_core::snark::Snark;

use crate::snark_pool::{SnarkPoolAutoCreateCommitmentAction, SnarkPoolWorkAddAction};

use super::{
    available_job_to_snark_worker_spec, ExternalSnarkWorkerAction,
    ExternalSnarkWorkerActionWithMeta,
};

pub fn external_snark_worker_effects<S: crate::Service>(
    store: &mut crate::Store<S>,
    action: ExternalSnarkWorkerActionWithMeta,
) {
    let (action, _) = action.split();
    match action {
        ExternalSnarkWorkerAction::Start => {
            let Some(config) = &store.state.get().config.snarker else {
                return;
            };
            let public_key = config.public_key.clone().into();
            let fee = config.fee.clone();
            if let Err(err) = store.service.start(&config.path, public_key, fee) {
                store.dispatch(ExternalSnarkWorkerAction::Error {
                    error: err,
                    permanent: true,
                });
            }
        }
        ExternalSnarkWorkerAction::Started => {
            store.dispatch(SnarkPoolAutoCreateCommitmentAction {});
        }
        ExternalSnarkWorkerAction::StartTimeout { .. } => {
            store.dispatch(ExternalSnarkWorkerAction::Error {
                error: super::ExternalSnarkWorkerError::StartTimeout,
                permanent: true,
            });
        }
        ExternalSnarkWorkerAction::Kill => {
            if let Err(err) = store.service().kill() {
                store.dispatch(ExternalSnarkWorkerAction::Error {
                    error: err,
                    permanent: true,
                });
            }
        }
        ExternalSnarkWorkerAction::Killed => {}
        ExternalSnarkWorkerAction::Error { .. } => {
            store.dispatch(ExternalSnarkWorkerAction::Kill);
        }
        ExternalSnarkWorkerAction::SubmitWork { job_id, .. } => {
            let Some(job) = store.state().snark_pool.get(&job_id) else {
                return;
            };
            let input = match available_job_to_snark_worker_spec(
                job.job.clone(),
                &store.state().transition_frontier,
            ) {
                Ok(v) => v,
                Err(err) => {
                    store.dispatch(ExternalSnarkWorkerAction::WorkError { error: err.into() });
                    return;
                }
            };
            if let Err(err) = store.service().submit(input) {
                store.dispatch(ExternalSnarkWorkerAction::WorkError { error: err.into() });
                return;
            }
        }
        ExternalSnarkWorkerAction::WorkResult { result } => {
            let Some(config) = &store.state().config.snarker else {
                return;
            };
            let snarker = config.public_key.clone().into();
            let fee = config.fee.clone();
            let snark = Snark {
                snarker,
                fee,
                proofs: result.clone(),
            };
            let sender = store.state().p2p.my_id();
            store.dispatch(SnarkPoolWorkAddAction { snark, sender });
            store.dispatch(ExternalSnarkWorkerAction::PruneWork);
        }
        ExternalSnarkWorkerAction::WorkError { .. } => {
            store.dispatch(ExternalSnarkWorkerAction::PruneWork);
        }
        ExternalSnarkWorkerAction::WorkTimeout { .. } => {
            store.dispatch(ExternalSnarkWorkerAction::CancelWork);
        }
        ExternalSnarkWorkerAction::CancelWork => {
            if let Err(err) = store.service().cancel() {
                store.dispatch(ExternalSnarkWorkerAction::Error {
                    error: err.into(),
                    permanent: true,
                });
                return;
            }
        }
        ExternalSnarkWorkerAction::WorkCancelled => {
            store.dispatch(ExternalSnarkWorkerAction::PruneWork);
        }
        ExternalSnarkWorkerAction::PruneWork => {
            store.dispatch(SnarkPoolAutoCreateCommitmentAction {});
        }
    }
}
