use std::collections::BTreeMap;

use openmina_core::snark::{SnarkJobCommitment, SnarkJobId};
use p2p::channels::{
    snark::P2pChannelsSnarkAction, snark_job_commitment::P2pChannelsSnarkJobCommitmentAction,
};

use crate::{snark_pool::JobCommitment, ExternalSnarkWorkerAction};

use super::{JobState, SnarkPoolAction, SnarkPoolActionWithMetaRef, SnarkPoolState, SnarkWork};

impl SnarkPoolState {
    pub fn reducer(
        &mut self,
        action: SnarkPoolActionWithMetaRef<'_>,
        global_state: &crate::State,
        dispatcher: &mut redux::ActionQueue<crate::Action, crate::State>,
    ) {
        let (action, meta) = action.split();
        match action {
            SnarkPoolAction::Candidate(action) => {
                self.candidates
                    .reducer(meta.with_action(action), global_state, dispatcher);
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

                self.retain(|id| jobs_map.remove(id).map(|(order, _)| order));
                for (id, (order, job)) in jobs_map {
                    self.insert(JobState {
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
                    let take = self
                        .get(&id)
                        .and_then(|job| job.snark.as_ref())
                        .map_or(true, |old_snark| snark.work > old_snark.work);
                    if take {
                        if let Some(mut job) = self.remove(&id) {
                            job.snark = Some(snark.clone());
                            self.insert(job);
                        }
                    }
                }

                self.candidates_prune();

                // Dispatch
                if let Some(job_id) = global_state.external_snark_worker.working_job_id() {
                    if !global_state.snark_pool.contains(job_id) {
                        // job is no longer needed.
                        dispatcher.push(ExternalSnarkWorkerAction::CancelWork);
                    }
                } else {
                    dispatcher.push(SnarkPoolAction::AutoCreateCommitment);
                }
            }
            SnarkPoolAction::AutoCreateCommitment => {}
            SnarkPoolAction::CommitmentCreate { job_id } => {
                let Some(summary) = global_state.snark_pool.job_summary(job_id) else {
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
                            job_id.clone(),
                            config.fee.clone(),
                            config.public_key.clone().into(),
                        ),
                        sender: global_state.p2p.my_id(),
                    });
                }
            }
            SnarkPoolAction::CommitmentAdd { commitment, sender } => {
                let Some(mut job) = self.remove(&commitment.job_id) else {
                    return;
                };
                job.commitment = Some(JobCommitment {
                    commitment: commitment.clone(),
                    received_t: meta.time(),
                    sender: *sender,
                });
                self.insert(job);

                // Dispatch
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
                let Some(mut job) = self.remove(&job_id) else {
                    return;
                };
                job.snark = Some(SnarkWork {
                    work: snark.clone(),
                    received_t: meta.time(),
                    sender: *sender,
                });
                self.insert(job);
                self.candidates.remove_inferior_snarks(snark);

                // Dispatch
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
                        if *snark > commitment.commitment {
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
                for peer_id in global_state.p2p.ready_peers() {
                    dispatcher.push(SnarkPoolAction::P2pSend { peer_id });
                }
            }
            SnarkPoolAction::P2pSend { peer_id } => {
                let Some(peer) = global_state.p2p.get_ready_peer(peer_id) else {
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
                    peer_id: *peer_id,
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
                    peer_id: *peer_id,
                    snarks,
                    first_index,
                    last_index,
                });
            }
            SnarkPoolAction::CheckTimeouts => {
                self.last_check_timeouts = meta.time();

                // Dispatch
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
                self.remove_commitment(&job_id);

                // Dispatch
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
