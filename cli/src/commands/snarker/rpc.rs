use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

use shared::requests::PendingRequests;
use shared::snark_job_id::SnarkJobId;
use snarker::event_source::Event;
use snarker::p2p::connection::P2pConnectionResponse;
use snarker::rpc::{ActionStatsResponse, RespondError, RpcId, RpcIdType};
use snarker::stats::sync::SyncStatsSnapshot;
use snarker::State;

use super::{SnarkerRpcRequest, SnarkerService};

pub type RpcStateGetResponse = Box<State>;
pub type RpcActionStatsGetResponse = Option<ActionStatsResponse>;
pub type RpcSyncStatsGetResponse = Option<Vec<SyncStatsSnapshot>>;
pub type RpcP2pConnectionOutgoingResponse = Result<(), String>;
pub type RpcSnarkerJobPickAndCommitResponse = Option<SnarkJobId>;

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

impl snarker::rpc::RpcService for SnarkerService {
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
        response: Option<ActionStatsResponse>,
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
        response: Result<(), String>,
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

    fn respond_snarker_job_pick_and_commit(
        &mut self,
        rpc_id: RpcId,
        response: Option<SnarkJobId>,
    ) -> Result<(), RespondError> {
        let entry = self.rpc.pending.remove(rpc_id);
        let chan = entry.ok_or(RespondError::UnknownRpcId)?;
        let chan = chan
            .downcast::<oneshot::Sender<RpcSnarkerJobPickAndCommitResponse>>()
            .or(Err(RespondError::UnexpectedResponseType))?;
        chan.send(response.clone())
            .or(Err(RespondError::RespondingFailed))?;
        Ok(())
    }
}
