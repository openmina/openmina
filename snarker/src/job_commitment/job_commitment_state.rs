use std::{collections::BTreeMap, fmt, ops::RangeBounds, time::Duration};

use redux::Timestamp;
use serde::{Deserialize, Serialize};
use shared::snark_job_id::SnarkJobId;

use crate::p2p::{channels::snark_job_commitment::SnarkJobCommitment, PeerId};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobCommitment {
    pub commitment: SnarkJobCommitment,
    pub sender: PeerId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobCommitmentsConfig {
    pub commitment_timeout: Duration,
}

#[derive(Clone)]
pub struct JobCommitmentsState {
    config: JobCommitmentsConfig,
    counter: u64,
    list: BTreeMap<u64, JobCommitment>,
    by_ledger_hash_index: BTreeMap<SnarkJobId, u64>,
    pub(super) last_check_timeouts: Timestamp,
}

impl JobCommitmentsState {
    pub fn new(config: JobCommitmentsConfig) -> Self {
        Self {
            config,
            counter: 0,
            list: Default::default(),
            by_ledger_hash_index: Default::default(),
            last_check_timeouts: Timestamp::ZERO,
        }
    }

    pub fn last_index(&self) -> u64 {
        self.list.last_key_value().map_or(0, |(k, _)| *k)
    }

    pub fn contains(&self, id: &SnarkJobId) -> bool {
        self.by_ledger_hash_index
            .get(id)
            .map_or(false, |i| self.list.contains_key(i))
    }

    pub fn insert(&mut self, commitment: JobCommitment) {
        let id = commitment.commitment.job_id.clone();
        self.list.insert(self.counter, commitment);
        self.by_ledger_hash_index.insert(id, self.counter);
        self.counter += 1;
    }

    pub fn remove(&mut self, id: &SnarkJobId) -> Option<JobCommitment> {
        let index = self.by_ledger_hash_index.remove(id)?;
        self.list.remove(&index)
    }

    pub fn range<'a, R>(
        &'a self,
        range: R,
    ) -> impl 'a + DoubleEndedIterator<Item = (u64, &'a JobCommitment)>
    where
        R: RangeBounds<u64>,
    {
        self.list.range(range).map(|(k, v)| (*k, v))
    }

    pub fn should_create_commitment(&self, job_id: &SnarkJobId) -> bool {
        !self.contains(job_id)
    }

    pub fn is_commitment_timed_out(&self, id: &SnarkJobId, time_now: Timestamp) -> bool {
        self.by_ledger_hash_index.get(id).map_or(false, |i| {
            self.is_commitment_timed_out_by_index(i, time_now)
        })
    }

    pub fn is_commitment_timed_out_by_index(&self, index: &u64, time_now: Timestamp) -> bool {
        self.list
            .get(index)
            .and_then(|v| time_now.checked_sub(v.commitment.timestamp()))
            .map_or(false, |dur| dur >= self.config.commitment_timeout)
    }

    pub fn timed_out_commitments_iter(
        &self,
        time_now: Timestamp,
    ) -> impl Iterator<Item = &SnarkJobId> {
        self.by_ledger_hash_index
            .iter()
            .filter(move |(_, index)| self.is_commitment_timed_out_by_index(index, time_now))
            .map(|(id, _)| id)
    }
}

impl fmt::Debug for JobCommitmentsState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JobCommitments")
            .field("counter", &self.counter)
            .field("len", &self.list.len())
            .finish()
    }
}

mod ser {
    use super::*;
    use serde::ser::SerializeStruct;

    #[derive(Serialize, Deserialize)]
    struct JobCommitments {
        config: JobCommitmentsConfig,
        counter: u64,
        list: BTreeMap<u64, JobCommitment>,
        last_check_timeouts: Timestamp,
    }

    impl Serialize for super::JobCommitmentsState {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut s = serializer.serialize_struct("JobCommitments", 4)?;
            s.serialize_field("config", &self.config)?;
            s.serialize_field("counter", &self.counter)?;
            s.serialize_field("list", &self.list)?;
            s.serialize_field("last_check_timeouts", &self.last_check_timeouts)?;
            s.end()
        }
    }
    impl<'de> Deserialize<'de> for super::JobCommitmentsState {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let v = JobCommitments::deserialize(deserializer)?;
            let by_ledger_hash_index = v
                .list
                .iter()
                .map(|(k, v)| (v.commitment.job_id.clone(), *k))
                .collect();
            Ok(Self {
                config: v.config,
                counter: v.counter,
                list: v.list,
                by_ledger_hash_index,
                last_check_timeouts: v.last_check_timeouts,
            })
        }
    }
}
