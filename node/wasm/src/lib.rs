use std::time::Duration;

use libp2p::futures::channel::mpsc;
use libp2p::futures::select_biased;
use libp2p::futures::FutureExt;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use lib::event_source::{
    Event, EventSourceProcessEventsAction, EventSourceWaitForEventsAction,
    EventSourceWaitTimeoutAction,
};

mod service;
pub use service::NodeWasmService;

pub type Store = lib::Store<NodeWasmService>;
pub type Node = lib::Node<NodeWasmService>;

/// Runs endless loop.
///
/// Doesn't exit.
pub async fn run(mut node: Node) {
    loop {
        let wait_for_events = node
            .store_mut()
            .service
            .event_source_receiver
            .wait_for_events();
        let timeout = wasm_timer::Delay::new(Duration::from_millis(200));

        select_biased! {
            _ = wait_for_events.fuse() => {
                while node.store_mut().service.event_source_receiver.has_next() {
                    node.store_mut().dispatch(EventSourceProcessEventsAction {});
                }
                node.store_mut().dispatch(EventSourceWaitForEventsAction {});
            }
            _ = timeout.fuse() => {
                node.store_mut().dispatch(EventSourceWaitTimeoutAction {});
            }
        }
    }
}

#[wasm_bindgen(js_name = start)]
pub async fn wasm_start() -> Result<JsHandle, JsValue> {
    // buffer size must be 1!
    let (tx, rx) = mpsc::channel(1);

    let service = NodeWasmService {
        event_source_sender: tx.clone(),
        event_source_receiver: rx.into(),
    };
    let state = lib::State::new();
    let node = lib::Node::new(state, service);

    spawn_local(run(node));
    Ok(JsHandle { sender: tx })
}

#[wasm_bindgen]
pub struct JsHandle {
    sender: mpsc::Sender<Event>,
}
