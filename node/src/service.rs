use crate::block_producer::vrf_evaluator::BlockProducerVrfEvaluatorLedgerService;
pub use crate::block_producer::vrf_evaluator::BlockProducerVrfEvaluatorService;
pub use crate::block_producer::BlockProducerService;
pub use crate::event_source::EventSourceService;
use crate::external_snark_worker::ExternalSnarkWorkerService;
pub use crate::p2p::channels::P2pChannelsService;
pub use crate::p2p::connection::P2pConnectionService;
pub use crate::p2p::disconnection::P2pDisconnectionService;
use crate::p2p::P2pCryptoService;
use crate::p2p::P2pMioService;
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
    + P2pConnectionService
    + P2pDisconnectionService
    + P2pChannelsService
    + P2pMioService
    + P2pCryptoService
    + TransitionFrontierSyncLedgerSnarkedService
    + TransitionFrontierSyncLedgerStagedService
    + TransitionFrontierService
    + SnarkPoolService
    + BlockProducerVrfEvaluatorService
    + BlockProducerService
    + BlockProducerVrfEvaluatorLedgerService
    + ExternalSnarkWorkerService
    + RpcService
{
    fn stats(&mut self) -> Option<&mut Stats>;
    fn recorder(&mut self) -> &mut Recorder;
}
