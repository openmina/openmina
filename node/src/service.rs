pub use crate::block_producer_effectful::vrf_evaluator_effectful::BlockProducerVrfEvaluatorService;
pub use crate::block_producer_effectful::BlockProducerService;
pub use crate::event_source::EventSourceService;
pub use crate::external_snark_worker_effectful::ExternalSnarkWorkerService;
pub use crate::ledger::LedgerService;
pub use crate::p2p::service::*;
pub use crate::recorder::Recorder;
pub use crate::rpc_effectful::RpcService;
pub use crate::snark::block_verify_effectful::SnarkBlockVerifyService;
pub use crate::snark::work_verify_effectful::SnarkWorkVerifyService;
pub use crate::snark_pool::SnarkPoolService;
pub use crate::transition_frontier::archive::archive_service::ArchiveService;
pub use crate::transition_frontier::genesis_effectful::TransitionFrontierGenesisService;
pub use crate::transition_frontier::sync::ledger::snarked::TransitionFrontierSyncLedgerSnarkedService;
pub use redux::TimeService;
pub use snark::user_command_verify_effectful::SnarkUserCommandVerifyService;

use crate::stats::Stats;

pub trait Service:
    TimeService
    + EventSourceService
    + SnarkBlockVerifyService
    + SnarkWorkVerifyService
    + P2pService
    + LedgerService
    + TransitionFrontierGenesisService
    + TransitionFrontierSyncLedgerSnarkedService
    + SnarkPoolService
    + SnarkUserCommandVerifyService
    + BlockProducerVrfEvaluatorService
    + BlockProducerService
    + ExternalSnarkWorkerService
    + RpcService
    + ArchiveService
{
    fn queues(&mut self) -> Queues;
    fn stats(&mut self) -> Option<&mut Stats>;
    fn recorder(&mut self) -> &mut Recorder;
    fn is_replay(&self) -> bool;
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Queues {
    pub events: usize,
    pub snark_block_verify: usize,
    pub ledger: usize,
    pub vrf_evaluator: Option<usize>,
    pub block_prover: Option<usize>,
    pub p2p_webrtc: usize,
    #[cfg(feature = "p2p-libp2p")]
    pub p2p_libp2p: usize,
    pub rpc: usize,
}
