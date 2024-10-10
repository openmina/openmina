#[cfg(target_family = "wasm")]
use gloo_utils::format::JsValueSerdeExt;
use serde::Serialize;
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

use node::core::channels::{mpsc, oneshot};
use node::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
use node::rpc::*;

use super::ledger::Ledger;
use super::state::State;
use super::stats::Stats;
use super::transaction_pool::TransactionPool;
use super::transition_frontier::TransitionFrontier;
use super::NodeRpcRequest;

#[derive(Clone)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub struct RpcSender {
    tx: mpsc::Sender<NodeRpcRequest>,
}

impl RpcSender {
    pub fn new(tx: mpsc::Sender<NodeRpcRequest>) -> Self {
        Self { tx }
    }

    pub async fn oneshot_request<T>(&self, req: RpcRequest) -> Option<T>
    where
        T: 'static + Send + Serialize,
    {
        let (tx, rx) = oneshot::channel::<T>();
        let responder = Box::new(tx);
        let sender = self.tx.clone();
        let _ = sender.send(NodeRpcRequest { req, responder }).await;

        rx.await.ok()
    }

    pub async fn multishot_request<T>(
        &self,
        expected_messages: usize,
        req: RpcRequest,
    ) -> mpsc::Receiver<T>
    where
        T: 'static + Send + Serialize,
    {
        let (tx, rx) = mpsc::channel::<T>(expected_messages);
        let responder = Box::new(tx);
        let sender = self.tx.clone();
        let _ = sender.send(NodeRpcRequest { req, responder }).await;

        rx
    }
}

impl RpcSender {
    pub async fn peer_connect(
        &self,
        opts: P2pConnectionOutgoingInitOpts,
    ) -> Result<String, String> {
        let peer_id = opts.peer_id().to_string();
        let req = RpcRequest::P2pConnectionOutgoing(opts);
        self.oneshot_request::<RpcP2pConnectionOutgoingResponse>(req)
            .await
            .ok_or_else(|| "state machine shut down".to_owned())??;

        Ok(peer_id)
    }
}

#[cfg_attr(target_family = "wasm", wasm_bindgen)]
impl RpcSender {
    pub fn state(&self) -> State {
        State::new(self.clone())
    }

    pub fn stats(&self) -> Stats {
        Stats::new(self.clone())
    }

    pub fn transaction_pool(&self) -> TransactionPool {
        TransactionPool::new(self.clone())
    }

    pub fn transition_frontier(&self) -> TransitionFrontier {
        TransitionFrontier::new(self.clone())
    }

    pub fn ledger(&self) -> Ledger {
        Ledger::new(self.clone())
    }
}

#[cfg(target_family = "wasm")]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
impl RpcSender {
    pub async fn status(&self) -> JsValue {
        let res = self
            .oneshot_request::<RpcStatusGetResponse>(RpcRequest::StatusGet)
            .await
            .flatten();
        JsValue::from_serde(&res).unwrap_or_default()
    }
}
