use std::collections::BTreeMap;

use openmina_core::snark::SnarkJobId;

use crate::snark_pool::JobCommitment;

use super::{JobState, SnarkPoolAction, SnarkPoolActionWithMetaRef, SnarkPoolState, SnarkWork};

impl SnarkPoolState {
    pub fn reducer(&mut self, action: SnarkPoolActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            SnarkPoolAction::Candidate(action) => self.candidates.reducer(meta.with_action(action)),
            SnarkPoolAction::JobsUpdate(action) => {
                let mut jobs_map = action
                    .jobs
                    .iter()
                    .enumerate()
                    .map(|(index, job)| (SnarkJobId::from(job), (index, job)))
                    .collect::<BTreeMap<_, _>>();

                self.retain(|id| jobs_map.remove(id).map(|(order, _)| order));
                for (id, (order, job)) in jobs_map {
                    self.insert(JobState {
                        time: meta.time(),
                        id,
                        job: job.clone(),
                        commitment: None,
                        snark: None,
                        order,
                    });
                }

                let orphaned_snarks = action
                    .orphaned_snarks
                    .iter()
                    .map(|snark| (snark.work.job_id(), snark));

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
            SnarkPoolAction::AutoCreateCommitment(_) => {}
            SnarkPoolAction::CommitmentCreate(_) => {}
            SnarkPoolAction::CommitmentAdd(a) => {
                let Some(mut job) = self.remove(&a.commitment.job_id) else {
                    return;
                };
                job.commitment = Some(JobCommitment {
                    commitment: a.commitment.clone(),
                    received_t: meta.time(),
                    sender: a.sender,
                });
                self.insert(job);
            }
            SnarkPoolAction::WorkAdd(a) => {
                let job_id = a.snark.job_id();
                let Some(mut job) = self.remove(&job_id) else {
                    return;
                };
                job.snark = Some(SnarkWork {
                    work: a.snark.clone(),
                    received_t: meta.time(),
                    sender: a.sender,
                });
                self.insert(job);
                self.candidates.remove_inferior_snarks(&a.snark);
            }
            SnarkPoolAction::P2pSendAll(_) => {}
            SnarkPoolAction::P2pSend(_) => {}
            SnarkPoolAction::CheckTimeouts(_) => {
                self.last_check_timeouts = meta.time();
            }
            SnarkPoolAction::JobCommitmentTimeout(a) => {
                self.remove_commitment(&a.job_id);
            }
        }
    }
}
