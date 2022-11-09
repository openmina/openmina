pub use redux::TimeService;

pub use crate::event_source::EventSourceService;
pub use crate::p2p::connection::P2pConnectionService;
pub use crate::rpc::RpcService;

pub trait Service: TimeService + EventSourceService + P2pConnectionService + RpcService {}
