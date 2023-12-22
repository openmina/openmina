use node::State;
use node::{p2p::connection::P2pConnectionResponse, rpc::RespondError, service::RpcService};
use openmina_core::requests::RpcId;

impl RpcService for super::NodeTestingService {
    fn respond_state_get(&mut self, rpc_id: RpcId, response: &State) -> Result<(), RespondError> {
        self.real.respond_state_get(rpc_id, response)
    }

    fn respond_sync_stats_get(
        &mut self,
        rpc_id: RpcId,
        response: node::rpc::RpcSyncStatsGetResponse,
    ) -> Result<(), RespondError> {
        self.real.respond_sync_stats_get(rpc_id, response)
    }

    fn respond_action_stats_get(
        &mut self,
        rpc_id: RpcId,
        response: node::rpc::RpcActionStatsGetResponse,
    ) -> Result<(), RespondError> {
        self.real.respond_action_stats_get(rpc_id, response)
    }

    fn respond_peers_get(
        &mut self,
        rpc_id: RpcId,
        response: node::rpc::RpcPeersGetResponse,
    ) -> Result<(), RespondError> {
        self.real.respond_peers_get(rpc_id, response)
    }

    fn respond_p2p_connection_outgoing(
        &mut self,
        rpc_id: RpcId,
        response: node::rpc::RpcP2pConnectionOutgoingResponse,
    ) -> Result<(), RespondError> {
        self.real.respond_p2p_connection_outgoing(rpc_id, response)
    }

    fn respond_p2p_connection_incoming_answer(
        &mut self,
        rpc_id: RpcId,
        response: P2pConnectionResponse,
    ) -> Result<(), RespondError> {
        self.real
            .respond_p2p_connection_incoming_answer(rpc_id, response)
    }

    fn respond_p2p_connection_incoming(
        &mut self,
        rpc_id: RpcId,
        response: Result<(), String>,
    ) -> Result<(), RespondError> {
        self.real.respond_p2p_connection_incoming(rpc_id, response)
    }

    fn respond_scan_state_summary_get(
        &mut self,
        rpc_id: RpcId,
        response: node::rpc::RpcScanStateSummaryGetResponse,
    ) -> Result<(), RespondError> {
        self.real.respond_scan_state_summary_get(rpc_id, response)
    }

    fn respond_snark_pool_get(
        &mut self,
        rpc_id: RpcId,
        response: node::rpc::RpcSnarkPoolGetResponse,
    ) -> Result<(), RespondError> {
        self.real.respond_snark_pool_get(rpc_id, response)
    }

    fn respond_snark_pool_job_get(
        &mut self,
        rpc_id: RpcId,
        response: node::rpc::RpcSnarkPoolJobGetResponse,
    ) -> Result<(), RespondError> {
        self.real.respond_snark_pool_job_get(rpc_id, response)
    }

    fn respond_snarker_job_commit(
        &mut self,
        rpc_id: RpcId,
        response: node::rpc::RpcSnarkerJobCommitResponse,
    ) -> Result<(), RespondError> {
        self.real.respond_snarker_job_commit(rpc_id, response)
    }

    fn respond_snarker_job_spec(
        &mut self,
        rpc_id: RpcId,
        response: node::rpc::RpcSnarkerJobSpecResponse,
    ) -> Result<(), RespondError> {
        self.real.respond_snarker_job_spec(rpc_id, response)
    }

    fn respond_snarker_workers(
        &mut self,
        rpc_id: RpcId,
        response: node::rpc::RpcSnarkerWorkersResponse,
    ) -> Result<(), RespondError> {
        self.real.respond_snarker_workers(rpc_id, response)
    }

    fn respond_snarker_config_get(
        &mut self,
        rpc_id: RpcId,
        response: node::rpc::RpcSnarkerConfigGetResponse,
    ) -> Result<(), RespondError> {
        self.real.respond_snarker_config_get(rpc_id, response)
    }

    fn respond_health_check(
        &mut self,
        rpc_id: RpcId,
        response: node::rpc::RpcHealthCheckResponse,
    ) -> Result<(), RespondError> {
        self.real.respond_health_check(rpc_id, response)
    }

    fn respond_readiness_check(
        &mut self,
        rpc_id: RpcId,
        response: node::rpc::RpcReadinessCheckResponse,
    ) -> Result<(), RespondError> {
        self.real.respond_readiness_check(rpc_id, response)
    }
}
