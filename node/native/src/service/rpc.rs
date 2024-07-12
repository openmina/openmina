use node::{p2p::connection::P2pConnectionResponse, rpc::*, State};

use crate::NodeService;

macro_rules! rpc_service_impl {
    ($name:ident, $ty:ty) => {
        fn $name(&mut self, rpc_id: RpcId, response: $ty) -> Result<(), RespondError> {
            RpcService::$name(&mut self.common, rpc_id, response)
        }
    };
}

impl RpcService for NodeService {
    rpc_service_impl!(respond_state_get, (&State, Option<&str>));
    rpc_service_impl!(respond_status_get, RpcStatusGetResponse);

    rpc_service_impl!(respond_sync_stats_get, RpcSyncStatsGetResponse);
    rpc_service_impl!(respond_action_stats_get, RpcActionStatsGetResponse);
    rpc_service_impl!(
        respond_block_producer_stats_get,
        RpcBlockProducerStatsGetResponse
    );
    rpc_service_impl!(
        respond_message_progress_stats_get,
        RpcMessageProgressResponse
    );
    rpc_service_impl!(respond_peers_get, RpcPeersGetResponse);
    rpc_service_impl!(
        respond_p2p_connection_outgoing,
        RpcP2pConnectionOutgoingResponse
    );
    rpc_service_impl!(
        respond_p2p_connection_incoming_answer,
        P2pConnectionResponse
    );
    rpc_service_impl!(
        respond_p2p_connection_incoming,
        Result<(), String>
    );

    rpc_service_impl!(
        respond_scan_state_summary_get,
        RpcScanStateSummaryGetResponse
    );
    rpc_service_impl!(respond_snark_pool_get, RpcSnarkPoolGetResponse);
    rpc_service_impl!(respond_snark_pool_job_get, RpcSnarkPoolJobGetResponse);
    rpc_service_impl!(respond_snarker_job_commit, RpcSnarkerJobCommitResponse);
    rpc_service_impl!(
        respond_snarker_job_spec,
        node::rpc::RpcSnarkerJobSpecResponse
    );
    rpc_service_impl!(
        respond_snarker_workers,
        node::rpc::RpcSnarkerWorkersResponse
    );
    rpc_service_impl!(
        respond_snarker_config_get,
        node::rpc::RpcSnarkerConfigGetResponse
    );
    rpc_service_impl!(respond_health_check, RpcHealthCheckResponse);
    rpc_service_impl!(respond_readiness_check, RpcReadinessCheckResponse);
    rpc_service_impl!(
        respond_discovery_routing_table,
        RpcDiscoveryRoutingTableResponse
    );
    rpc_service_impl!(
        respond_discovery_bootstrap_stats,
        RpcDiscoveryBoostrapStatsResponse
    );
    rpc_service_impl!(respond_transaction_pool, RpcTransactionPoolResponse);
    rpc_service_impl!(respond_ledger_accounts, RpcLedgerAccountsResponse);
    rpc_service_impl!(respond_transaction_inject, RpcTransactionInjectResponse);
    rpc_service_impl!(respond_transaction_inject_failed, RpcTransactionInjectFailure);
    rpc_service_impl!(respond_transition_frontier_commands, RpcTransitionFrontierUserCommandsResponse);
}
