use openmina_core::snark::SnarkJobCommitment;

use crate::p2p::channels::snark::{
    P2pChannelsSnarkLibp2pBroadcastAction, P2pChannelsSnarkResponseSendAction,
};
use crate::p2p::channels::snark_job_commitment::P2pChannelsSnarkJobCommitmentResponseSendAction;
use crate::{ExternalSnarkWorkerAction, Service, SnarkerStrategy, State, Store};

use super::candidate::snark_pool_candidate_effects;
use super::{
    JobState, SnarkPoolAction, SnarkPoolActionWithMeta, SnarkPoolAutoCreateCommitmentAction,
    SnarkPoolCommitmentCreateAction, SnarkPoolJobCommitmentAddAction,
    SnarkPoolJobCommitmentTimeoutAction, SnarkPoolP2pSendAction,
};

pub fn snark_pool_effects<S: Service>(store: &mut Store<S>, action: SnarkPoolActionWithMeta) {
    let (action, meta) = action.split();

    match action {
        SnarkPoolAction::Candidate(action) => {
            snark_pool_candidate_effects(store, meta.with_action(action))
        }
        SnarkPoolAction::JobsUpdate(_) => {
            let state = store.state();
            if let Some(job_id) = state.external_snark_worker.working_job_id() {
                if !state.snark_pool.contains(job_id) {
                    // job is no longer needed.
                    store.dispatch(ExternalSnarkWorkerAction::CancelWork);
                }
            } else {
                store.dispatch(SnarkPoolAutoCreateCommitmentAction {});
            }
        }
        SnarkPoolAction::AutoCreateCommitment(_) => {
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
                    store.dispatch(SnarkPoolCommitmentCreateAction { job_id });
                }
            }
        }
        SnarkPoolAction::CommitmentCreate(a) => {
            let Some(summary) = store.state().snark_pool.job_summary(&a.job_id) else {
                return;
            };
            if store.dispatch(ExternalSnarkWorkerAction::SubmitWork {
                job_id: a.job_id.clone(),
                summary,
            }) {
                let timestamp_ms = meta.time_as_nanos() / 1_000_000;
                let Some(config) = store.state.get().config.snarker.as_ref() else {
                    return;
                };
                store.dispatch(SnarkPoolJobCommitmentAddAction {
                    commitment: SnarkJobCommitment::new(
                        timestamp_ms,
                        a.job_id,
                        config.fee.clone(),
                        config.public_key.clone().into(),
                    ),
                    sender: store.state().p2p.my_id(),
                });
            }
        }
        SnarkPoolAction::CommitmentAdd(a) => {
            let state = store.state();
            if let Some(job_id) = state.external_snark_worker.working_job_id() {
                let Some(config) = store.state.get().config.snarker.as_ref() else {
                    return;
                };
                if &a.commitment.job_id == job_id
                    && &a.commitment.snarker != config.public_key.as_ref()
                {
                    store.dispatch(ExternalSnarkWorkerAction::CancelWork);
                }
            }
        }
        SnarkPoolAction::WorkAdd(a) => {
            let state = store.state();
            if let Some(job_id) = state
                .external_snark_worker
                .working_job_id()
                .filter(|job_id| *job_id == &a.snark.job_id())
            {
                if let Some(commitment) = state
                    .snark_pool
                    .get(job_id)
                    .and_then(|job| job.commitment.as_ref())
                {
                    if a.snark > commitment.commitment {
                        store.dispatch(ExternalSnarkWorkerAction::CancelWork);
                    }
                }
            }

            store.dispatch(P2pChannelsSnarkLibp2pBroadcastAction {
                snark: a.snark,
                nonce: 0,
            });
        }
        SnarkPoolAction::P2pSendAll(_) => {
            for peer_id in store.state().p2p.ready_peers() {
                store.dispatch(SnarkPoolP2pSendAction { peer_id });
            }
        }
        SnarkPoolAction::P2pSend(a) => {
            let state = store.state();
            let Some(peer) = state.p2p.get_ready_peer(&a.peer_id) else {
                return;
            };

            // Send commitments.
            let index_and_limit = peer
                .channels
                .snark_job_commitment
                .next_send_index_and_limit();
            let (commitments, first_index, last_index) =
                data_to_send(state, index_and_limit, |job| job.commitment_msg().cloned());

            let send_commitments = P2pChannelsSnarkJobCommitmentResponseSendAction {
                peer_id: a.peer_id,
                commitments,
                first_index,
                last_index,
            };

            // Send snarks.
            let index_and_limit = peer.channels.snark.next_send_index_and_limit();
            let (snarks, first_index, last_index) =
                data_to_send(state, index_and_limit, |job| job.snark_msg());

            store.dispatch(send_commitments);
            store.dispatch(P2pChannelsSnarkResponseSendAction {
                peer_id: a.peer_id,
                snarks,
                first_index,
                last_index,
            });
        }
        SnarkPoolAction::CheckTimeouts(_) => {
            let timed_out_ids = store
                .state()
                .snark_pool
                .timed_out_commitments_iter(meta.time())
                .cloned()
                .collect::<Vec<_>>();
            for job_id in timed_out_ids {
                store.dispatch(SnarkPoolJobCommitmentTimeoutAction { job_id });
            }
        }
        SnarkPoolAction::JobCommitmentTimeout(_) => {
            store.dispatch(SnarkPoolAutoCreateCommitmentAction {});
        }
    }
}

pub fn data_to_send<F, T>(
    state: &State,
    (index, limit): (u64, u8),
    get_data: F,
) -> (Vec<T>, u64, u64)
where
    F: Fn(&JobState) -> Option<T>,
{
    if limit == 0 {
        let index = index.saturating_sub(1);
        return (vec![], index, index);
    }

    state
        .snark_pool
        .range(index..)
        .try_fold(
            (vec![], None),
            |(mut list, mut first_index), (index, job)| {
                if let Some(data) = get_data(job) {
                    let first_index = *first_index.get_or_insert(index);
                    list.push(data);
                    if list.len() >= limit as usize {
                        return Err((list, first_index, index));
                    }
                }

                Ok((list, first_index))
            },
        )
        // Loop iterated on whole snark pool.
        .map(|(list, first_index)| {
            let snark_pool_last_index = state.snark_pool.last_index();
            (list, first_index.unwrap_or(index), snark_pool_last_index)
        })
        // Loop preemptively ended.
        .unwrap_or_else(|v| v)
}
