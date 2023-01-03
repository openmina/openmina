use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::State;

use super::RpcId;

#[derive(Error, Serialize, Deserialize, Debug, Clone)]
pub enum RespondError {
    #[error("unknown rpc id")]
    UnknownRpcId,
    #[error("unexpected response type")]
    UnexpectedResponseType,
}

pub trait RpcService: redux::Service {
    fn respond_state_get(&mut self, rpc_id: RpcId, response: &State) -> Result<(), RespondError>;
    fn respond_p2p_connection_outgoing(
        &mut self,
        rpc_id: RpcId,
        response: Result<(), String>,
    ) -> Result<(), RespondError>;
    fn respond_p2p_publish(
        &mut self,
        rpc_id: RpcId,
        response: Result<(), String>,
    ) -> Result<(), RespondError>;
    fn respond_watched_accounts_add(
        &mut self,
        rpc_id: RpcId,
        response: bool,
    ) -> Result<(), RespondError>;
}
