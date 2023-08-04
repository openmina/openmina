mod rpc_state;
use mina_p2p_messages::v2::SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse;
pub use rpc_state::*;

mod rpc_actions;
pub use rpc_actions::*;

mod rpc_reducer;
pub use rpc_reducer::*;

mod rpc_effects;
pub use rpc_effects::*;

mod rpc_service;
pub use rpc_service::*;

mod rpc_impls;

use ledger::scan_state::scan_state::transaction_snark::OneOrTwo;
use ledger::scan_state::scan_state::AvailableJobMessage;
use mina_p2p_messages::v2::{CurrencyFeeStableV1, NonZeroCurvePoint};
use redux::Timestamp;
use serde::{Deserialize, Serialize};
pub use shared::requests::{RpcId, RpcIdType};
use shared::snark_job_id::SnarkJobId;

use crate::external_snark_worker::{ExternalSnarkWorkerWorkError, ExternalSnarkWorkerError};
use crate::p2p::connection::incoming::P2pConnectionIncomingInitOpts;
use crate::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
use crate::p2p::PeerId;
use crate::snark_pool::JobCommitment;
use crate::stats::actions::{ActionStatsForBlock, ActionStatsSnapshot};
use crate::stats::sync::SyncStatsSnapshot;
use crate::State;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RpcRequest {
    GetState,
    ActionStatsGet(ActionStatsQuery),
    SyncStatsGet(SyncStatsQuery),
    P2pConnectionOutgoing(P2pConnectionOutgoingInitOpts),
    P2pConnectionIncoming(P2pConnectionIncomingInitOpts),
    SnarkPoolGet,
    SnarkPoolJobGet { job_id: SnarkJobId },
    SnarkerJobCommit { job_id: SnarkJobId },
    SnarkerJobSpec { job_id: SnarkJobId },
    SnarkerWorkers,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActionStatsQuery {
    SinceStart,
    ForLatestBlock,
    ForBlockWithId(u64),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncStatsQuery {
    pub limit: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum ActionStatsResponse {
    SinceStart { stats: ActionStatsSnapshot },
    ForBlock(ActionStatsForBlock),
}

#[derive(Serialize, Debug, Clone)]
pub struct RpcSnarkPoolJobSummary {
    pub time: Timestamp,
    pub id: SnarkJobId,
    pub commitment: Option<JobCommitment>,
    pub snark: Option<RpcSnarkPoolJobSnarkWork>,
}

#[derive(Serialize, Debug, Clone)]
pub struct RpcSnarkPoolJobFull {
    pub time: Timestamp,
    pub id: SnarkJobId,
    pub job: OneOrTwo<AvailableJobMessage>,
    pub commitment: Option<JobCommitment>,
    pub snark: Option<RpcSnarkPoolJobSnarkWork>,
}

#[derive(Serialize, Debug, Clone)]
pub struct RpcSnarkPoolJobSnarkWork {
    pub prover: NonZeroCurvePoint,
    pub fee: CurrencyFeeStableV1,
    pub received_t: Timestamp,
    pub sender: PeerId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum RpcSnarkerJobCommitResponse {
    Ok,
    JobNotFound,
    JobTaken,
    SnarkerBusy,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum RpcSnarkerJobSpecResponse {
    Ok(SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse),
    JobNotFound,
}

pub type RpcStateGetResponse = Box<State>;
pub type RpcActionStatsGetResponse = Option<ActionStatsResponse>;
pub type RpcSyncStatsGetResponse = Option<Vec<SyncStatsSnapshot>>;
pub type RpcP2pConnectionOutgoingResponse = Result<(), String>;
pub type RpcSnarkPoolGetResponse = Vec<RpcSnarkPoolJobSummary>;
pub type RpcSnarkPoolJobGetResponse = Option<RpcSnarkPoolJobFull>;

#[derive(Serialize, Debug, Clone)]
pub struct RpcSnarkWorker {
    pub time: Option<Timestamp>,
    pub id: Option<String>,
    pub status: RpcSnarkWorkerStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum RpcSnarkWorkerStatus {
    None,
    Starting,
    Idle,
    Working { job_id: SnarkJobId },
    WorkReady { job_id: SnarkJobId },
    WorkError { job_id: SnarkJobId, error: ExternalSnarkWorkerWorkError },
    Error { error: ExternalSnarkWorkerError },
    Killing,
}

pub type RpcSnarkerWorkersResponse = Vec<RpcSnarkWorker>;
