#[cfg(target_family = "wasm")]
use gloo_utils::format::JsValueSerdeExt;
#[cfg(target_family = "wasm")]
use node::rpc::*;
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

use super::RpcSender;

#[derive(Clone)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub struct Stats {
    #[allow(unused)]
    sender: RpcSender,
}

impl Stats {
    pub fn new(sender: RpcSender) -> Self {
        Self { sender }
    }
}

#[cfg(target_family = "wasm")]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
impl Stats {
    pub async fn sync(&self, limit: Option<usize>) -> JsValue {
        let query = SyncStatsQuery { limit };
        let res = self
            .sender
            .oneshot_request::<RpcSyncStatsGetResponse>(RpcRequest::SyncStatsGet(query))
            .await
            .flatten();
        JsValue::from_serde(&res).unwrap_or_default()
    }

    pub async fn block_producer(&self) -> JsValue {
        let res = self
            .sender
            .oneshot_request::<RpcBlockProducerStatsGetResponse>(RpcRequest::BlockProducerStatsGet)
            .await
            .flatten();
        JsValue::from_serde(&res).unwrap_or_default()
    }
}
