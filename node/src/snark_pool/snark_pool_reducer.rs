use std::collections::BTreeMap;

use openmina_core::snark::SnarkJobId;

use crate::snark_pool::JobCommitment;

use super::{JobState, SnarkPoolAction, SnarkPoolActionWithMetaRef, SnarkPoolState, SnarkWork};

impl SnarkPoolState {
    pub fn reducer(&mut self, action: SnarkPoolActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            SnarkPoolAction::Candidate(action) => {
                self.candidates.reducer(meta.with_action(action));
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
            }
            SnarkPoolAction::AutoCreateCommitment => {}
            SnarkPoolAction::CommitmentCreate { .. } => {}
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
            }
            SnarkPoolAction::P2pSendAll { .. } => {}
            SnarkPoolAction::P2pSend { .. } => {}
            SnarkPoolAction::CheckTimeouts => {
                self.last_check_timeouts = meta.time();
            }
            SnarkPoolAction::JobCommitmentTimeout { job_id } => {
                self.remove_commitment(&job_id);
            }
        }
    }
}
