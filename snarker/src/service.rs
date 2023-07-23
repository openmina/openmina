pub use crate::event_source::EventSourceService;
pub use crate::p2p::channels::P2pChannelsService;
pub use crate::p2p::connection::P2pConnectionService;
pub use crate::p2p::disconnection::P2pDisconnectionService;
pub use crate::rpc::RpcService;
pub use crate::snark::block_verify::SnarkBlockVerifyService;
pub use crate::transition_frontier::sync::ledger::snarked::TransitionFrontierSyncLedgerSnarkedService;
pub use crate::transition_frontier::sync::ledger::staged::TransitionFrontierSyncLedgerStagedService;
pub use crate::transition_frontier::TransitionFrontierService;
pub use redux::TimeService;

use crate::stats::Stats;

pub trait Service:
    TimeService
    + EventSourceService
    + SnarkBlockVerifyService
    + P2pConnectionService
    + P2pDisconnectionService
    + P2pChannelsService
    + TransitionFrontierSyncLedgerSnarkedService
    + TransitionFrontierSyncLedgerStagedService
    + TransitionFrontierService
    + RpcService
{
    fn stats(&mut self) -> Option<&mut Stats>;
}
