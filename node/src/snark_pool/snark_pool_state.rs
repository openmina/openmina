use std::time::Duration;
use std::{fmt, ops::RangeBounds};

use ledger::scan_state::scan_state::{transaction_snark::OneOrTwo, AvailableJobMessage};
use openmina_core::snark::{Snark, SnarkInfo, SnarkJobCommitment, SnarkJobId};
use redux::Timestamp;
use serde::{Deserialize, Serialize};

use crate::core::distributed_pool::DistributedPool;
use crate::p2p::PeerId;

use super::candidate::SnarkPoolCandidatesState;
use super::SnarkPoolConfig;

#[derive(Serialize, Deserialize, Clone)]
pub struct SnarkPoolState {
    config: SnarkPoolConfig,
    pool: DistributedPool<JobState, SnarkJobId>,
    pub candidates: SnarkPoolCandidatesState,
    pub(super) last_check_timeouts: Timestamp,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobState {
    pub time: Timestamp,
    pub id: SnarkJobId,
    pub job: OneOrTwo<AvailableJobMessage>,
    pub commitment: Option<JobCommitment>,
    pub snark: Option<SnarkWork>,
    /// Lower order has higher priority to be done as it represents older job.
    pub order: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobCommitment {
    pub commitment: SnarkJobCommitment,
    pub received_t: Timestamp,
    pub sender: PeerId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkWork {
    pub work: Snark,
    pub received_t: Timestamp,
    pub sender: PeerId,
}

/// Whether the job is a merge proof job, or a transaction proof job, with particular number of account updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobSummary {
    Tx(usize),
    Merge(usize),
}

impl Default for SnarkPoolState {
    fn default() -> Self {
        Self::new()
    }
}

impl SnarkPoolState {
    pub fn new() -> Self {
        Self {
            config: SnarkPoolConfig {},
            pool: Default::default(),
            candidates: SnarkPoolCandidatesState::new(),
            last_check_timeouts: Timestamp::ZERO,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }

    pub fn last_index(&self) -> u64 {
        self.pool.last_index()
    }

    pub fn contains(&self, id: &SnarkJobId) -> bool {
        self.pool.contains(id)
    }

    pub fn get(&self, id: &SnarkJobId) -> Option<&JobState> {
        self.pool.get(id)
    }

    pub fn insert(&mut self, job: JobState) {
        self.pool.insert(job)
    }

    pub fn remove(&mut self, id: &SnarkJobId) -> Option<JobState> {
        self.pool.remove(id)
    }

    pub fn set_snark_work(&mut self, snark: SnarkWork) -> Option<SnarkWork> {
        self.pool
            .update(&snark.work.job_id(), move |job| job.snark.replace(snark))?
    }

    pub fn set_commitment(&mut self, commitment: JobCommitment) -> Option<JobCommitment> {
        let job_id = commitment.commitment.job_id.clone();
        self.pool
            .update(&job_id, move |job| job.commitment.replace(commitment))?
    }

    pub fn remove_commitment(&mut self, id: &SnarkJobId) -> Option<JobCommitment> {
        self.pool
            .silent_update(id, |job_state| job_state.commitment.take())?
    }

    pub fn retain<F>(&mut self, mut get_new_job_order: F)
    where
        F: FnMut(&SnarkJobId) -> Option<usize>,
    {
        self.pool
            .retain_and_update(|id, job| match get_new_job_order(id) {
                None => false,
                Some(order) => {
                    job.order = order;
                    true
                }
            });
    }

    pub fn range<R>(&self, range: R) -> impl '_ + DoubleEndedIterator<Item = (u64, &'_ JobState)>
    where
        R: RangeBounds<u64>,
    {
        self.pool.range(range)
    }

    pub fn should_create_commitment(&self, job_id: &SnarkJobId) -> bool {
        self.get(job_id).is_some_and(|s| s.is_available())
    }

    pub fn is_commitment_timed_out(&self, id: &SnarkJobId, time_now: Timestamp) -> bool {
        self.get(id)
            .is_some_and(|job| is_job_commitment_timed_out(job, time_now))
    }

    pub fn timed_out_commitments_iter(
        &self,
        time_now: Timestamp,
    ) -> impl Iterator<Item = &SnarkJobId> {
        self.jobs_iter()
            .filter(move |job| is_job_commitment_timed_out(job, time_now))
            .map(|job| &job.id)
    }

    pub fn jobs_iter(&self) -> impl Iterator<Item = &JobState> {
        self.pool.states()
    }

    pub fn available_jobs_iter(&self) -> impl Iterator<Item = &JobState> {
        self.jobs_iter().filter(|job| job.is_available())
    }

    pub fn available_jobs_with_highest_priority(&self, n: usize) -> Vec<&JobState> {
        // find `n` jobs with lowest order (highest priority).
        self.available_jobs_iter()
            .fold(Vec::with_capacity(n.saturating_add(1)), |mut jobs, job| {
                jobs.push(job);
                if jobs.len() > n {
                    jobs.sort_by_key(|job| job.order);
                    jobs.pop();
                }
                jobs
            })
    }

    pub fn completed_snarks_iter(&self) -> impl '_ + Iterator<Item = &'_ Snark> {
        self.jobs_iter()
            .filter_map(|job| job.snark.as_ref())
            .map(|snark| &snark.work)
    }

    pub(super) fn job_summary(&self, id: &SnarkJobId) -> Option<JobSummary> {
        self.get(id).map(|job| job.summary())
    }

    pub fn candidates_prune(&mut self) {
        self.candidates.retain(|id| {
            let job = self.pool.get(id);
            move |candidate| match job {
                None => false,
                Some(job) => match job.snark.as_ref() {
                    None => true,
                    Some(snark) => &snark.work < candidate,
                },
            }
        });
    }

    pub fn next_commitments_to_send(
        &self,
        index_and_limit: (u64, u8),
    ) -> (Vec<SnarkJobCommitment>, u64, u64) {
        self.pool
            .next_messages_to_send(index_and_limit, |job| job.commitment_msg().cloned())
    }

    pub fn next_snarks_to_send(&self, index_and_limit: (u64, u8)) -> (Vec<SnarkInfo>, u64, u64) {
        self.pool
            .next_messages_to_send(index_and_limit, |job| job.snark_msg())
    }

    pub fn resources_usage(&self) -> serde_json::Value {
        let (size, inconsistency) = self.candidates.check();

        serde_json::json!({
            "pool_size": self.pool.len(),
            "candidates_size": size,
            "candidates_inconsistency": inconsistency,
        })
    }
}

fn is_job_commitment_timed_out(job: &JobState, time_now: Timestamp) -> bool {
    let Some(commitment) = job.commitment.as_ref() else {
        return false;
    };

    let timeout = job.estimated_duration();
    let passed_time = time_now.checked_sub(commitment.commitment.timestamp());
    let is_timed_out = passed_time.is_some_and(|dur| dur >= timeout);
    let didnt_deliver = job
        .snark
        .as_ref()
        .map_or(true, |snark| snark.work < commitment.commitment);

    is_timed_out && didnt_deliver
}

impl fmt::Debug for SnarkPoolState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SnarkPoolState")
            .field("pool", &self.pool)
            .finish()
    }
}

