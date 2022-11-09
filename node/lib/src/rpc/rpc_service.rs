use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::State;

#[derive(Error, Serialize, Deserialize, Debug, Clone)]
pub enum RespondError {
    #[error("unknown rpc id")]
    UnknownRpcId,
    #[error("unexpected response type")]
    UnexpectedResponseType,
}

#[derive(Serialize, Deserialize, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct RpcIdType;
impl shared::requests::RequestIdType for RpcIdType {
    fn request_id_type() -> &'static str {
        "RpcId"
    }
}

pub type RpcId = shared::requests::RequestId<RpcIdType>;

pub trait RpcService {
    fn respond_state_get(&mut self, rpc_id: RpcId, response: &State) -> Result<(), RespondError>;
}
