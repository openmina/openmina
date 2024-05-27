use node::rpc::RpcMessageProgressResponse;
use node::State;
use node::{p2p::connection::P2pConnectionResponse, rpc::RespondError, service::RpcService};
use openmina_core::requests::RpcId;

macro_rules! to_real {
    ($name:ident, $response:ty $(,)?) => {
        fn $name(&mut self, rpc_id: RpcId, response: $response) -> Result<(), RespondError> {
            self.real.$name(rpc_id, response)
        }
    };
}

impl RpcService for super::NodeTestingService {
    to_real!(respond_state_get, (&State, Option<&str>));
    to_real!(respond_sync_stats_get, node::rpc::RpcSyncStatsGetResponse);
    to_real!(
        respond_block_producer_stats_get,
        node::rpc::RpcBlockProducerStatsGetResponse
    );

    to_real!(
        respond_action_stats_get,
        node::rpc::RpcActionStatsGetResponse,
    );
    to_real!(
        respond_message_progress_stats_get,
        RpcMessageProgressResponse
    );
    to_real!(respond_peers_get, node::rpc::RpcPeersGetResponse,);
    to_real!(
        respond_p2p_connection_outgoing,
        node::rpc::RpcP2pConnectionOutgoingResponse,
    );
    to_real!(
        respond_p2p_connection_incoming_answer,
        P2pConnectionResponse,
    );

    to_real!(respond_p2p_connection_incoming, Result<(), String>,);
    to_real!(
        respond_scan_state_summary_get,
        node::rpc::RpcScanStateSummaryGetResponse,
    );
    to_real!(respond_snark_pool_get, node::rpc::RpcSnarkPoolGetResponse,);
    to_real!(
        respond_snark_pool_job_get,
        node::rpc::RpcSnarkPoolJobGetResponse,
    );
    to_real!(
        respond_snarker_job_commit,
        node::rpc::RpcSnarkerJobCommitResponse,
    );
    to_real!(
        respond_snarker_job_spec,
        node::rpc::RpcSnarkerJobSpecResponse,
    );
    to_real!(
        respond_snarker_workers,
        node::rpc::RpcSnarkerWorkersResponse,
    );
    to_real!(
        respond_snarker_config_get,
        node::rpc::RpcSnarkerConfigGetResponse,
    );
    to_real!(respond_health_check, node::rpc::RpcHealthCheckResponse,);
    to_real!(
        respond_readiness_check,
        node::rpc::RpcReadinessCheckResponse,
    );
    to_real!(
        respond_discovery_routing_table,
        node::rpc::RpcDiscoveryRoutingTableResponse
    );
    to_real!(
        respond_discovery_bootstrap_stats,
        node::rpc::RpcDiscoveryBoostrapStatsResponse
    );
}
