use std::{collections::BTreeMap, fmt, ops::RangeBounds};

use serde::{Deserialize, Serialize};

use crate::p2p::{
    channels::snark_job_commitment::{SnarkJobCommitment, SnarkJobId},
    PeerId,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobCommitment {
    pub commitment: SnarkJobCommitment,
    pub sender: PeerId,
}

#[derive(Clone)]
pub struct JobCommitmentsState {
    counter: u64,
    list: BTreeMap<u64, JobCommitment>,
    by_ledger_hash_index: BTreeMap<SnarkJobId, u64>,
}

impl JobCommitmentsState {
    pub fn new() -> Self {
        Self {
            counter: 0,
            list: Default::default(),
            by_ledger_hash_index: Default::default(),
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
        counter: u64,
        list: BTreeMap<u64, JobCommitment>,
    }

    impl Serialize for super::JobCommitmentsState {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut s = serializer.serialize_struct("JobCommitments", 2)?;
            s.serialize_field("counter", &self.counter)?;
            s.serialize_field("list", &self.list)?;
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
                counter: v.counter,
                list: v.list,
                by_ledger_hash_index,
            })
        }
    }
}
