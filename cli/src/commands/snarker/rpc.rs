use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

use node::p2p::connection::P2pConnectionResponse;
pub use node::rpc::{
    ActionStatsResponse, RespondError, RpcActionStatsGetResponse, RpcId, RpcIdType,
    RpcP2pConnectionOutgoingResponse, RpcScanStateSummaryGetResponse, RpcSnarkPoolGetResponse,
    RpcSnarkerJobCommitResponse, RpcSnarkerJobSpecResponse, RpcStateGetResponse,
    RpcSyncStatsGetResponse,
};
use node::State;
use node::{event_source::Event, rpc::RpcSnarkPoolJobGetResponse};
use openmina_core::requests::PendingRequests;

use super::{SnarkerRpcRequest, SnarkerService};

#[derive(Serialize, Deserialize, Debug)]
pub enum RpcP2pConnectionIncomingResponse {
    Answer(P2pConnectionResponse),
    Result(Result<(), String>),
}

pub struct RpcService {
    pending: PendingRequests<RpcIdType, Box<dyn Send + std::any::Any>>,

    req_sender: mpsc::Sender<SnarkerRpcRequest>,
    req_receiver: mpsc::Receiver<SnarkerRpcRequest>,
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
    pub fn req_sender(&mut self) -> &mut mpsc::Sender<SnarkerRpcRequest> {
        &mut self.req_sender
    }

    /// Channel for receiving rpc requests in state machine.
    pub fn req_receiver(&mut self) -> &mut mpsc::Receiver<SnarkerRpcRequest> {
        &mut self.req_receiver
    }
}

impl SnarkerService {
    /// Channel for sending the rpc request to state machine.
    #[allow(dead_code)]
    pub fn rpc_req_sender(&mut self) -> &mut mpsc::Sender<SnarkerRpcRequest> {
        &mut self.rpc.req_sender
    }

    pub fn process_rpc_request(&mut self, req: SnarkerRpcRequest) {
        let rpc_id = self.rpc.pending.add(req.responder);
        let req = req.req;
        let tx = self.event_sender.clone();

        let _ = tx.send(Event::Rpc(rpc_id, req));
    }
}

impl node::rpc::RpcService for SnarkerService {
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

    fn respond_sync_stats_get(
        &mut self,
        rpc_id: RpcId,
        response: RpcSyncStatsGetResponse,
    ) -> Result<(), RespondError> {
        let entry = self.rpc.pending.remove(rpc_id);
        let chan = entry.ok_or(RespondError::UnknownRpcId)?;
        let chan = chan
            .downcast::<oneshot::Sender<RpcSyncStatsGetResponse>>()
            .or(Err(RespondError::UnexpectedResponseType))?;
        chan.send(response)
            .or(Err(RespondError::RespondingFailed))?;
        Ok(())
    }

    fn respond_action_stats_get(
        &mut self,
        rpc_id: RpcId,
        response: RpcActionStatsGetResponse,
    ) -> Result<(), RespondError> {
        let entry = self.rpc.pending.remove(rpc_id);
        let chan = entry.ok_or(RespondError::UnknownRpcId)?;
        let chan = chan
            .downcast::<oneshot::Sender<RpcActionStatsGetResponse>>()
            .or(Err(RespondError::UnexpectedResponseType))?;
        chan.send(response)
            .or(Err(RespondError::RespondingFailed))?;
        Ok(())
    }

    fn respond_p2p_connection_outgoing(
        &mut self,
        rpc_id: RpcId,
        response: RpcP2pConnectionOutgoingResponse,
    ) -> Result<(), RespondError> {
        let entry = self.rpc.pending.remove(rpc_id);
        let chan = entry.ok_or(RespondError::UnknownRpcId)?;
        let chan = chan
            .downcast::<oneshot::Sender<RpcP2pConnectionOutgoingResponse>>()
            .or(Err(RespondError::UnexpectedResponseType))?;
        chan.send(response)
            .or(Err(RespondError::RespondingFailed))?;
        Ok(())
    }

    fn respond_p2p_connection_incoming_answer(
        &mut self,
        rpc_id: RpcId,
        response: P2pConnectionResponse,
    ) -> Result<(), RespondError> {
        let entry = self.rpc.pending.get(rpc_id);
        let chan = entry.ok_or(RespondError::UnknownRpcId)?;
        let chan = chan
            .downcast_ref::<mpsc::Sender<RpcP2pConnectionIncomingResponse>>()
            .ok_or(RespondError::UnexpectedResponseType)?
            .clone();
        chan.try_send(RpcP2pConnectionIncomingResponse::Answer(response))
            .or(Err(RespondError::RespondingFailed))?;
        Ok(())
    }

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

