#[cfg(target_family = "wasm")]
use gloo_utils::format::JsValueSerdeExt;
use node::rpc::*;
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

use super::RpcSender;

#[derive(Clone)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub struct Ledger {
    #[allow(unused)]
    sender: RpcSender,
}

#[derive(Clone)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub struct LedgerSelected {
    #[allow(unused)]
    sender: RpcSender,
}

#[derive(Clone)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub struct LedgerAccounts {
    #[allow(unused)]
    sender: RpcSender,
}

impl Ledger {
    pub fn new(sender: RpcSender) -> Self {
        Self { sender }
    }
}

#[cfg_attr(target_family = "wasm", wasm_bindgen)]
impl Ledger {
    pub fn latest(&self) -> LedgerSelected {
        LedgerSelected {
            sender: self.sender.clone(),
        }
    }
}

#[cfg_attr(target_family = "wasm", wasm_bindgen)]
impl LedgerSelected {
    pub fn accounts(&self) -> LedgerAccounts {
        LedgerAccounts {
            sender: self.sender.clone(),
        }
    }
}

impl LedgerAccounts {
    async fn _all(&self) -> Option<RpcLedgerSlimAccountsResponse> {
        self.sender
            .oneshot_request(RpcRequest::LedgerAccountsGet(AccountQuery::All))
            .await
    }
}

#[cfg(not(target_family = "wasm"))]
impl LedgerAccounts {
    pub async fn all(&self) -> Option<RpcLedgerSlimAccountsResponse> {
        self._all().await
    }
}

#[cfg(target_family = "wasm")]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
impl LedgerAccounts {
    pub async fn all(&self) -> JsValue {
        JsValue::from_serde(&self._all().await).unwrap_or_default()
    }
}
