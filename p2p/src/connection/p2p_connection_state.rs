use openmina_core::requests::RpcId;
use redux::Timestamp;
use serde::{Deserialize, Serialize};

pub trait ConnectionState {
    fn is_success(&self) -> bool;
    fn is_error(&self) -> bool;
    fn rpc_id(&self) -> Option<RpcId>;
    fn time(&self) -> Timestamp;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionState<I, O> {
    Incoming(I),
    Outgoing(O),
}

impl<I, O> P2pConnectionState<I, O>
where
    I: ConnectionState,
    O: ConnectionState,
{
    pub fn as_incoming(&self) -> Option<&I> {
        match self {
            P2pConnectionState::Incoming(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_outgoing(&self) -> Option<&O> {
        match self {
            P2pConnectionState::Outgoing(v) => Some(v),
            _ => None,
        }
    }
}

impl<I, O> ConnectionState for P2pConnectionState<I, O>
where
    I: ConnectionState,
    O: ConnectionState,
{
    fn is_success(&self) -> bool {
        match self {
            P2pConnectionState::Incoming(v) => v.is_success(),
            P2pConnectionState::Outgoing(v) => v.is_success(),
        }
    }

    fn is_error(&self) -> bool {
        match self {
            P2pConnectionState::Incoming(v) => v.is_error(),
            P2pConnectionState::Outgoing(v) => v.is_error(),
        }
    }

    fn rpc_id(&self) -> Option<RpcId> {
        match self {
            P2pConnectionState::Incoming(v) => v.rpc_id(),
            P2pConnectionState::Outgoing(v) => v.rpc_id(),
        }
    }

    fn time(&self) -> Timestamp {
        match self {
            P2pConnectionState::Incoming(i) => i.time(),
            P2pConnectionState::Outgoing(o) => o.time(),
        }
    }
}
