use libp2p::futures::channel::{mpsc, oneshot};
use libp2p::futures::SinkExt;
use wasm_bindgen_futures::spawn_local;

use lib::event_source::Event;
use lib::rpc::{RespondError, RpcId, RpcIdType};
use lib::State;
use shared::requests::PendingRequests;

use crate::{NodeWasmService, WasmRpcRequest};

pub struct RpcService {
    pending: PendingRequests<RpcIdType, Box<dyn std::any::Any>>,

    wasm_req_sender: mpsc::Sender<WasmRpcRequest>,
    wasm_req_receiver: mpsc::Receiver<WasmRpcRequest>,
}

impl RpcService {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(8);
        Self {
            pending: Default::default(),
            wasm_req_sender: tx,
            wasm_req_receiver: rx,
        }
    }

    /// Channel for receiving rpc requests in state machine.
    pub fn wasm_req_receiver(&mut self) -> &mut mpsc::Receiver<WasmRpcRequest> {
        &mut self.wasm_req_receiver
    }
}

impl NodeWasmService {
    /// Channel for sending the rpc request to state machine.
    pub fn wasm_rpc_req_sender(&mut self) -> &mut mpsc::Sender<WasmRpcRequest> {
        &mut self.rpc.wasm_req_sender
    }

    pub fn process_wasm_rpc_request(&mut self, req: WasmRpcRequest) {
        let rpc_id = self.rpc.pending.add(req.responder);
        let req = req.req;
        let mut tx = self.event_source_sender.clone();

        spawn_local(async move {
            let _ = tx.send(Event::Rpc(rpc_id, req)).await;
        });
    }
}

impl lib::rpc::RpcService for NodeWasmService {
    fn respond_state_get(&mut self, rpc_id: RpcId, response: &State) -> Result<(), RespondError> {
        let entry = self.rpc.pending.remove(rpc_id);
        let chan = entry.ok_or(RespondError::UnknownRpcId)?;
        let chan = chan
            .downcast::<oneshot::Sender<Box<State>>>()
            .or(Err(RespondError::UnexpectedResponseType))?;
        // TODO(binier): don't ignore error
        let _ = chan.send(Box::new(response.clone()));
        Ok(())
    }
}
