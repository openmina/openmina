use std::collections::BTreeMap;

use openmina_core::snark::{SnarkJobCommitment, SnarkJobId};
use p2p::channels::{
    snark::P2pChannelsSnarkAction, snark_job_commitment::P2pChannelsSnarkJobCommitmentAction,
};

use crate::{snark_pool::JobCommitment, ExternalSnarkWorkerAction, SnarkerStrategy};

use super::{
    JobState, SnarkPoolAction, SnarkPoolActionWithMetaRef, SnarkPoolEffectfulAction,
    SnarkPoolState, SnarkWork,
};

impl SnarkPoolState {
    pub fn reducer(
        mut state_context: crate::Substate<Self>,
        action: SnarkPoolActionWithMetaRef<'_>,
    ) {
        let Ok(state) = state_context.get_substate_mut() else {
            // TODO: log or propagate
            return;
        };
        let (action, meta) = action.split();

        match action {
            SnarkPoolAction::Candidate(action) => {
                super::candidate::SnarkPoolCandidatesState::reducer(
                    crate::Substate::from_compatible_substate(state_context),
                    meta.with_action(action),
                );
            }
            SnarkPoolAction::JobsUpdate {
                jobs,
                orphaned_snarks,
            } => {
                let mut jobs_map = jobs
                    .iter()
                    .enumerate()
                    .map(|(index, job)| (SnarkJobId::from(job), (index, job.clone())))
                    .collect::<BTreeMap<_, _>>();

                state.retain(|id| jobs_map.remove(id).map(|(order, _)| order));
                for (id, (order, job)) in jobs_map {
                    state.insert(JobState {
                        time: meta.time(),
                        id,
                        job,
                        commitment: None,
                        snark: None,
                        order,
                    });
                }

                let orphaned_snarks = orphaned_snarks
                    .iter()
                    .map(|snark| (snark.work.job_id(), snark.clone()));

                for (id, snark) in orphaned_snarks {
                    let take = state
                        .get(&id)
                        .and_then(|job| job.snark.as_ref())
                        .map_or(true, |old_snark| snark.work > old_snark.work);
                    if take {
                        if let Some(mut job) = state.remove(&id) {
                            job.snark = Some(snark.clone());
                            state.insert(job);
                        }
                    }
                }

                state.candidates_prune();

                // Dispatch
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                if let Some(job_id) = global_state.external_snark_worker.working_job_id() {
                    if !global_state.snark_pool.contains(job_id) {
                        // job is no longer needed.
                        dispatcher.push(ExternalSnarkWorkerAction::CancelWork);
                    }
                } else {
                    dispatcher.push(SnarkPoolAction::AutoCreateCommitment);
                }
            }
            SnarkPoolAction::AutoCreateCommitment => {
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                let Some(snarker_config) = &global_state.config.snarker else {
                    return;
                };
                let available_workers = global_state.external_snark_worker.available();

                if available_workers > 0 {
                    let jobs = global_state
                        .snark_pool
                        .available_jobs_with_highest_priority(available_workers);
                    match snarker_config.strategy {
                        SnarkerStrategy::Sequential => {
                            let job_ids = jobs
                                .into_iter()
                                .map(|job| job.id.clone())
                                .take(available_workers) // just in case
                                .collect();
                            dispatcher.push(SnarkPoolAction::CommitmentCreateMany { job_ids });
                        }
                        SnarkerStrategy::Random => {
                            let jobs = global_state.snark_pool.available_jobs_iter();
                            let choices = jobs.map(|job| job.id.clone()).collect();

                            dispatcher.push(SnarkPoolEffectfulAction::SnarkPoolJobsRandomChoose {
                                choices,
                                count: available_workers,
                                on_result: redux::callback!(
                                    on_snark_pool_jobs_random_choose_result(job_ids: Vec<SnarkJobId>) -> crate::Action {
                                        SnarkPoolAction::CommitmentCreateMany { job_ids }
                                    }
                                ),
                            });
                        }
                    }
                };
            }
            SnarkPoolAction::CommitmentCreateMany { job_ids } => {
                let dispatcher = state_context.into_dispatcher();
                for job_id in job_ids.iter().cloned() {
                    dispatcher.push(SnarkPoolAction::CommitmentCreate { job_id });
                }
            }
            SnarkPoolAction::CommitmentCreate { job_id } => {
                let job_id = job_id.clone();
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                let Some(summary) = global_state.snark_pool.job_summary(&job_id) else {
                    return;
                };

                if global_state.external_snark_worker.is_idle() {
                    dispatcher.push(ExternalSnarkWorkerAction::SubmitWork {
                        job_id: job_id.clone(),
                        summary,
                    });

                    let timestamp_ms = meta.time_as_nanos() / 1_000_000;
                    let Some(config) = global_state.config.snarker.as_ref() else {
                        return;
                    };
                    dispatcher.push(SnarkPoolAction::CommitmentAdd {
                        commitment: SnarkJobCommitment::new(
                            timestamp_ms,
                            job_id,
                            config.fee.clone(),
                            config.public_key.clone().into(),
                        ),
                        sender: global_state.p2p.my_id(),
                    });
                }
            }
            SnarkPoolAction::CommitmentAdd { commitment, sender } => {
                let Some(mut job) = state.remove(&commitment.job_id) else {
                    return;
                };
                job.commitment = Some(JobCommitment {
                    commitment: commitment.clone(),
                    received_t: meta.time(),
                    sender: *sender,
                });
                state.insert(job);

                // Dispatch
                let commitment = commitment.clone();
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                if let Some(job_id) = global_state.external_snark_worker.working_job_id() {
                    let Some(config) = global_state.config.snarker.as_ref() else {
                        return;
                    };
                    if &commitment.job_id == job_id
                        && &commitment.snarker != config.public_key.as_ref()
                    {
                        dispatcher.push(ExternalSnarkWorkerAction::CancelWork);
                    }
                }
            }
            SnarkPoolAction::WorkAdd { snark, sender } => {
                let job_id = snark.job_id();
                let Some(mut job) = state.remove(&job_id) else {
                    return;
                };
                job.snark = Some(SnarkWork {
                    work: snark.clone(),
                    received_t: meta.time(),
                    sender: *sender,
                });
                state.insert(job);
                state.candidates.remove_inferior_snarks(snark);

                // Dispatch
                let snark = snark.clone();
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                if let Some(job_id) = global_state
                    .external_snark_worker
                    .working_job_id()
                    .filter(|job_id| *job_id == &snark.job_id())
                {
                    if let Some(commitment) = global_state
                        .snark_pool
                        .get(job_id)
                        .and_then(|job| job.commitment.as_ref())
                    {
                        if snark > commitment.commitment {
                            dispatcher.push(ExternalSnarkWorkerAction::CancelWork);
                        }
                    }
                }

                dispatcher.push(P2pChannelsSnarkAction::Libp2pBroadcast {
                    snark: snark.clone(),
                    nonce: 0,
                });
            }
            SnarkPoolAction::P2pSendAll { .. } => {
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                for peer_id in global_state.p2p.ready_peers() {
                    dispatcher.push(SnarkPoolAction::P2pSend { peer_id });
                }
            }
            SnarkPoolAction::P2pSend { peer_id } => {
                let peer_id = *peer_id;
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                let Some(peer) = global_state.p2p.get_ready_peer(&peer_id) else {
                    return;
                };

                // Send commitments.
                let index_and_limit = peer
                    .channels
                    .snark_job_commitment
                    .next_send_index_and_limit();
                let (commitments, first_index, last_index) =
                    data_to_send(global_state, index_and_limit, |job| {
                        job.commitment_msg().cloned()
                    });

                let send_commitments = P2pChannelsSnarkJobCommitmentAction::ResponseSend {
                    peer_id,
                    commitments,
                    first_index,
                    last_index,
                };

                // Send snarks.
                let index_and_limit = peer.channels.snark.next_send_index_and_limit();
                let (snarks, first_index, last_index) =
                    data_to_send(global_state, index_and_limit, |job| job.snark_msg());

                dispatcher.push(send_commitments);
                dispatcher.push(P2pChannelsSnarkAction::ResponseSend {
                    peer_id,
                    snarks,
                    first_index,
                    last_index,
                });
            }
            SnarkPoolAction::CheckTimeouts => {
                state.last_check_timeouts = meta.time();

                // Dispatch
                let (dispatcher, global_state) = state_context.into_dispatcher_and_state();
                let timed_out_ids = global_state
                    .snark_pool
                    .timed_out_commitments_iter(meta.time())
                    .cloned()
                    .collect::<Vec<_>>();
                for job_id in timed_out_ids {
                    dispatcher.push(SnarkPoolAction::JobCommitmentTimeout { job_id });
                }
            }
            SnarkPoolAction::JobCommitmentTimeout { job_id } => {
                state.remove_commitment(job_id);

                // Dispatch
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(SnarkPoolAction::AutoCreateCommitment);
            }
        }
    }
}

pub fn data_to_send<F, T>(
    state: &crate::State,
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
