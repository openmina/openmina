use node::rpc::RpcHealthCheckResponse;
use serde::{Deserialize, Serialize};

use node::core::channels::{mpsc, oneshot};
use node::core::requests::PendingRequests;
use node::p2p::connection::P2pConnectionResponse;
pub use node::rpc::{
    ActionStatsResponse, RespondError, RpcActionStatsGetResponse, RpcId, RpcIdType,
    RpcP2pConnectionOutgoingResponse, RpcScanStateSummaryGetResponse, RpcSnarkPoolGetResponse,
    RpcSnarkerJobCommitResponse, RpcSnarkerJobSpecResponse, RpcStateGetResponse,
    RpcSyncStatsGetResponse,
};
use node::State;
use node::{event_source::Event, rpc::RpcSnarkPoolJobGetResponse};

use super::{NodeRpcRequest, NodeService};

#[derive(Serialize, Deserialize, Debug)]
pub enum RpcP2pConnectionIncomingResponse {
    Answer(P2pConnectionResponse),
    Result(Result<(), String>),
}

pub struct RpcService {
    pending: PendingRequests<RpcIdType, Box<dyn Send + std::any::Any>>,

    req_sender: mpsc::Sender<NodeRpcRequest>,
    req_receiver: mpsc::Receiver<NodeRpcRequest>,
}

impl RpcService {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(8);
        Self {
            pending: Default::default(),
            req_sender: tx,
            req_receiver: rx,
        }
    }

    /// Channel for sending the rpc request to state machine.
    pub fn req_sender(&mut self) -> &mut mpsc::Sender<NodeRpcRequest> {
        &mut self.req_sender
    }

    /// Channel for receiving rpc requests in state machine.
    pub fn req_receiver(&mut self) -> &mut mpsc::Receiver<NodeRpcRequest> {
        &mut self.req_receiver
    }
}

impl NodeService {
    /// Channel for sending the rpc request to state machine.
    #[allow(dead_code)]
    pub fn rpc_req_sender(&mut self) -> &mut mpsc::Sender<NodeRpcRequest> {
        &mut self.rpc.req_sender
    }

    pub fn process_rpc_request(&mut self, req: NodeRpcRequest) {
        let rpc_id = self.rpc.pending.add(req.responder);
        let req = req.req;
        let tx = self.event_sender.clone();

        let _ = tx.send(Event::Rpc(rpc_id, req));
    }
}

macro_rules! rpc_service_impl {
    ($name:ident, $ty:ty) => {
        fn $name(&mut self, rpc_id: RpcId, response: $ty) -> Result<(), RespondError> {
            let entry = self.rpc.pending.remove(rpc_id);
            let chan = entry.ok_or(RespondError::UnknownRpcId)?;
            let chan = chan
                .downcast::<oneshot::Sender<$ty>>()
                .or(Err(RespondError::UnexpectedResponseType))?;
            chan.send(response)
                .or(Err(RespondError::RespondingFailed))?;
            Ok(())
        }
    };
}

impl node::rpc::RpcService for NodeService {
    fn respond_state_get(&mut self, rpc_id: RpcId, response: &State) -> Result<(), RespondError> {
        let entry = self.rpc.pending.remove(rpc_id);
        let chan = entry.ok_or(RespondError::UnknownRpcId)?;
        let chan = chan
            .downcast::<oneshot::Sender<RpcStateGetResponse>>()
            .or(Err(RespondError::UnexpectedResponseType))?;
        chan.send(Box::new(response.clone()))
            .or(Err(RespondError::RespondingFailed))?;
        Ok(())
    }

    rpc_service_impl!(respond_sync_stats_get, RpcSyncStatsGetResponse);
    rpc_service_impl!(respond_action_stats_get, RpcActionStatsGetResponse);
    rpc_service_impl!(
        respond_p2p_connection_outgoing,
        RpcP2pConnectionOutgoingResponse
    );
    rpc_service_impl!(
        respond_p2p_connection_incoming_answer,
        P2pConnectionResponse
    );

    fn respond_p2p_connection_incoming(
        &mut self,
        rpc_id: RpcId,
        response: Result<(), String>,
    ) -> Result<(), RespondError> {
        let entry = self.rpc.pending.remove(rpc_id);
        let chan = entry.ok_or(RespondError::UnknownRpcId)?;
        let chan = chan
            .downcast::<mpsc::Sender<RpcP2pConnectionIncomingResponse>>()
            .or(Err(RespondError::UnexpectedResponseType))?;
        chan.try_send(RpcP2pConnectionIncomingResponse::Result(response))
            .or(Err(RespondError::RespondingFailed))?;
        Ok(())
    }

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
}
