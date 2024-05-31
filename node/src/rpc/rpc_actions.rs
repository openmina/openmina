use openmina_core::block::ArcBlockWithHash;
use openmina_core::snark::SnarkJobId;
use serde::{Deserialize, Serialize};

use crate::external_snark_worker::SnarkWorkId;
use crate::p2p::connection::incoming::P2pConnectionIncomingInitOpts;
use crate::p2p::connection::outgoing::{P2pConnectionOutgoingError, P2pConnectionOutgoingInitOpts};
use crate::p2p::connection::P2pConnectionResponse;

use super::{
    ActionStatsQuery, RpcId, RpcScanStateSummaryGetQuery, RpcScanStateSummaryScanStateJob,
    SyncStatsQuery,
};

pub type RpcActionWithMeta = redux::ActionWithMeta<RpcAction>;
pub type RpcActionWithMetaRef<'a> = redux::ActionWithMeta<&'a RpcAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum RpcAction {
    GlobalStateGet {
        rpc_id: RpcId,
        filter: Option<String>,
    },
    StatusGet {
        rpc_id: RpcId,
    },

    // Stats
    ActionStatsGet {
        rpc_id: RpcId,
        query: ActionStatsQuery,
    },
    SyncStatsGet {
        rpc_id: RpcId,
        query: SyncStatsQuery,
    },
    BlockProducerStatsGet {
        rpc_id: RpcId,
    },

    MessageProgressGet {
        rpc_id: RpcId,
    },

    PeersGet {
        rpc_id: RpcId,
    },

    P2pConnectionOutgoingInit {
        rpc_id: RpcId,
        opts: P2pConnectionOutgoingInitOpts,
    },
    P2pConnectionOutgoingPending {
        rpc_id: RpcId,
    },
    P2pConnectionOutgoingError {
        rpc_id: RpcId,
        error: P2pConnectionOutgoingError,
    },
    P2pConnectionOutgoingSuccess {
        rpc_id: RpcId,
    },

    P2pConnectionIncomingInit {
        rpc_id: RpcId,
        opts: P2pConnectionIncomingInitOpts,
    },
    P2pConnectionIncomingPending {
        rpc_id: RpcId,
    },
    P2pConnectionIncomingRespond {
        rpc_id: RpcId,
        response: P2pConnectionResponse,
    },
    P2pConnectionIncomingError {
        rpc_id: RpcId,
        error: String,
    },
    P2pConnectionIncomingSuccess {
        rpc_id: RpcId,
    },

    ScanStateSummaryGetInit {
        rpc_id: RpcId,
        query: RpcScanStateSummaryGetQuery,
    },
    ScanStateSummaryLedgerGetInit {
        rpc_id: RpcId,
    },
    ScanStateSummaryGetPending {
        rpc_id: RpcId,
        block: Option<ArcBlockWithHash>,
    },
    ScanStateSummaryGetSuccess {
        rpc_id: RpcId,
        scan_state: Vec<Vec<RpcScanStateSummaryScanStateJob>>,
    },

    SnarkPoolAvailableJobsGet {
        rpc_id: RpcId,
    },
    SnarkPoolJobGet {
        job_id: SnarkWorkId,
        rpc_id: RpcId,
    },

    SnarkerConfigGet {
        rpc_id: RpcId,
    },
    SnarkerJobCommit {
        rpc_id: RpcId,
        job_id: SnarkJobId,
    },
    SnarkerJobSpec {
        rpc_id: RpcId,
        job_id: SnarkJobId,
    },

    SnarkerWorkersGet {
        rpc_id: RpcId,
    },

    HealthCheck {
        rpc_id: RpcId,
    },
    ReadinessCheck {
        rpc_id: RpcId,
    },

    DiscoveryRoutingTable {
        rpc_id: RpcId,
    },
    DiscoveryBoostrapStats {
        rpc_id: RpcId,
    },

    Finish {
        rpc_id: RpcId,
    },
}

impl redux::EnablingCondition<crate::State> for RpcAction {
    fn is_enabled(&self, state: &crate::State, _time: redux::Timestamp) -> bool {
        match self {
            RpcAction::GlobalStateGet { .. } => true,
            RpcAction::StatusGet { .. } => true,
            RpcAction::ActionStatsGet { .. } => true,
            RpcAction::SyncStatsGet { .. } => true,
            RpcAction::BlockProducerStatsGet { .. } => true,
            RpcAction::MessageProgressGet { .. } => true,
            RpcAction::PeersGet { .. } => true,
            RpcAction::P2pConnectionOutgoingInit { rpc_id, .. } => {
                !state.rpc.requests.contains_key(rpc_id)
            }
            RpcAction::P2pConnectionOutgoingPending { rpc_id } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_init()),
            RpcAction::P2pConnectionOutgoingError { rpc_id, .. } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_pending()),
            RpcAction::P2pConnectionOutgoingSuccess { rpc_id } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_pending()),
            RpcAction::P2pConnectionIncomingInit { rpc_id, .. } => {
                !state.rpc.requests.contains_key(rpc_id)
            }
            RpcAction::P2pConnectionIncomingPending { rpc_id } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_init()),
            RpcAction::P2pConnectionIncomingRespond { rpc_id, .. } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_init() || v.status.is_pending()),
            RpcAction::P2pConnectionIncomingError { rpc_id, .. } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_init() || v.status.is_pending()),
            RpcAction::P2pConnectionIncomingSuccess { rpc_id } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_pending()),
            RpcAction::ScanStateSummaryGetInit { .. } => true,
            RpcAction::ScanStateSummaryLedgerGetInit { rpc_id, .. } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_init()),
            RpcAction::ScanStateSummaryGetPending { rpc_id, .. } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_init()),
            RpcAction::ScanStateSummaryGetSuccess { rpc_id, .. } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_pending()),
            RpcAction::SnarkPoolAvailableJobsGet { .. } => true,
            RpcAction::SnarkPoolJobGet { .. } => true,
            RpcAction::SnarkerConfigGet { .. } => true,
            RpcAction::SnarkerJobCommit { .. } => true,
            RpcAction::SnarkerJobSpec { .. } => true,
            RpcAction::SnarkerWorkersGet { .. } => true,
            RpcAction::HealthCheck { .. } => true,
            RpcAction::ReadinessCheck { .. } => true,
            RpcAction::DiscoveryRoutingTable { .. } => true,
            RpcAction::DiscoveryBoostrapStats { .. } => true,
            RpcAction::Finish { rpc_id } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_finished()),
        }
    }
}
