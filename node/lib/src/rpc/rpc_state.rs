use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::RpcId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcRequestState {
    req: RpcRequest,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RpcRequest {
    GetState,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcState {
    pub requests: BTreeMap<RpcId, RpcRequestState>,
}

impl RpcState {
    pub fn new() -> Self {
        Self {
            requests: Default::default(),
        }
    }
}
