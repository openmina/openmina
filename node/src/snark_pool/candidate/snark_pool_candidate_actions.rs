use std::cmp::Ordering;

use openmina_core::snark::{Snark, SnarkInfo, SnarkJobId};
use serde::{Deserialize, Serialize};

use crate::p2p::channels::rpc::P2pRpcId;
use crate::p2p::PeerId;
use crate::snark::work_verify::SnarkWorkVerifyId;

use super::SnarkPoolCandidateState;

pub type SnarkPoolCandidateActionWithMeta = redux::ActionWithMeta<SnarkPoolCandidateAction>;
pub type SnarkPoolCandidateActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a SnarkPoolCandidateAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum SnarkPoolCandidateAction {
    InfoReceived(SnarkPoolCandidateInfoReceivedAction),
    WorkFetchAll(SnarkPoolCandidateWorkFetchAllAction),
    WorkFetchInit(SnarkPoolCandidateWorkFetchInitAction),
    WorkFetchPending(SnarkPoolCandidateWorkFetchPendingAction),
    WorkReceived(SnarkPoolCandidateWorkReceivedAction),
    WorkVerifyNext(SnarkPoolCandidateWorkVerifyNextAction),
    WorkVerifyPending(SnarkPoolCandidateWorkVerifyPendingAction),
    WorkVerifyError(SnarkPoolCandidateWorkVerifyErrorAction),
    WorkVerifySuccess(SnarkPoolCandidateWorkVerifySuccessAction),
    PeerPrune(SnarkPoolCandidatePeerPruneAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolCandidateInfoReceivedAction {
    pub peer_id: PeerId,
    pub info: SnarkInfo,
}

impl redux::EnablingCondition<crate::State> for SnarkPoolCandidateInfoReceivedAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.snark_pool.contains(&self.info.job_id)
            && state
                .snark_pool
                .candidates
                .get(self.peer_id, &self.info.job_id)
                .map_or(true, |v| &self.info > v)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolCandidateWorkFetchAllAction {}

impl redux::EnablingCondition<crate::State> for SnarkPoolCandidateWorkFetchAllAction {
    fn is_enabled(&self, _: &crate::State) -> bool {
        true
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolCandidateWorkFetchInitAction {
    pub peer_id: PeerId,
    pub job_id: SnarkJobId,
}

impl redux::EnablingCondition<crate::State> for SnarkPoolCandidateWorkFetchInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        let is_peer_available = state
            .p2p
            .get_ready_peer(&self.peer_id)
            .map_or(false, |peer| peer.channels.rpc.can_send_request());
        is_peer_available
            && state
                .snark_pool
                .candidates
                .get(self.peer_id, &self.job_id)
                .map_or(false, |s| {
                    matches!(s, SnarkPoolCandidateState::InfoReceived { .. })
                })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolCandidateWorkFetchPendingAction {
    pub peer_id: PeerId,
    pub job_id: SnarkJobId,
    pub rpc_id: P2pRpcId,
}

impl redux::EnablingCondition<crate::State> for SnarkPoolCandidateWorkFetchPendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .snark_pool
            .candidates
            .get(self.peer_id, &self.job_id)
            .map_or(false, |s| {
                matches!(s, SnarkPoolCandidateState::InfoReceived { .. })
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolCandidateWorkReceivedAction {
    pub peer_id: PeerId,
    pub work: Snark,
}

impl redux::EnablingCondition<crate::State> for SnarkPoolCandidateWorkReceivedAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        let job_id = self.work.job_id();
        state.snark_pool.contains(&job_id)
            && state
                .snark_pool
                .candidates
                .get(self.peer_id, &job_id)
                .map_or(true, |v| match self.work.partial_cmp(v).unwrap() {
                    Ordering::Less => false,
                    Ordering::Greater => true,
                    Ordering::Equal => {
                        matches!(v, SnarkPoolCandidateState::WorkFetchPending { .. })
                    }
                })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolCandidateWorkVerifyNextAction {}

impl redux::EnablingCondition<crate::State> for SnarkPoolCandidateWorkVerifyNextAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.snark.work_verify.jobs.is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolCandidateWorkVerifyPendingAction {
    pub peer_id: PeerId,
    pub job_ids: Vec<SnarkJobId>,
    pub verify_id: SnarkWorkVerifyId,
}

impl redux::EnablingCondition<crate::State> for SnarkPoolCandidateWorkVerifyPendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        !self.job_ids.is_empty()
            && state
                .snark_pool
                .candidates
                .jobs_from_peer_with_job_ids(self.peer_id, &self.job_ids)
                .all(|(_, state)| {
                    matches!(state, Some(SnarkPoolCandidateState::WorkReceived { .. }))
                })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolCandidateWorkVerifyErrorAction {
    pub peer_id: PeerId,
    pub verify_id: SnarkWorkVerifyId,
}

impl redux::EnablingCondition<crate::State> for SnarkPoolCandidateWorkVerifyErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        // TODO(bineir)
        true
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolCandidateWorkVerifySuccessAction {
    pub peer_id: PeerId,
    pub verify_id: SnarkWorkVerifyId,
}

impl redux::EnablingCondition<crate::State> for SnarkPoolCandidateWorkVerifySuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        // TODO(bineir)
        true
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnarkPoolCandidatePeerPruneAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<crate::State> for SnarkPoolCandidatePeerPruneAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state.snark_pool.candidates.peer_work_count(&self.peer_id) > 0
    }
}

use crate::snark_pool::SnarkPoolAction;

macro_rules! impl_into_global_action {
    ($a:ty) => {
        impl From<$a> for crate::Action {
            fn from(value: $a) -> Self {
                Self::SnarkPool(SnarkPoolAction::Candidate(value.into()))
            }
        }
    };
}

impl_into_global_action!(SnarkPoolCandidateInfoReceivedAction);
impl_into_global_action!(SnarkPoolCandidateWorkFetchAllAction);
impl_into_global_action!(SnarkPoolCandidateWorkFetchInitAction);
impl_into_global_action!(SnarkPoolCandidateWorkFetchPendingAction);
impl_into_global_action!(SnarkPoolCandidateWorkReceivedAction);
impl_into_global_action!(SnarkPoolCandidateWorkVerifyNextAction);
impl_into_global_action!(SnarkPoolCandidateWorkVerifyPendingAction);
impl_into_global_action!(SnarkPoolCandidateWorkVerifyErrorAction);
impl_into_global_action!(SnarkPoolCandidateWorkVerifySuccessAction);
impl_into_global_action!(SnarkPoolCandidatePeerPruneAction);
