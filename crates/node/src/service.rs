pub use crate::{
    block_producer_effectful::{
        vrf_evaluator_effectful::BlockProducerVrfEvaluatorService, BlockProducerService,
    },
    event_source::EventSourceService,
    external_snark_worker_effectful::ExternalSnarkWorkerService,
    ledger::LedgerService,
    p2p::service::*,
    recorder::Recorder,
    rpc_effectful::RpcService,
    snark::{
        block_verify_effectful::SnarkBlockVerifyService,
        work_verify_effectful::SnarkWorkVerifyService,
    },
    snark_pool::SnarkPoolService,
    transition_frontier::{
        archive::archive_service::ArchiveService,
        genesis_effectful::TransitionFrontierGenesisService,
        sync::ledger::snarked::TransitionFrontierSyncLedgerSnarkedService,
    },
};
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
