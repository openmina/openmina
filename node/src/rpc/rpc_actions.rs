use openmina_core::snark::SnarkJobId;
use serde::{Deserialize, Serialize};

use crate::external_snark_worker::SnarkWorkId;
use crate::p2p::connection::incoming::P2pConnectionIncomingInitOpts;
use crate::p2p::connection::outgoing::{P2pConnectionOutgoingError, P2pConnectionOutgoingInitOpts};
use crate::p2p::connection::P2pConnectionResponse;

use super::{ActionStatsQuery, RpcId, RpcScanStateSummaryGetQuery, SyncStatsQuery};

pub type RpcActionWithMeta = redux::ActionWithMeta<RpcAction>;
pub type RpcActionWithMetaRef<'a> = redux::ActionWithMeta<&'a RpcAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum RpcAction {
    GlobalStateGet(RpcGlobalStateGetAction),

    // Stats
    ActionStatsGet(RpcActionStatsGetAction),
    SyncStatsGet(RpcSyncStatsGetAction),

    P2pConnectionOutgoingInit(RpcP2pConnectionOutgoingInitAction),
    P2pConnectionOutgoingPending(RpcP2pConnectionOutgoingPendingAction),
    P2pConnectionOutgoingError(RpcP2pConnectionOutgoingErrorAction),
    P2pConnectionOutgoingSuccess(RpcP2pConnectionOutgoingSuccessAction),

    P2pConnectionIncomingInit(RpcP2pConnectionIncomingInitAction),
    P2pConnectionIncomingPending(RpcP2pConnectionIncomingPendingAction),
    P2pConnectionIncomingRespond(RpcP2pConnectionIncomingRespondAction),
    P2pConnectionIncomingError(RpcP2pConnectionIncomingErrorAction),
    P2pConnectionIncomingSuccess(RpcP2pConnectionIncomingSuccessAction),

    ScanStateSummaryGet(RpcScanStateSummaryGetAction),

    SnarkPoolAvailableJobsGet(RpcSnarkPoolAvailableJobsGetAction),
    SnarkPoolJobGet(RpcSnarkPoolJobGetAction),

    SnarkerConfigGet(RpcSnarkerConfigGetAction),
    SnarkerJobCommit(RpcSnarkerJobCommitAction),
    SnarkerJobSpec(RpcSnarkerJobSpecAction),

    SnarkerWorkersGet(RpcSnarkersWorkersGetAction),

    HealthCheck(RpcHealthCheckAction),
    ReadinessCheck(RpcReadinessCheckAction),

    Finish(RpcFinishAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcGlobalStateGetAction {
    pub rpc_id: RpcId,
}

impl redux::EnablingCondition<crate::State> for RpcGlobalStateGetAction {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcActionStatsGetAction {
    pub rpc_id: RpcId,
    pub query: ActionStatsQuery,
}

impl redux::EnablingCondition<crate::State> for RpcActionStatsGetAction {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcSyncStatsGetAction {
    pub rpc_id: RpcId,
    pub query: SyncStatsQuery,
}

impl redux::EnablingCondition<crate::State> for RpcSyncStatsGetAction {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcP2pConnectionOutgoingInitAction {
    pub rpc_id: RpcId,
    pub opts: P2pConnectionOutgoingInitOpts,
}

impl redux::EnablingCondition<crate::State> for RpcP2pConnectionOutgoingInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        !state.rpc.requests.contains_key(&self.rpc_id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcP2pConnectionOutgoingPendingAction {
    pub rpc_id: RpcId,
}

impl redux::EnablingCondition<crate::State> for RpcP2pConnectionOutgoingPendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .rpc
            .requests
            .get(&self.rpc_id)
            .map_or(false, |v| v.status.is_init())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcP2pConnectionOutgoingErrorAction {
    pub rpc_id: RpcId,
    pub error: P2pConnectionOutgoingError,
}

impl redux::EnablingCondition<crate::State> for RpcP2pConnectionOutgoingErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .rpc
            .requests
            .get(&self.rpc_id)
            .map_or(false, |v| v.status.is_pending())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcP2pConnectionOutgoingSuccessAction {
    pub rpc_id: RpcId,
}

impl redux::EnablingCondition<crate::State> for RpcP2pConnectionOutgoingSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .rpc
            .requests
            .get(&self.rpc_id)
            .map_or(false, |v| v.status.is_pending())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcP2pConnectionIncomingInitAction {
    pub rpc_id: RpcId,
    pub opts: P2pConnectionIncomingInitOpts,
}

impl redux::EnablingCondition<crate::State> for RpcP2pConnectionIncomingInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        !state.rpc.requests.contains_key(&self.rpc_id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcP2pConnectionIncomingPendingAction {
    pub rpc_id: RpcId,
}

impl redux::EnablingCondition<crate::State> for RpcP2pConnectionIncomingPendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .rpc
            .requests
            .get(&self.rpc_id)
            .map_or(false, |v| v.status.is_init())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcP2pConnectionIncomingRespondAction {
    pub rpc_id: RpcId,
    pub response: P2pConnectionResponse,
}

impl redux::EnablingCondition<crate::State> for RpcP2pConnectionIncomingRespondAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .rpc
            .requests
            .get(&self.rpc_id)
            .map_or(false, |v| v.status.is_init() || v.status.is_pending())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcP2pConnectionIncomingErrorAction {
    pub rpc_id: RpcId,
    pub error: String,
}

impl redux::EnablingCondition<crate::State> for RpcP2pConnectionIncomingErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .rpc
            .requests
            .get(&self.rpc_id)
            .map_or(false, |v| v.status.is_init() || v.status.is_pending())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcP2pConnectionIncomingSuccessAction {
    pub rpc_id: RpcId,
}

impl redux::EnablingCondition<crate::State> for RpcP2pConnectionIncomingSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .rpc
            .requests
            .get(&self.rpc_id)
            .map_or(false, |v| v.status.is_pending())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcScanStateSummaryGetAction {
    pub rpc_id: RpcId,
    pub query: RpcScanStateSummaryGetQuery,
}

impl redux::EnablingCondition<crate::State> for RpcScanStateSummaryGetAction {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcSnarkPoolAvailableJobsGetAction {
    pub rpc_id: RpcId,
}

impl redux::EnablingCondition<crate::State> for RpcSnarkPoolAvailableJobsGetAction {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcSnarkPoolJobGetAction {
    pub job_id: SnarkWorkId,
    pub rpc_id: RpcId,
}

impl redux::EnablingCondition<crate::State> for RpcSnarkPoolJobGetAction {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcSnarkerConfigGetAction {
    pub rpc_id: RpcId,
}

impl redux::EnablingCondition<crate::State> for RpcSnarkerConfigGetAction {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcSnarkerJobCommitAction {
    pub rpc_id: RpcId,
    pub job_id: SnarkJobId,
}

impl redux::EnablingCondition<crate::State> for RpcSnarkerJobCommitAction {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcSnarkerJobSpecAction {
    pub rpc_id: RpcId,
    pub job_id: SnarkJobId,
}

impl redux::EnablingCondition<crate::State> for RpcSnarkerJobSpecAction {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcSnarkersWorkersGetAction {
    pub rpc_id: RpcId,
}

impl redux::EnablingCondition<crate::State> for RpcSnarkersWorkersGetAction {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcHealthCheckAction {
    pub rpc_id: RpcId,
}

impl redux::EnablingCondition<crate::State> for RpcHealthCheckAction {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcReadinessCheckAction {
    pub rpc_id: RpcId,
}

impl redux::EnablingCondition<crate::State> for RpcReadinessCheckAction {}

/// Finish/Cleanup rpc request.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcFinishAction {
    pub rpc_id: RpcId,
}

impl redux::EnablingCondition<crate::State> for RpcFinishAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .rpc
            .requests
            .get(&self.rpc_id)
            .map_or(false, |v| v.status.is_finished())
    }
}

impl_into_global_action!(
    Rpc:
    RpcGlobalStateGetAction,
    RpcActionStatsGetAction,
    RpcSyncStatsGetAction,

    RpcP2pConnectionOutgoingInitAction,
    RpcP2pConnectionOutgoingPendingAction,
    RpcP2pConnectionOutgoingErrorAction,
    RpcP2pConnectionOutgoingSuccessAction,

    RpcP2pConnectionIncomingInitAction,
    RpcP2pConnectionIncomingPendingAction,
    RpcP2pConnectionIncomingRespondAction,
    RpcP2pConnectionIncomingErrorAction,
    RpcP2pConnectionIncomingSuccessAction,

    RpcScanStateSummaryGetAction,

    RpcSnarkPoolAvailableJobsGetAction,
    RpcSnarkPoolJobGetAction,

    RpcSnarkerConfigGetAction,
    RpcSnarkerJobCommitAction,
    RpcSnarkerJobSpecAction,

    RpcSnarkersWorkersGetAction,

    RpcHealthCheckAction,
    RpcReadinessCheckAction,

    RpcFinishAction,
);
