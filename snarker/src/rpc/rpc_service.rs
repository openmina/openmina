use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::p2p::connection::P2pConnectionResponse;
use crate::State;

use super::{
    RpcActionStatsGetResponse, RpcId, RpcP2pConnectionOutgoingResponse, RpcSnarkPoolGetResponse,
    RpcSnarkPoolJobGetResponse, RpcSnarkerJobCommitResponse, RpcSnarkerJobSpecResponse,
    RpcSnarkerWorkersResponse, RpcSyncStatsGetResponse,
};

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
        response: RpcActionStatsGetResponse,
    ) -> Result<(), RespondError>;
    fn respond_sync_stats_get(
        &mut self,
        rpc_id: RpcId,
        response: RpcSyncStatsGetResponse,
    ) -> Result<(), RespondError>;
    fn respond_p2p_connection_outgoing(
        &mut self,
        rpc_id: RpcId,
        response: RpcP2pConnectionOutgoingResponse,
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
    fn respond_snark_pool_get(
        &mut self,
        rpc_id: RpcId,
        response: RpcSnarkPoolGetResponse,
    ) -> Result<(), RespondError>;
    fn respond_snark_pool_job_get(
        &mut self,
        rpc_id: RpcId,
        response: RpcSnarkPoolJobGetResponse,
    ) -> Result<(), RespondError>;
    fn respond_snarker_config_get(
        &mut self,
        rpc_id: RpcId,
        response: super::RpcSnarkerConfigGetResponse,
    ) -> Result<(), RespondError>;
    fn respond_snarker_job_commit(
        &mut self,
        rpc_id: RpcId,
        response: RpcSnarkerJobCommitResponse,
    ) -> Result<(), RespondError>;
    fn respond_snarker_job_spec(
        &mut self,
        rpc_id: RpcId,
        response: RpcSnarkerJobSpecResponse,
    ) -> Result<(), RespondError>;
    fn respond_snarker_workers(
        &mut self,
        rpc_id: RpcId,
        response: RpcSnarkerWorkersResponse,
    ) -> Result<(), RespondError>;
}
