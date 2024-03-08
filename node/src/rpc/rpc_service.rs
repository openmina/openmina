use mina_p2p_messages::v2::LedgerHash;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::p2p::connection::P2pConnectionResponse;
use crate::State;

use super::{
    RpcActionStatsGetResponse, RpcHealthCheckResponse, RpcId, RpcMessageProgressResponse,
    RpcP2pConnectionOutgoingResponse, RpcPeersGetResponse, RpcReadinessCheckResponse,
    RpcScanStateSummaryGetResponse, RpcScanStateSummaryScanStateJob, RpcSnarkPoolGetResponse,
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

pub trait RpcLedgerService: redux::Service {
    fn scan_state_summary(
        &self,
        staged_ledger_hash: LedgerHash,
    ) -> Vec<Vec<RpcScanStateSummaryScanStateJob>>;
}

pub trait RpcService: RpcLedgerService {
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
    fn respond_message_progress_stats_get(
        &mut self,
        rpc_id: RpcId,
        response: RpcMessageProgressResponse,
    ) -> Result<(), RespondError>;
    fn respond_peers_get(
        &mut self,
        rpc_id: RpcId,
        response: RpcPeersGetResponse,
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
    fn respond_scan_state_summary_get(
        &mut self,
        rpc_id: RpcId,
        response: RpcScanStateSummaryGetResponse,
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
    fn respond_health_check(
        &mut self,
        rpc_id: RpcId,
        response: RpcHealthCheckResponse,
    ) -> Result<(), RespondError>;
    fn respond_readiness_check(
        &mut self,
        rpc_id: RpcId,
        response: RpcReadinessCheckResponse,
    ) -> Result<(), RespondError>;
}
