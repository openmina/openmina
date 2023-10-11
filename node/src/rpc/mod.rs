mod rpc_state;
use mina_p2p_messages::v2::{
    MinaBaseSignedCommandPayloadBodyStableV2, MinaBaseTransactionStatusStableV2,
    MinaBaseUserCommandStableV2, MinaTransactionTransactionStableV2,
    SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse, StateHash, TransactionHash,
};
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
pub use openmina_core::requests::{RpcId, RpcIdType};
use openmina_core::snark::SnarkJobId;
use redux::Timestamp;
use serde::{Deserialize, Serialize};

use crate::external_snark_worker::{
    ExternalSnarkWorkerError, ExternalSnarkWorkerWorkError, SnarkWorkSpecError,
};
use crate::p2p::connection::incoming::P2pConnectionIncomingInitOpts;
use crate::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
use crate::p2p::PeerId;
use crate::snark_pool::{JobCommitment, JobSummary};
use crate::stats::actions::{ActionStatsForBlock, ActionStatsSnapshot};
use crate::stats::sync::SyncStatsSnapshot;
use crate::State;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RpcRequest {
    StateGet,
    ActionStatsGet(ActionStatsQuery),
    SyncStatsGet(SyncStatsQuery),
    P2pConnectionOutgoing(P2pConnectionOutgoingInitOpts),
    P2pConnectionIncoming(P2pConnectionIncomingInitOpts),
    ScanStateSummaryGet(RpcScanStateSummaryGetQuery),
    SnarkPoolGet,
    SnarkPoolJobGet { job_id: SnarkJobId },
    SnarkerConfig,
    SnarkerJobCommit { job_id: SnarkJobId },
    SnarkerJobSpec { job_id: SnarkJobId },
    SnarkerWorkers,
    HealthCheck,
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
pub enum RpcScanStateSummaryGetQuery {
    ForBestTip,
    ForBlockWithHash(StateHash),
    ForBlockWithHeight(u32),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum ActionStatsResponse {
    SinceStart { stats: ActionStatsSnapshot },
    ForBlock(ActionStatsForBlock),
}

#[derive(Serialize, Debug, Clone)]
pub struct RpcScanStateSummary {
    pub block: RpcScanStateSummaryBlock,
    pub scan_state: Vec<Vec<RpcScanStateSummaryScanStateJob>>,
}

#[derive(Serialize, Debug, Clone)]
pub struct RpcScanStateSummaryBlock {
    pub hash: StateHash,
    pub height: u32,
    pub global_slot: u32,
    pub transactions: Vec<RpcScanStateSummaryBlockTransaction>,
    pub completed_works: Vec<SnarkJobId>,
}

#[derive(Serialize, Debug, Clone)]
pub struct RpcScanStateSummaryBlockTransaction {
    /// None if hashing fails.
    pub hash: Option<TransactionHash>,
    pub kind: RpcScanStateSummaryBlockTransactionKind,
    pub status: MinaBaseTransactionStatusStableV2,
}

#[derive(Serialize, Debug, Clone)]
pub enum RpcScanStateSummaryBlockTransactionKind {
    Payment,
    StakeDelegation,
    Zkapp,
    FeeTransfer,
    Coinbase,
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "status")]
pub enum RpcScanStateSummaryScanStateJob {
    Empty,
    Todo {
        job_id: SnarkJobId,
        bundle_job_id: SnarkJobId,
        job: RpcScanStateSummaryScanStateJobKind,
        seq_no: u64,
    },
    Pending {
        job_id: SnarkJobId,
        bundle_job_id: SnarkJobId,
        job: RpcScanStateSummaryScanStateJobKind,
        seq_no: u64,
        commitment: Option<JobCommitment>,
        snark: Option<RpcSnarkPoolJobSnarkWork>,
    },
    Done {
        job_id: SnarkJobId,
        bundle_job_id: SnarkJobId,
        job: RpcScanStateSummaryScanStateJobKind,
        seq_no: u64,
        snark: RpcSnarkPoolJobSnarkWorkDone,
    },
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum RpcScanStateSummaryScanStateJobKind {
    Base(RpcScanStateSummaryBlockTransaction),
    Merge,
}

#[derive(Serialize, Debug, Clone)]
pub enum RpcScanStateSummaryScanStateJobStatus {
    Todo,
    Done,
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
    pub snarker: NonZeroCurvePoint,
    pub fee: CurrencyFeeStableV1,
    pub received_t: Timestamp,
    pub sender: PeerId,
}

#[derive(Serialize, Debug, Clone)]
pub struct RpcSnarkPoolJobSnarkWorkDone {
    pub snarker: NonZeroCurvePoint,
    pub fee: CurrencyFeeStableV1,
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
    Err(SnarkWorkSpecError),
    JobNotFound,
}

pub type RpcStateGetResponse = Box<State>;
pub type RpcActionStatsGetResponse = Option<ActionStatsResponse>;
pub type RpcSyncStatsGetResponse = Option<Vec<SyncStatsSnapshot>>;
pub type RpcP2pConnectionOutgoingResponse = Result<(), String>;
pub type RpcScanStateSummaryGetResponse = Option<RpcScanStateSummary>;
pub type RpcSnarkPoolGetResponse = Vec<RpcSnarkPoolJobSummary>;
pub type RpcSnarkPoolJobGetResponse = Option<RpcSnarkPoolJobFull>;
pub type RpcSnarkerConfigGetResponse = Option<RpcSnarkerConfig>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcSnarkerConfig {
    public_key: NonZeroCurvePoint,
    fee: CurrencyFeeStableV1,
}

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
    Working {
        job_id: SnarkJobId,
        summary: JobSummary,
    },
    WorkReady {
        job_id: SnarkJobId,
    },
    WorkError {
        job_id: SnarkJobId,
        error: ExternalSnarkWorkerWorkError,
    },
    Cancelling {
        job_id: SnarkJobId,
    },
    Cancelled {
        job_id: SnarkJobId,
    },
    Error {
        error: ExternalSnarkWorkerError,
        permanent: bool,
    },
    Killing,
}

pub type RpcSnarkerWorkersResponse = Vec<RpcSnarkWorker>;

impl From<&MinaTransactionTransactionStableV2> for RpcScanStateSummaryBlockTransactionKind {
    fn from(value: &MinaTransactionTransactionStableV2) -> Self {
        match value {
            MinaTransactionTransactionStableV2::Command(v) => (&**v).into(),
            MinaTransactionTransactionStableV2::FeeTransfer(_) => Self::FeeTransfer,
            MinaTransactionTransactionStableV2::Coinbase(_) => Self::Coinbase,
        }
    }
}

impl From<&MinaBaseUserCommandStableV2> for RpcScanStateSummaryBlockTransactionKind {
    fn from(value: &MinaBaseUserCommandStableV2) -> Self {
        match value {
            MinaBaseUserCommandStableV2::SignedCommand(v) => match &v.payload.body {
                MinaBaseSignedCommandPayloadBodyStableV2::Payment(_) => Self::Payment,
                MinaBaseSignedCommandPayloadBodyStableV2::StakeDelegation(_) => {
                    Self::StakeDelegation
                }
            },
            MinaBaseUserCommandStableV2::ZkappCommand(_) => Self::Zkapp,
        }
    }
}

pub type RpcHealthCheckResponse = Result<(), String>;
