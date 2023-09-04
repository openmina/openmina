use openmina_core::snark::Snark;

use crate::{
    external_snark_worker::ExternalSnarkWorkerPruneWorkAction,
    snark_pool::{SnarkPoolAutoCreateCommitmentAction, SnarkPoolWorkAddAction},
};

use super::{
    available_job_to_snark_worker_spec, ExternalSnarkWorkerAction,
    ExternalSnarkWorkerActionWithMeta, ExternalSnarkWorkerCancelWorkAction,
    ExternalSnarkWorkerErrorAction, ExternalSnarkWorkerKillAction,
    ExternalSnarkWorkerWorkErrorAction,
};

pub fn external_snark_worker_effects<S: crate::Service>(
    store: &mut crate::Store<S>,
    action: ExternalSnarkWorkerActionWithMeta,
) {
    let (action, _) = action.split();
    match action {
        ExternalSnarkWorkerAction::Start(_) => {
            let config = &store.state().config;
            let Some(path) = config.path.as_ref().cloned() else {
                return;
            };
            let public_key = config.public_key.clone().into();
            let fee = config.fee.clone();
            if let Err(err) = store.service().start(path, public_key, fee) {
                store.dispatch(ExternalSnarkWorkerErrorAction {
                    error: err,
                    permanent: true,
                });
            }
        }
        ExternalSnarkWorkerAction::Started(_) => {
            store.dispatch(SnarkPoolAutoCreateCommitmentAction {});
        }
        ExternalSnarkWorkerAction::StartTimeout(_) => {
            store.dispatch(ExternalSnarkWorkerErrorAction {
                error: super::ExternalSnarkWorkerError::StartTimeout,
                permanent: true,
            });
        }
        ExternalSnarkWorkerAction::Kill(_) => {
            if let Err(err) = store.service().kill() {
                store.dispatch(ExternalSnarkWorkerErrorAction {
                    error: err,
                    permanent: true,
                });
            }
        }
        ExternalSnarkWorkerAction::Killed(_) => {}
        ExternalSnarkWorkerAction::Error(_action) => {
            store.dispatch(ExternalSnarkWorkerKillAction {});
        }
        ExternalSnarkWorkerAction::SubmitWork(action) => {
            let job_id = &action.job_id;
            let Some(job) = store.state().snark_pool.get(job_id) else {
                return;
            };
            let input = match available_job_to_snark_worker_spec(
                job.job.clone(),
                &store.state().transition_frontier,
            ) {
                Ok(v) => v,
                Err(err) => {
                    store.dispatch(ExternalSnarkWorkerWorkErrorAction { error: err.into() });
                    return;
                }
            };
            if let Err(err) = store.service().submit(input) {
                store.dispatch(ExternalSnarkWorkerWorkErrorAction { error: err.into() });
                return;
            }
        }
        ExternalSnarkWorkerAction::WorkResult(action) => {
            let config = &store.state().config;
            let snarker = config.public_key.clone().into();
            let fee = config.fee.clone();
            let snark = Snark {
                snarker,
                fee,
                proofs: action.result.clone(),
            };
            let sender = store.state().p2p.config.identity_pub_key.peer_id();
            // Directly add snark to the snark pool as it's produced by us.
            store.dispatch(SnarkPoolWorkAddAction { snark, sender });
            store.dispatch(ExternalSnarkWorkerPruneWorkAction {});
        }
        ExternalSnarkWorkerAction::WorkError(_) => {
            store.dispatch(ExternalSnarkWorkerPruneWorkAction {});
        }
        ExternalSnarkWorkerAction::WorkTimeout(_) => {
            store.dispatch(ExternalSnarkWorkerCancelWorkAction {});
        }
        ExternalSnarkWorkerAction::CancelWork(_) => {
            if let Err(err) = store.service().cancel() {
                store.dispatch(ExternalSnarkWorkerErrorAction {
                    error: err.into(),
                    permanent: true,
                });
                return;
            }
        }
        ExternalSnarkWorkerAction::WorkCancelled(_) => {
            store.dispatch(ExternalSnarkWorkerPruneWorkAction {});
        }
        ExternalSnarkWorkerAction::PruneWork(_) => {
            store.dispatch(SnarkPoolAutoCreateCommitmentAction {});
        }
    }
}