    fn respond_scan_state_summary_get(
        &mut self,
        rpc_id: RpcId,
        response: RpcScanStateSummaryGetResponse,
    ) -> Result<(), RespondError> {
        let entry = self.rpc.pending.remove(rpc_id);
        let chan = entry.ok_or(RespondError::UnknownRpcId)?;
        let chan = chan
            .downcast::<oneshot::Sender<RpcScanStateSummaryGetResponse>>()
            .or(Err(RespondError::UnexpectedResponseType))?;
        chan.send(response)
            .or(Err(RespondError::RespondingFailed))?;
        Ok(())
    }

    fn respond_snark_pool_get(
        &mut self,
        rpc_id: RpcId,
        response: RpcSnarkPoolGetResponse,
    ) -> Result<(), RespondError> {
        let entry = self.rpc.pending.remove(rpc_id);
        let chan = entry.ok_or(RespondError::UnknownRpcId)?;
        let chan = chan
            .downcast::<oneshot::Sender<RpcSnarkPoolGetResponse>>()
            .or(Err(RespondError::UnexpectedResponseType))?;
        chan.send(response)
            .or(Err(RespondError::RespondingFailed))?;
        Ok(())
    }

    fn respond_snark_pool_job_get(
        &mut self,
        rpc_id: RpcId,
        response: RpcSnarkPoolJobGetResponse,
    ) -> Result<(), RespondError> {
        let entry = self.rpc.pending.remove(rpc_id);
        let chan = entry.ok_or(RespondError::UnknownRpcId)?;
        let chan = chan
            .downcast::<oneshot::Sender<RpcSnarkPoolJobGetResponse>>()
            .or(Err(RespondError::UnexpectedResponseType))?;
        chan.send(response)
            .or(Err(RespondError::RespondingFailed))?;
        Ok(())
    }

    fn respond_snarker_job_commit(
        &mut self,
        rpc_id: RpcId,
        response: RpcSnarkerJobCommitResponse,
    ) -> Result<(), RespondError> {
        let entry = self.rpc.pending.remove(rpc_id);
        let chan = entry.ok_or(RespondError::UnknownRpcId)?;
        let chan = chan
            .downcast::<oneshot::Sender<RpcSnarkerJobCommitResponse>>()
            .or(Err(RespondError::UnexpectedResponseType))?;
        chan.send(response)
            .or(Err(RespondError::RespondingFailed))?;
        Ok(())
    }

    fn respond_snarker_job_spec(
        &mut self,
        rpc_id: RpcId,
        response: node::rpc::RpcSnarkerJobSpecResponse,
    ) -> Result<(), RespondError> {
        let entry = self.rpc.pending.remove(rpc_id);
        let chan = entry.ok_or(RespondError::UnknownRpcId)?;
        let chan = chan
            .downcast::<oneshot::Sender<RpcSnarkerJobSpecResponse>>()
            .or(Err(RespondError::UnexpectedResponseType))?;
        chan.send(response.clone())
            .or(Err(RespondError::RespondingFailed))?;
        Ok(())
    }

    fn respond_snarker_workers(
        &mut self,
        rpc_id: RpcId,
        response: node::rpc::RpcSnarkerWorkersResponse,
    ) -> Result<(), RespondError> {
        let entry = self.rpc.pending.remove(rpc_id);
        let chan = entry.ok_or(RespondError::UnknownRpcId)?;
        let chan = chan
            .downcast::<oneshot::Sender<node::rpc::RpcSnarkerWorkersResponse>>()
            .or(Err(RespondError::UnexpectedResponseType))?;
        chan.send(response.clone())
            .or(Err(RespondError::RespondingFailed))?;
        Ok(())
    }

    fn respond_snarker_config_get(
        &mut self,
        rpc_id: RpcId,
        response: node::rpc::RpcSnarkerConfigGetResponse,
    ) -> Result<(), RespondError> {
        let entry = self.rpc.pending.remove(rpc_id);
        let chan = entry.ok_or(RespondError::UnknownRpcId)?;
        let chan = chan
            .downcast::<oneshot::Sender<node::rpc::RpcSnarkerConfigGetResponse>>()
            .or(Err(RespondError::UnexpectedResponseType))?;
        chan.send(response.clone())
            .or(Err(RespondError::RespondingFailed))?;
        Ok(())
    }
}
