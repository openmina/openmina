use serde::{Deserialize, Serialize};
use shared::snark_job_id::SnarkJobId;
use thiserror::Error;

use crate::p2p::connection::P2pConnectionResponse;
use crate::stats::sync::SyncStatsSnapshot;
use crate::State;

use super::{ActionStatsResponse, RpcId, SnarkerJobCommitResponse};

#[derive(Error, Serialize, Deserialize, Debug, Clone)]
pub enum RespondError {
    #[error("unknown rpc id")]
    UnknownRpcId,
    #[error("unexpected response type")]
    UnexpectedResponseType,
    #[error("responding failed")]
    RespondingFailed,
}

pub trait RpcService: redux::Service {
    fn respond_state_get(&mut self, rpc_id: RpcId, response: &State) -> Result<(), RespondError>;
    fn respond_action_stats_get(
        &mut self,
        rpc_id: RpcId,
        response: Option<ActionStatsResponse>,
    ) -> Result<(), RespondError>;
    fn respond_sync_stats_get(
        &mut self,
        rpc_id: RpcId,
        response: Option<Vec<SyncStatsSnapshot>>,
    ) -> Result<(), RespondError>;
    fn respond_p2p_connection_outgoing(
        &mut self,
        rpc_id: RpcId,
        response: Result<(), String>,
    ) -> Result<(), RespondError>;
    fn respond_p2p_connection_incoming_answer(
        &mut self,
        rpc_id: RpcId,
        response: P2pConnectionResponse,
    ) -> Result<(), RespondError>;
    fn respond_p2p_connection_incoming(
        &mut self,
        rpc_id: RpcId,
        response: Result<(), String>,
    ) -> Result<(), RespondError>;
    fn respond_snark_pool_available_jobs(
        &mut self,
        rpc_id: RpcId,
        response: Vec<SnarkJobId>,
    ) -> Result<(), RespondError>;
    fn respond_snarker_job_commit(
        &mut self,
        rpc_id: RpcId,
        response: SnarkerJobCommitResponse,
    ) -> Result<(), RespondError>;
}
