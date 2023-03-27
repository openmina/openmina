use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

use shared::requests::PendingRequests;
use snarker::event_source::Event;
use snarker::p2p::webrtc;
use snarker::rpc::{ActionStatsResponse, RespondError, RpcId, RpcIdType};
use snarker::State;

use super::{SnarkerRpcRequest, SnarkerService};

pub type RpcStateGetResponse = Box<State>;
pub type RpcActionStatsGetResponse = Option<ActionStatsResponse>;
pub type RpcP2pConnectionOutgoingResponse = Result<(), String>;

#[derive(Serialize, Deserialize)]
pub enum RpcP2pConnectionIncomingResponse {
    Answer(Result<webrtc::Answer, String>),
    Result(Result<(), String>),
}

pub struct RpcService {
    pending: PendingRequests<RpcIdType, Box<dyn std::any::Any>>,

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
        // TODO(binier): don't ignore error
        let _ = chan.send(Box::new(response.clone()));
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
        // TODO(binier): don't ignore error
        let _ = chan.send(response);
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
        // TODO(binier): don't ignore error
        let _ = chan.send(response);
        Ok(())
    }

    fn respond_p2p_connection_incoming_answer(
        &mut self,
        rpc_id: RpcId,
        response: Result<webrtc::Answer, String>,
    ) -> Result<(), RespondError> {
        let entry = self.rpc.pending.get(rpc_id);
        let chan = entry.ok_or(RespondError::UnknownRpcId)?;
        let chan = chan
            .downcast_ref::<mpsc::Sender<RpcP2pConnectionIncomingResponse>>()
            .ok_or(RespondError::UnexpectedResponseType)?
            .clone();
        // TODO(binier): don't ignore error
        tokio::task::spawn_local(async move {
            let _ = chan
                .send(RpcP2pConnectionIncomingResponse::Answer(response))
                .await;
        });
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
        // TODO(binier): don't ignore error
        tokio::task::spawn_local(async move {
            let _ = chan
                .send(RpcP2pConnectionIncomingResponse::Result(response))
                .await;
        });
        Ok(())
    }
}
