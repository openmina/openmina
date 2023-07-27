mod rpc_state;
pub use rpc_state::*;

mod rpc_actions;
pub use rpc_actions::*;

mod rpc_reducer;
pub use rpc_reducer::*;

mod rpc_effects;
pub use rpc_effects::*;

mod rpc_service;
pub use rpc_service::*;

use serde::{Deserialize, Serialize};
pub use shared::requests::{RpcId, RpcIdType};
use shared::snark_job_id::SnarkJobId;

use crate::p2p::connection::incoming::P2pConnectionIncomingInitOpts;
use crate::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
use crate::stats::actions::{ActionStatsForBlock, ActionStatsSnapshot};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RpcRequest {
    GetState,
    ActionStatsGet(ActionStatsQuery),
    SyncStatsGet(SyncStatsQuery),
    P2pConnectionOutgoing(P2pConnectionOutgoingInitOpts),
    P2pConnectionIncoming(P2pConnectionIncomingInitOpts),
    SnarkPoolAvailableJobsGet,
    SnarkerJobCommit { job_id: SnarkJobId },
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum SnarkerJobCommitResponse {
    Ok,
    JobNotFound,
    JobTaken,
    SnarkerBusy,
}
