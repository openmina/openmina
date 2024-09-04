#[cfg(target_family = "wasm")]
use gloo_utils::format::JsValueSerdeExt;
#[cfg(target_family = "wasm")]
use node::rpc::*;
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

use super::RpcSender;

#[derive(Clone)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub struct State {
    #[allow(unused)]
    sender: RpcSender,
}

impl State {
    pub fn new(sender: RpcSender) -> Self {
        Self { sender }
    }
}

#[cfg(target_family = "wasm")]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
impl State {
    pub async fn peers(&self) -> JsValue {
        let res = self
            .sender
            .oneshot_request::<RpcPeersGetResponse>(RpcRequest::PeersGet)
            .await;
        JsValue::from_serde(&res).unwrap_or_default()
    }

    pub async fn message_progress(&self) -> JsValue {
        let res = self
            .sender
            .oneshot_request::<RpcMessageProgressResponse>(RpcRequest::MessageProgressGet)
            .await;
        JsValue::from_serde(&res).unwrap_or_default()
    }
}
