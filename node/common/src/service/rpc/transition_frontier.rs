#[cfg(target_family = "wasm")]
use gloo_utils::format::JsValueSerdeExt;
use node::rpc::*;
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

use super::RpcSender;

#[derive(Clone)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub struct TransitionFrontier {
    #[allow(unused)]
    sender: RpcSender,
}

#[derive(Clone)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub struct TransitionFrontierBestChain {
    #[allow(unused)]
    sender: RpcSender,
}

impl TransitionFrontier {
    pub fn new(sender: RpcSender) -> Self {
        Self { sender }
    }
}

#[cfg_attr(target_family = "wasm", wasm_bindgen)]
impl TransitionFrontier {
    pub fn best_chain(&self) -> TransitionFrontierBestChain {
        TransitionFrontierBestChain {
            sender: self.sender.clone(),
        }
    }
}

impl TransitionFrontierBestChain {
    async fn _user_commands(&self) -> Option<RpcTransitionFrontierUserCommandsResponse> {
        self.sender
            .oneshot_request(RpcRequest::TransitionFrontierUserCommandsGet)
            .await
    }
}

#[cfg(not(target_family = "wasm"))]
impl TransitionFrontierBestChain {
    pub async fn user_commands(&self) -> Option<RpcTransitionFrontierUserCommandsResponse> {
        self._user_commands().await
    }
}

#[cfg(target_family = "wasm")]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
impl TransitionFrontierBestChain {
    pub async fn user_commands(&self) -> JsValue {
        JsValue::from_serde(&self._user_commands().await).unwrap_or_default()
    }
}
