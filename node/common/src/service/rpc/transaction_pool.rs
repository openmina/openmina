#[cfg(target_family = "wasm")]
use gloo_utils::format::JsValueSerdeExt;
use mina_p2p_messages::v2;
use node::rpc::*;
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

use super::RpcSender;

#[derive(Clone)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub struct TransactionPool {
    #[allow(unused)]
    sender: RpcSender,
}

#[derive(Clone)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub struct TransactionPoolInject {
    #[allow(unused)]
    sender: RpcSender,
}

impl TransactionPool {
    pub fn new(sender: RpcSender) -> Self {
        Self { sender }
    }

    pub fn inject(&self) -> TransactionPoolInject {
        TransactionPoolInject {
            sender: self.sender.clone(),
        }
    }

    async fn _get(&self) -> Option<RpcTransactionPoolResponse> {
        self.sender
            .oneshot_request(RpcRequest::TransactionPoolGet)
            .await
    }
}

#[cfg(not(target_family = "wasm"))]
impl TransactionPool {
    pub async fn get(&self) -> Option<RpcTransactionPoolResponse> {
        self._get().await
    }
}

#[cfg(target_family = "wasm")]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
impl TransactionPool {
    pub async fn get(&self) -> JsValue {
        JsValue::from_serde(&self._get().await).unwrap_or_default()
    }
}

impl TransactionPoolInject {
    async fn _payment(
        &self,
        payments: Vec<RpcInjectPayment>,
    ) -> Result<Option<RpcTransactionInjectResponse>, String> {
        let res = self
            .sender
            .oneshot_request(RpcRequest::TransactionInject(
                payments
                    .into_iter()
                    .map(v2::MinaBaseUserCommandStableV2::try_from)
                    .collect::<Result<_, _>>()
                    .map_err(|err| err.to_string())?,
            ))
            .await;
        Ok(res)
    }
}

#[cfg(not(target_family = "wasm"))]
impl TransactionPoolInject {
    pub async fn payment(
        &self,
        payments: Vec<RpcInjectPayment>,
    ) -> Result<Option<RpcTransactionInjectResponse>, String> {
        self._payment(payments).await
    }
}

#[cfg(target_family = "wasm")]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
impl TransactionPoolInject {
    pub async fn payment(&self, payments: JsValue) -> Result<JsValue, JsValue> {
        let payments: Vec<RpcInjectPayment> = if payments.is_array() {
            payments.into_serde().map_err(|err| err.to_string())?
        } else {
            let payment = payments.into_serde().map_err(|err| err.to_string())?;
            vec![payment]
        };

        self._payment(payments)
            .await
            .map(|res| JsValue::from_serde(&res).unwrap_or_default())
            .map_err(Into::into)
    }
}
