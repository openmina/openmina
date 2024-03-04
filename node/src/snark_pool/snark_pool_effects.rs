use crate::{Service, SnarkerStrategy, Store};

use super::{SnarkPoolAction, SnarkPoolActionWithMeta};

// TODO: make an effectful mode to get the list of jobs instead
// and move AutoCreateCommitment to the reducer

pub fn snark_pool_effects<S: Service>(store: &mut Store<S>, action: SnarkPoolActionWithMeta) {
    let (action, _meta) = action.split();

    match action {
        SnarkPoolAction::AutoCreateCommitment { .. } => {
            let state = store.state.get();
            let Some(snarker_config) = &state.config.snarker else {
                return;
            };
            let available_workers = state.external_snark_worker.available();

            if available_workers > 0 {
                let jobs = state
                    .snark_pool
                    .available_jobs_with_highest_priority(available_workers);
                let job_ids: Vec<_> = match snarker_config.strategy {
                    SnarkerStrategy::Sequential => {
                        jobs.into_iter()
                            .map(|job| job.id.clone())
                            .take(available_workers) // just in case
                            .collect()
                    }
                    SnarkerStrategy::Random => {
                        let jobs = state.snark_pool.available_jobs_iter();
                        store.service.random_choose(jobs, available_workers)
                    }
                };

                for job_id in job_ids {
                    store.dispatch(SnarkPoolAction::CommitmentCreate { job_id });
                }
            }
        }
        _ => {
            // Handled by reducer
        }
    }
}
