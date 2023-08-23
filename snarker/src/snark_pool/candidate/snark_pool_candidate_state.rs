use std::collections::{BTreeMap, BTreeSet, VecDeque};

use mina_p2p_messages::v2::{CurrencyFeeStableV1, NonZeroCurvePoint};
use redux::Timestamp;
use serde::{Deserialize, Serialize};
use shared::snark::{Snark, SnarkInfo, SnarkJobId};

use crate::p2p::channels::rpc::P2pRpcId;
use crate::p2p::PeerId;
use crate::snark::work_verify::SnarkWorkVerifyId;

static EMPTY_PEER_WORK_CANDIDATES: BTreeMap<SnarkJobId, SnarkPoolCandidateState> = BTreeMap::new();

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolCandidatesState {
    by_peer: BTreeMap<PeerId, BTreeMap<SnarkJobId, SnarkPoolCandidateState>>,
    by_job_id: BTreeMap<SnarkJobId, BTreeSet<PeerId>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SnarkPoolCandidateState {
    InfoReceived {
        time: Timestamp,
        fee: CurrencyFeeStableV1,
        prover: NonZeroCurvePoint,
    },
    WorkFetchPending {
        time: Timestamp,
        fee: CurrencyFeeStableV1,
        prover: NonZeroCurvePoint,
        rpc_id: P2pRpcId,
    },
    WorkReceived {
        time: Timestamp,
        work: Snark,
    },
    WorkVerifyPending {
        time: Timestamp,
        work: Snark,
        verify_id: SnarkWorkVerifyId,
    },
    WorkVerifyError {
        time: Timestamp,
        work: Snark,
    },
    WorkVerifySuccess {
        time: Timestamp,
        work: Snark,
    },
}

impl SnarkPoolCandidatesState {
    pub fn new() -> Self {
        Self {
            by_peer: Default::default(),
            by_job_id: Default::default(),
        }
    }

    pub fn peer_work_count(&self, peer_id: &PeerId) -> usize {
        self.by_peer.get(peer_id).map(|v| v.len()).unwrap_or(0)
    }

    pub fn get(&self, peer_id: PeerId, job_id: &SnarkJobId) -> Option<&SnarkPoolCandidateState> {
        self.by_peer.get(&peer_id)?.get(job_id)
    }

    fn jobs_from_peer_or_empty(
        &self,
        peer_id: PeerId,
    ) -> &BTreeMap<SnarkJobId, SnarkPoolCandidateState> {
        self.by_peer
            .get(&peer_id)
            .unwrap_or(&EMPTY_PEER_WORK_CANDIDATES)
    }

    pub fn jobs_from_peer_iter(
        &self,
        peer_id: PeerId,
    ) -> impl Iterator<Item = (&SnarkJobId, &SnarkPoolCandidateState)> {
        self.jobs_from_peer_or_empty(peer_id).iter()
    }

    pub fn jobs_from_peer_with_job_ids<'a, I>(
        &'a self,
        peer_id: PeerId,
        job_ids: I,
    ) -> impl Iterator<Item = (&'a SnarkJobId, Option<&'a SnarkPoolCandidateState>)>
    where
        I: IntoIterator<Item = &'a SnarkJobId>,
    {
        let jobs = self.jobs_from_peer_or_empty(peer_id);
        job_ids.into_iter().map(|id| (id, jobs.get(id)))
    }

    pub fn info_received(&mut self, time: Timestamp, peer_id: PeerId, info: SnarkInfo) {
        self.by_job_id
            .entry(info.job_id.clone())
            .or_default()
            .insert(peer_id);

        let state = SnarkPoolCandidateState::InfoReceived {
            time,
            fee: info.fee,
            prover: info.prover,
        };
        self.by_peer
            .entry(peer_id)
            .or_default()
            .insert(info.job_id, state);
    }

    pub fn peers_next_work_to_fetch<I, F>(
        &self,
        peers: I,
        get_order: F,
    ) -> Vec<(PeerId, SnarkJobId)>
    where
        I: IntoIterator<Item = PeerId>,
        F: Copy + Fn(&SnarkJobId) -> usize,
    {
        let mut needs_fetching = peers
            .into_iter()
            .filter_map(|peer_id| Some((peer_id, self.by_peer.get(&peer_id)?)))
            .flat_map(|(peer_id, jobs)| {
                jobs.iter()
                    .filter(|(_, state)| {
                        matches!(state, SnarkPoolCandidateState::InfoReceived { .. })
                    })
                    .map(move |(job_id, state)| (get_order(job_id), state.fee(), peer_id, job_id))
            })
            .collect::<Vec<_>>();
        needs_fetching
            .sort_by(|(ord1, fee1, ..), (ord2, fee2, ..)| ord1.cmp(ord2).then(fee1.cmp(fee2)));

        needs_fetching
            .into_iter()
            .scan(None, |last_ord, (ord, _, peer_id, job_id)| {
                if *last_ord == Some(ord) {
                    return Some(None);
                }
                *last_ord = Some(ord);
                Some(Some((peer_id, job_id.clone())))
            })
            .filter_map(|v| v)
            .collect()
    }

    pub fn work_fetch_pending(
        &mut self,
        time: Timestamp,
        peer_id: &PeerId,
        job_id: &SnarkJobId,
        rpc_id: P2pRpcId,
    ) {
        if let Some(state) = self
            .by_peer
            .get_mut(&peer_id)
            .and_then(|jobs| jobs.get_mut(job_id))
        {
            if let SnarkPoolCandidateState::InfoReceived { fee, prover, .. } = state {
                *state = SnarkPoolCandidateState::WorkFetchPending {
                    time,
                    fee: fee.clone(),
                    prover: prover.clone(),
                    rpc_id,
                };
            }
        }
    }

    pub fn work_received(&mut self, time: Timestamp, peer_id: PeerId, work: Snark) {
        let job_id = work.job_id();
        self.by_job_id
            .entry(job_id.clone())
            .or_default()
            .insert(peer_id);

        let state = SnarkPoolCandidateState::WorkReceived { time, work };
        self.by_peer
            .entry(peer_id)
            .or_default()
            .insert(job_id, state);
    }

    pub fn get_batch_to_verify<'a, I>(&'a self, job_ids_ordered: I) -> Option<(PeerId, Vec<Snark>)>
    where
        I: IntoIterator<Item = &'a SnarkJobId>,
    {
        for job_id in job_ids_ordered {
            if let Some(res) = None.or_else(|| {
                for peer_id in self.by_job_id.get(job_id)? {
                    let peer_jobs = self.by_peer.get(peer_id)?;
                    if peer_jobs.get(job_id)?.work().is_some() {
                        let jobs = peer_jobs
                            .iter()
                            .filter_map(|(_, v)| match v {
                                SnarkPoolCandidateState::WorkReceived { work, .. } => Some(work),
                                _ => None,
                            })
                            .cloned()
                            .collect();
                        return Some((*peer_id, jobs));
                    }
                }
                None
            }) {
                return Some(res);
            }
        }
        None
    }

    pub fn verify_pending(
        &mut self,
        time: Timestamp,
        peer_id: &PeerId,
        verify_id: SnarkWorkVerifyId,
        job_ids: &[SnarkJobId],
    ) {
        let Some(peer_jobs) = self.by_peer.get_mut(peer_id) else { return };

        for job_id in job_ids {
            if let Some(job_state) = peer_jobs.get_mut(job_id) {
                if let SnarkPoolCandidateState::WorkReceived { work, .. } = job_state {
                    *job_state = SnarkPoolCandidateState::WorkVerifyPending {
                        time,
                        work: work.clone(),
                        verify_id,
                    };
                }
            }
        }
    }

    pub fn verify_result(
        &mut self,
        time: Timestamp,
        peer_id: &PeerId,
        verify_id: SnarkWorkVerifyId,
        result: Result<(), ()>,
    ) {
        if let Some(peer_jobs) = self.by_peer.get_mut(peer_id) {
            for (_, job_state) in peer_jobs
                .iter_mut()
                .filter(|(_, job_state)| job_state.pending_verify_id() == Some(verify_id))
            {
                let SnarkPoolCandidateState::WorkVerifyPending {
                    work,
                    ..
                } = job_state else { continue };
                match result {
                    Ok(_) => {
                        *job_state = SnarkPoolCandidateState::WorkVerifySuccess {
                            time,
                            work: work.clone(),
                        };
                    }
                    Err(_) => {
                        *job_state = SnarkPoolCandidateState::WorkVerifyError {
                            time,
                            work: work.clone(),
                        };
                    }
                }
            }
        }
    }

    pub fn remove_snarks_with_higher_fee(&mut self, job_id: &SnarkJobId, max_fee: u64) {
        let by_peer = &mut self.by_peer;
        if let Some(peers) = self.by_job_id.get(job_id) {
            for peer_id in peers.iter() {
                let Some(jobs) = by_peer.get_mut(peer_id) else { continue };
                let Some(job) = jobs.get(job_id) else { continue };
                if job.fee() >= max_fee {
                    jobs.remove(job_id);
                }
            }
        }
    }

    pub fn retain<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&SnarkJobId) -> bool,
    {
        let by_peer = &mut self.by_peer;
        self.by_job_id.retain(|job_id, peers| {
            if predicate(job_id) {
                return true;
            }
            for peer_id in peers.iter() {
                if let Some(peer_works) = by_peer.get_mut(peer_id) {
                    peer_works.remove(job_id);
                }
            }
            false
        })
    }
}

impl SnarkPoolCandidateState {
    pub fn fee(&self) -> u64 {
        match self {
            Self::InfoReceived { fee, .. } | Self::WorkFetchPending { fee, .. } => fee.0.as_u64(),
            Self::WorkReceived { work, .. }
            | Self::WorkVerifyPending { work, .. }
            | Self::WorkVerifyError { work, .. }
            | Self::WorkVerifySuccess { work, .. } => work.fee.0.as_u64(),
        }
    }

    pub fn work(&self) -> Option<&Snark> {
        match self {
            Self::InfoReceived { .. } => None,
            Self::WorkFetchPending { .. } => None,
            Self::WorkReceived { work, .. } => Some(work),
            Self::WorkVerifyPending { work, .. } => Some(work),
            Self::WorkVerifyError { work, .. } => Some(work),
            Self::WorkVerifySuccess { work, .. } => Some(work),
        }
    }

    pub fn pending_verify_id(&self) -> Option<SnarkWorkVerifyId> {
        match self {
            Self::WorkVerifyPending { verify_id, .. } => Some(*verify_id),
            _ => None,
        }
    }
}
