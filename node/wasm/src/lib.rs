use std::time::Duration;

use gloo_utils::format::JsValueSerdeExt;
use libp2p::futures::channel::{mpsc, oneshot};
use libp2p::futures::select_biased;
use libp2p::futures::FutureExt;
use libp2p::futures::{SinkExt, StreamExt};
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use lib::event_source::{
    Event, EventSourceProcessEventsAction, EventSourceWaitForEventsAction,
    EventSourceWaitTimeoutAction,
};
use lib::p2p::connection::outgoing::P2pConnectionOutgoingInitAction;
use lib::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
use lib::rpc::RpcRequest;

mod service;
use service::libp2p::Libp2pService;
use service::rpc::RpcService;
pub use service::NodeWasmService;

pub type Store = lib::Store<NodeWasmService>;
pub type Node = lib::Node<NodeWasmService>;

/// Runs endless loop.
///
/// Doesn't exit.
pub async fn run(mut node: Node) {
    let target_peer_id = "QmegiCDEULhpyW55B2qQNMSURWBKSR72445DS6JgQsfkPj";
    let target_node_addr = format!(
        "/dns4/webrtc.minasync.com/tcp/443/http/p2p-webrtc-direct/p2p/{}",
        target_peer_id
    );
    node.store_mut().dispatch(P2pConnectionOutgoingInitAction {
        opts: P2pConnectionOutgoingInitOpts {
            peer_id: target_peer_id.parse().unwrap(),
            addrs: vec![target_node_addr.parse().unwrap()],
        },
    });
    loop {
        let service = &mut node.store_mut().service;
        let wait_for_events = service.event_source_receiver.wait_for_events();
        let wasm_rpc_req_fut = service.rpc.wasm_req_receiver().next().then(|res| async {
            // TODO(binier): optimize maybe to not check it all the time.
            match res {
                Some(v) => v,
                None => std::future::pending().await,
            }
        });
        let timeout = wasm_timer::Delay::new(Duration::from_millis(1000));

        select_biased! {
            _ = wait_for_events.fuse() => {
                while node.store_mut().service.event_source_receiver.has_next() {
                    node.store_mut().dispatch(EventSourceProcessEventsAction {});
                }
                node.store_mut().dispatch(EventSourceWaitForEventsAction {});
            }
            req = wasm_rpc_req_fut.fuse() => {
                node.store_mut().service.process_wasm_rpc_request(req);
            }
            _ = timeout.fuse() => {
                node.store_mut().dispatch(EventSourceWaitTimeoutAction {});
            }
        }
    }
}

#[wasm_bindgen(js_name = start)]
pub async fn wasm_start() -> Result<JsHandle, JsValue> {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    let (tx, rx) = mpsc::channel(1024);

    let service = NodeWasmService {
        event_source_sender: tx.clone(),
        event_source_receiver: rx.into(),
        libp2p: Libp2pService::run(tx.clone()).await,
        rpc: RpcService::new(),
    };
    let state = lib::State::new();
    let mut node = lib::Node::new(state, service);
    let rpc_sender = node.store_mut().service.wasm_rpc_req_sender().clone();

    spawn_local(run(node));
    Ok(JsHandle {
        sender: tx,
        rpc_sender,
    })
}

pub struct WasmRpcRequest {
    pub req: RpcRequest,
    pub responder: Box<dyn std::any::Any>,
}

#[wasm_bindgen]
pub struct JsHandle {
    sender: mpsc::Sender<Event>,
    rpc_sender: mpsc::Sender<WasmRpcRequest>,
}

#[wasm_bindgen]
impl JsHandle {
    async fn rpc_oneshot_request<T>(&mut self, req: RpcRequest) -> JsValue
    where
        T: 'static + Serialize,
    {
        let (tx, rx) = oneshot::channel::<T>();
        let responder = Box::new(tx);
        self.rpc_sender
            .send(WasmRpcRequest { req, responder })
            .await;
        JsValue::from_serde(&rx.await.ok()).unwrap()
    }

    pub async fn global_state_get(&mut self) -> JsValue {
        let req = RpcRequest::GetState;
        self.rpc_oneshot_request::<Box<lib::State>>(req).await
    }
}