impl JobState {
    pub fn is_available(&self) -> bool {
        self.commitment.is_none() && self.snark.is_none()
    }

    pub fn commitment_msg(&self) -> Option<&SnarkJobCommitment> {
        self.commitment.as_ref().map(|v| &v.commitment)
    }

    pub fn snark_msg(&self) -> Option<SnarkInfo> {
        self.snark.as_ref().map(|v| v.work.info())
    }

    pub fn summary(&self) -> JobSummary {
        use mina_p2p_messages::v2::{
            MinaTransactionLogicTransactionAppliedCommandAppliedStableV2 as CommandApplied,
            MinaTransactionLogicTransactionAppliedVaryingStableV2 as Varying,
        };
        let account_updates = |job: &_| match job {
            AvailableJobMessage::Base(base) => match &base.transaction_with_info.varying {
                Varying::Command(CommandApplied::ZkappCommand(zkapp)) => {
                    zkapp.command.data.account_updates.len()
                }
                _ => 1,
            },
            AvailableJobMessage::Merge { .. } => 1,
        };
        let account_updates = match &self.job {
            OneOrTwo::One(job) => account_updates(job),
            OneOrTwo::Two((job1, job2)) => account_updates(job1)
                .checked_add(account_updates(job2))
                .expect("overflow"),
        };

        if matches!(
            self.job,
            OneOrTwo::One(AvailableJobMessage::Base(_))
                | OneOrTwo::Two((AvailableJobMessage::Base(_), _))
        ) {
            JobSummary::Tx(account_updates)
        } else {
            JobSummary::Merge(account_updates)
        }
    }

    pub fn estimated_duration(&self) -> Duration {
        self.summary().estimated_duration()
    }
}

impl AsRef<SnarkJobId> for JobState {
    fn as_ref(&self) -> &SnarkJobId {
        &self.id
    }
}

impl JobSummary {
    pub fn estimated_duration(&self) -> Duration {
        const BASE: Duration = Duration::from_secs(10);
        const MAX_LATENCY: Duration = Duration::from_secs(10);

        let (JobSummary::Tx(n) | JobSummary::Merge(n)) = self;
        BASE.saturating_mul(*n as u32).saturating_add(MAX_LATENCY)
    }
}
