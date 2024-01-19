pub use crate::event_source::EventSourceService;
use crate::external_snark_worker::ExternalSnarkWorkerService;
pub use crate::p2p::channels::P2pChannelsService;
pub use crate::p2p::connection::webrtc::P2pConnectionWebRTCService;
pub use crate::p2p::connection::libp2p::P2pConnectionLibP2pService;
pub use crate::p2p::disconnection::P2pDisconnectionService;
pub use crate::recorder::Recorder;
pub use crate::rpc::RpcService;
pub use crate::snark::block_verify::SnarkBlockVerifyService;
pub use crate::snark::work_verify::SnarkWorkVerifyService;
pub use crate::snark_pool::SnarkPoolService;
pub use crate::transition_frontier::sync::ledger::snarked::TransitionFrontierSyncLedgerSnarkedService;
pub use crate::transition_frontier::sync::ledger::staged::TransitionFrontierSyncLedgerStagedService;
pub use crate::transition_frontier::TransitionFrontierService;
pub use redux::TimeService;

use crate::stats::Stats;

pub trait Service:
    TimeService
    + EventSourceService
    + SnarkBlockVerifyService
    + SnarkWorkVerifyService
    + P2pConnectionWebRTCService
    + P2pConnectionLibP2pService
    + P2pDisconnectionService
    + P2pChannelsService
    + TransitionFrontierSyncLedgerSnarkedService
    + TransitionFrontierSyncLedgerStagedService
    + TransitionFrontierService
    + SnarkPoolService
    + ExternalSnarkWorkerService
    + RpcService
{
    fn stats(&mut self) -> Option<&mut Stats>;
    fn recorder(&mut self) -> &mut Recorder;
}
