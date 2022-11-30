pub use redux::TimeService;

pub use crate::event_source::EventSourceService;
pub use crate::p2p::connection::P2pConnectionService;
pub use crate::p2p::pubsub::P2pPubsubService;
pub use crate::p2p::rpc::P2pRpcService;
pub use crate::rpc::RpcService;
pub use crate::snark::block_verify::SnarkBlockVerifyService;

pub trait Service:
    TimeService
    + EventSourceService
    + P2pConnectionService
    + P2pPubsubService
    + P2pRpcService
    + SnarkBlockVerifyService
    + RpcService
{
}
