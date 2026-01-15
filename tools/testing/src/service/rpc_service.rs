use mina_core::requests::RpcId;
use mina_node::{
    p2p::connection::P2pConnectionResponse, rpc::RpcMessageProgressResponse,
    rpc_effectful::RespondError, service::RpcService, State,
};

macro_rules! to_real {
    ($name:ident, $response:ty $(,)?) => {
        fn $name(&mut self, rpc_id: RpcId, response: $response) -> Result<(), RespondError> {
            self.real.$name(rpc_id, response)
        }
    };
}

impl RpcService for super::NodeTestingService {
    to_real!(respond_state_get, (&State, Option<&str>));
    to_real!(respond_status_get, mina_node::rpc::RpcStatusGetResponse);
    to_real!(
        respond_heartbeat_get,
        mina_node::rpc::RpcHeartbeatGetResponse
    );
    to_real!(
        respond_sync_stats_get,
        mina_node::rpc::RpcSyncStatsGetResponse
    );
    to_real!(
        respond_block_producer_stats_get,
        mina_node::rpc::RpcBlockProducerStatsGetResponse
    );

    to_real!(
        respond_action_stats_get,
        mina_node::rpc::RpcActionStatsGetResponse,
    );
    to_real!(
        respond_message_progress_stats_get,
        RpcMessageProgressResponse
    );
    to_real!(respond_peers_get, mina_node::rpc::RpcPeersGetResponse,);
    to_real!(
        respond_p2p_connection_outgoing,
        mina_node::rpc::RpcP2pConnectionOutgoingResponse,
    );
    to_real!(
        respond_p2p_connection_incoming_answer,
        P2pConnectionResponse,
    );

    to_real!(respond_p2p_connection_incoming, Result<(), String>,);
    to_real!(
        respond_scan_state_summary_get,
        mina_node::rpc::RpcScanStateSummaryGetResponse,
    );
    to_real!(
        respond_snark_pool_get,
        mina_node::rpc::RpcSnarkPoolGetResponse,
    );
    to_real!(
        respond_snark_pool_job_get,
        mina_node::rpc::RpcSnarkPoolJobGetResponse,
    );
    to_real!(
        respond_snark_pool_completed_jobs_get,
        mina_node::rpc::RpcSnarkPoolCompletedJobsResponse,
    );
    to_real!(
        respond_snark_pool_pending_jobs_get,
        mina_node::rpc::RpcSnarkPoolPendingJobsGetResponse
    );
    to_real!(
        respond_snarker_job_commit,
        mina_node::rpc::RpcSnarkerJobCommitResponse,
    );
    to_real!(
        respond_snarker_job_spec,
        mina_node::rpc::RpcSnarkerJobSpecResponse,
    );
    to_real!(
        respond_snarker_workers,
        mina_node::rpc::RpcSnarkerWorkersResponse,
    );
    to_real!(
        respond_snarker_config_get,
        mina_node::rpc::RpcSnarkerConfigGetResponse,
    );
    to_real!(respond_health_check, mina_node::rpc::RpcHealthCheckResponse,);
    to_real!(
        respond_readiness_check,
        mina_node::rpc::RpcReadinessCheckResponse,
    );
    to_real!(
        respond_discovery_routing_table,
        mina_node::rpc::RpcDiscoveryRoutingTableResponse
    );
    to_real!(
        respond_discovery_bootstrap_stats,
        mina_node::rpc::RpcDiscoveryBoostrapStatsResponse
    );
    to_real!(
        respond_transaction_pool,
        mina_node::rpc::RpcTransactionPoolResponse
    );
    to_real!(
        respond_ledger_slim_accounts,
        mina_node::rpc::RpcLedgerSlimAccountsResponse
    );
    to_real!(
        respond_ledger_accounts,
        mina_node::rpc::RpcLedgerAccountsResponse
    );
    to_real!(
        respond_transaction_inject,
        mina_node::rpc::RpcTransactionInjectResponse
    );
    to_real!(
        respond_transition_frontier_commands,
        mina_node::rpc::RpcTransitionFrontierUserCommandsResponse,
    );
    to_real!(respond_best_chain, mina_node::rpc::RpcBestChainResponse,);
    to_real!(
        respond_consensus_constants,
        mina_node::rpc::RpcConsensusConstantsGetResponse,
    );
    to_real!(
        respond_transaction_status,
        mina_node::rpc::RpcTransactionStatusGetResponse,
    );
    to_real!(respond_block_get, mina_node::rpc::RpcGetBlockResponse,);
    to_real!(
        respond_pooled_user_commands,
        mina_node::rpc::RpcPooledUserCommandsResponse,
    );
    to_real!(
        respond_pooled_zkapp_commands,
        mina_node::rpc::RpcPooledZkappCommandsResponse,
    );
    to_real!(
        respond_genesis_block,
        mina_node::rpc::RpcGenesisBlockResponse,
    );
    to_real!(
        respond_consensus_time_get,
        mina_node::rpc::RpcConsensusTimeGetResponse,
    );
    to_real!(
        respond_ledger_status_get,
        mina_node::rpc::RpcLedgerStatusGetResponse,
    );
    to_real!(
        respond_ledger_account_delegators_get,
        mina_node::rpc::RpcLedgerAccountDelegatorsGetResponse,
    );
}
