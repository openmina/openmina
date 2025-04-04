use std::sync::Arc;

use node::{
    core::{channels::mpsc, invariants::InvariantsState},
    event_source::Event,
    ledger::LedgerManager,
    p2p::identity::SecretKey as P2pSecretKey,
    service::Recorder,
    stats::Stats,
    transition_frontier::genesis::GenesisConfig,
};
use rand::{rngs::StdRng, SeedableRng};
use sha3::{
    digest::{core_api::XofReaderCoreWrapper, ExtendableOutput, Update},
    Shake256, Shake256ReaderCore,
};

use crate::rpc::RpcReceiver;

use super::{
    archive::ArchiveService,
    block_producer::BlockProducerService,
    p2p::webrtc_with_libp2p::P2pServiceCtx,
    replay::ReplayerState,
    rpc::{RpcSender, RpcService},
    snark_worker::SnarkWorker,
    snarks::SnarkBlockVerifyArgs,
    EventReceiver, EventSender,
};

pub struct NodeService {
    pub rng_seed: [u8; 32],
    pub rng_ephemeral: XofReaderCoreWrapper<Shake256ReaderCore>,
    pub rng_static: XofReaderCoreWrapper<Shake256ReaderCore>,
    pub rng: StdRng,

    /// Events sent on this channel are retrieved and processed in the
    /// `event_source` state machine defined in the `openmina-node` crate.
    pub event_sender: EventSender,
    pub event_receiver: EventReceiver,

    pub snark_block_proof_verify: mpsc::TrackedUnboundedSender<SnarkBlockVerifyArgs>,

    pub ledger_manager: LedgerManager,
    pub snark_worker: Option<SnarkWorker>,
    pub block_producer: Option<BlockProducerService>,
    pub archive: Option<ArchiveService>,
    pub p2p: P2pServiceCtx,

    pub stats: Option<Stats>,
    pub rpc: RpcService,
    pub recorder: Recorder,
    pub replayer: Option<ReplayerState>,
    pub invariants_state: InvariantsState,
}

impl NodeService {
    pub fn event_sender(&self) -> &EventSender {
        &self.event_sender
    }

    pub fn rpc_sender(&self) -> RpcSender {
        self.rpc.req_sender()
    }

    pub fn event_receiver_with_rpc_receiver(&mut self) -> (&mut EventReceiver, &mut RpcReceiver) {
        (&mut self.event_receiver, self.rpc.req_receiver())
    }

    pub fn event_receiver(&mut self) -> &mut EventReceiver {
        &mut self.event_receiver
    }

    pub fn rpc_receiver(&mut self) -> &mut RpcReceiver {
        self.rpc.req_receiver()
    }

    pub fn ledger_manager(&self) -> &LedgerManager {
        &self.ledger_manager
    }

    pub fn block_producer(&self) -> Option<&BlockProducerService> {
        self.block_producer.as_ref()
    }

    pub fn archive(&self) -> Option<&ArchiveService> {
        self.archive.as_ref()
    }

    pub fn stats(&mut self) -> Option<&mut Stats> {
        self.stats.as_mut()
    }

    pub fn replayer(&mut self) -> Option<&mut ReplayerState> {
        self.replayer.as_mut()
    }
}

impl NodeService {
    pub fn for_replay(
        rng_seed: [u8; 32],
        initial_time: redux::Timestamp,
        p2p_sec_key: P2pSecretKey,
        dynamic_effects_lib: Option<String>,
    ) -> Self {
        Self {
            rng_seed,
            rng_ephemeral: Shake256::default()
                .chain(rng_seed)
                .chain(b"ephemeral")
                .finalize_xof(),
            rng_static: Shake256::default()
                .chain(rng_seed)
                .chain(b"static")
                .finalize_xof(),
            rng: StdRng::from_seed(rng_seed),
            event_sender: mpsc::unbounded_channel().0,
            event_receiver: mpsc::unbounded_channel().1.into(),
            snark_block_proof_verify: mpsc::unbounded_channel().0,
            ledger_manager: LedgerManager::spawn(Default::default()),
            snark_worker: None,
            block_producer: None,
            archive: None,
            p2p: P2pServiceCtx::mocked(p2p_sec_key),
            stats: Some(Stats::new()),
            rpc: RpcService::new(),
            recorder: Recorder::None,
            replayer: Some(ReplayerState {
                initial_monotonic: redux::Instant::now(),
                initial_time,
                expected_actions: Default::default(),
                replay_dynamic_effects_lib: dynamic_effects_lib.unwrap_or_default(),
            }),
            invariants_state: Default::default(),
        }
    }
}

impl AsMut<NodeService> for NodeService {
    fn as_mut(&mut self) -> &mut NodeService {
        self
    }
}

impl redux::Service for NodeService {}

impl node::Service for NodeService {
    fn queues(&mut self) -> node::service::Queues {
        node::service::Queues {
            events: self.event_receiver.len(),
            snark_block_verify: self.snark_block_proof_verify.len(),
            ledger: self.ledger_manager.pending_calls(),
            vrf_evaluator: self
                .block_producer
                .as_ref()
                .map(|v| v.vrf_pending_requests()),
            block_prover: self
                .block_producer
                .as_ref()
                .map(|v| v.prove_pending_requests()),
            p2p_webrtc: self.p2p.webrtc.pending_cmds(),
            #[cfg(feature = "p2p-libp2p")]
            p2p_libp2p: self.p2p.mio.pending_cmds(),
            rpc: self.rpc.req_receiver().len(),
        }
    }

    fn stats(&mut self) -> Option<&mut Stats> {
        self.stats()
    }

    fn recorder(&mut self) -> &mut Recorder {
        &mut self.recorder
    }

    fn is_replay(&self) -> bool {
        self.replayer.is_some()
    }
}

impl redux::TimeService for NodeService {
    fn monotonic_time(&mut self) -> redux::Instant {
        self.replayer
            .as_ref()
            .map(|v| v.next_monotonic_time())
            .unwrap_or_else(redux::Instant::now)
    }
}

impl node::service::EventSourceService for NodeService {
    fn next_event(&mut self) -> Option<Event> {
        self.event_receiver.try_next()
    }
}

impl node::service::LedgerService for NodeService {
    fn ledger_manager(&self) -> &LedgerManager {
        &self.ledger_manager
    }

    fn force_sync_calls(&self) -> bool {
        self.replayer.is_some()
    }
}

impl node::service::TransitionFrontierGenesisService for NodeService {
    fn load_genesis(&mut self, config: Arc<GenesisConfig>) {
        let res = match config.load() {
            Err(err) => Err(err.to_string()),
            Ok((masks, data)) => {
                let is_archive = self.archive().is_some();
                masks.into_iter().for_each(|mut mask| {
                    if !is_archive {
                        // Optimization: We don't need token owners if the node is not an archive
                        mask.unset_token_owners();
                    }
                    self.ledger_manager.insert_genesis_ledger(mask);
                });
                Ok(data)
            }
        };
        let _ = self.event_sender.send(Event::GenesisLoad(res));
    }
}

impl node::core::invariants::InvariantService for NodeService {
    type ClusterInvariantsState<'a> = std::cell::RefMut<'a, InvariantsState>;

    fn invariants_state(&mut self) -> &mut InvariantsState {
        &mut self.invariants_state
    }
}
