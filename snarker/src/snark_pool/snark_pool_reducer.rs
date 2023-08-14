use std::collections::BTreeMap;

use shared::snark_job_id::SnarkJobId;

use crate::snark_pool::JobCommitment;

use super::{JobState, SnarkPoolAction, SnarkPoolActionWithMetaRef, SnarkPoolState, SnarkWork};

impl SnarkPoolState {
    pub fn reducer(&mut self, action: SnarkPoolActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            SnarkPoolAction::JobsUpdate(action) => {
                let mut jobs_map = action
                    .jobs
                    .iter()
                    .map(|job| (SnarkJobId::from(job), job))
                    .collect::<BTreeMap<_, _>>();

                self.retain(|id| jobs_map.remove(id).is_some());
                for (id, job) in jobs_map {
                    self.insert(JobState {
                        time: meta.time(),
                        id,
                        job: job.clone(),
                        commitment: None,
                        snark: None,
                    });
                }
            }
            SnarkPoolAction::AutoCreateCommitment(_) => {}
            SnarkPoolAction::CommitmentCreate(_) => {}
            SnarkPoolAction::CommitmentAdd(a) => {
                let Some(mut job) = self.remove(&a.commitment.job_id) else { return };
                job.commitment = Some(JobCommitment {
                    commitment: a.commitment.clone(),
                    received_t: meta.time(),
                    sender: a.sender,
                });
                self.insert(job);
            }
            SnarkPoolAction::WorkAdd(a) => {
                let Some(mut job) = self.remove(&a.snark.job_id()) else { return };
                job.snark = Some(SnarkWork {
                    work: a.snark.clone(),
                    received_t: meta.time(),
                    sender: a.sender,
                });
                self.insert(job);
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
