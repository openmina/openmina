use crate::{
    p2p::connection::P2pConnectionResponse,
    rpc::{
        RpcActionStatsGetResponse, RpcBestChainResponse, RpcBlockProducerStatsGetResponse,
        RpcConsensusTimeGetResponse, RpcDiscoveryBoostrapStatsResponse,
        RpcDiscoveryRoutingTableResponse, RpcGenesisBlockResponse, RpcGetBlockResponse,
        RpcHealthCheckResponse, RpcHeartbeatGetResponse, RpcId,
        RpcLedgerAccountDelegatorsGetResponse, RpcLedgerAccountsResponse,
        RpcLedgerSlimAccountsResponse, RpcLedgerStatusGetResponse, RpcMessageProgressResponse,
        RpcP2pConnectionOutgoingResponse, RpcPeersGetResponse, RpcPooledUserCommandsResponse,
        RpcPooledZkappCommandsResponse, RpcReadinessCheckResponse, RpcScanStateSummaryGetResponse,
        RpcSnarkPoolCompletedJobsResponse, RpcSnarkPoolGetResponse, RpcSnarkPoolJobGetResponse,
        RpcSnarkPoolPendingJobsGetResponse, RpcSnarkerConfigGetResponse,
        RpcSnarkerJobCommitResponse, RpcSnarkerJobSpecResponse, RpcSnarkerWorkersResponse,
        RpcStatusGetResponse, RpcSyncStatsGetResponse, RpcTransactionInjectResponse,
        RpcTransactionPoolResponse, RpcTransactionStatusGetResponse,
        RpcTransitionFrontierUserCommandsResponse,
    },
    State,
};
use openmina_core::consensus::ConsensusConstants;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Serialize, Deserialize, Debug, Clone)]
pub enum RespondError {
    #[error("unknown rpc id")]
    UnknownRpcId,
    #[error("unexpected response type")]
    UnexpectedResponseType,
    #[error("responding failed")]
    RespondingFailed,
    #[error("{0}")]
    Custom(String),
}

macro_rules! from_error {
    ($error:ty) => {
        impl From<$error> for RespondError {
            fn from(value: $error) -> Self {
                RespondError::Custom(value.to_string())
            }
        }
    };
}

from_error!(serde_json::Error);

pub trait RpcService {
    fn respond_state_get(
        &mut self,
        rpc_id: RpcId,
        response: (&State, Option<&str>),
    ) -> Result<(), RespondError>;
    fn respond_status_get(
        &mut self,
        rpc_id: RpcId,
        response: RpcStatusGetResponse,
    ) -> Result<(), RespondError>;
    fn respond_heartbeat_get(
        &mut self,
        rpc_id: RpcId,
        response: RpcHeartbeatGetResponse,
    ) -> Result<(), RespondError>;
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
    fn respond_block_producer_stats_get(
        &mut self,
        rpc_id: RpcId,
        response: RpcBlockProducerStatsGetResponse,
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
    fn respond_snark_pool_completed_jobs_get(
        &mut self,
        rpc_id: RpcId,
        response: RpcSnarkPoolCompletedJobsResponse,
    ) -> Result<(), RespondError>;
    fn respond_snark_pool_pending_jobs_get(
        &mut self,
        rpc_id: RpcId,
        response: RpcSnarkPoolPendingJobsGetResponse,
    ) -> Result<(), RespondError>;
    fn respond_snarker_config_get(
        &mut self,
        rpc_id: RpcId,
        response: RpcSnarkerConfigGetResponse,
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
    fn respond_discovery_routing_table(
        &mut self,
        rpc_id: RpcId,
        response: RpcDiscoveryRoutingTableResponse,
    ) -> Result<(), RespondError>;
    fn respond_discovery_bootstrap_stats(
        &mut self,
        rpc_id: RpcId,
        response: RpcDiscoveryBoostrapStatsResponse,
    ) -> Result<(), RespondError>;
    fn respond_readiness_check(
        &mut self,
        rpc_id: RpcId,
        response: RpcReadinessCheckResponse,
    ) -> Result<(), RespondError>;
    fn respond_transaction_pool(
        &mut self,
        rpc_id: RpcId,
        response: RpcTransactionPoolResponse,
    ) -> Result<(), RespondError>;
    fn respond_ledger_slim_accounts(
        &mut self,
        rpc_id: RpcId,
        response: RpcLedgerSlimAccountsResponse,
    ) -> Result<(), RespondError>;
    fn respond_ledger_accounts(
        &mut self,
        rpc_id: RpcId,
        response: RpcLedgerAccountsResponse,
    ) -> Result<(), RespondError>;
    fn respond_transaction_inject(
        &mut self,
        rpc_id: RpcId,
        response: RpcTransactionInjectResponse,
    ) -> Result<(), RespondError>;
    fn respond_transition_frontier_commands(
        &mut self,
        rpc_id: RpcId,
        response: RpcTransitionFrontierUserCommandsResponse,
    ) -> Result<(), RespondError>;
    fn respond_best_chain(
        &mut self,
        rpc_id: RpcId,
        response: RpcBestChainResponse,
    ) -> Result<(), RespondError>;
    fn respond_consensus_constants(
        &mut self,
        rpc_id: RpcId,
        response: ConsensusConstants,
    ) -> Result<(), RespondError>;
    fn respond_transaction_status(
        &mut self,
        rpc_id: RpcId,
        response: RpcTransactionStatusGetResponse,
    ) -> Result<(), RespondError>;
    fn respond_block_get(
        &mut self,
        rpc_id: RpcId,
        response: RpcGetBlockResponse,
    ) -> Result<(), RespondError>;
    fn respond_pooled_user_commands(
        &mut self,
        rpc_id: RpcId,
        response: RpcPooledUserCommandsResponse,
    ) -> Result<(), RespondError>;
    fn respond_pooled_zkapp_commands(
        &mut self,
        rpc_id: RpcId,
        response: RpcPooledZkappCommandsResponse,
    ) -> Result<(), RespondError>;
    fn respond_genesis_block(
        &mut self,
        rpc_id: RpcId,
        response: RpcGenesisBlockResponse,
    ) -> Result<(), RespondError>;
    fn respond_consensus_time_get(
        &mut self,
        rpc_id: RpcId,
        response: RpcConsensusTimeGetResponse,
    ) -> Result<(), RespondError>;
    fn respond_ledger_status_get(
        &mut self,
        rpc_id: RpcId,
        response: RpcLedgerStatusGetResponse,
    ) -> Result<(), RespondError>;
    fn respond_ledger_account_delegators_get(
        &mut self,
        rpc_id: RpcId,
        response: RpcLedgerAccountDelegatorsGetResponse,
    ) -> Result<(), RespondError>;
}
