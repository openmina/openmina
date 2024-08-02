use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use mina_p2p_messages::v2::{ProverExtendBlockchainInputStableV2, StateHash};
use node::{
    block_producer::vrf_evaluator::VrfEvaluatorInput,
    core::{
        channels::mpsc,
        invariants::{InvariantService, InvariantsState},
        snark::{Snark, SnarkJobId},
    },
    event_source::Event,
    ledger::{LedgerManager, LedgerService},
    p2p::{
        connection::outgoing::P2pConnectionOutgoingInitOpts, identity::SecretKey as P2pSecretKey,
        P2pCryptoService, PeerId,
    },
    service::{
        BlockProducerService, BlockProducerVrfEvaluatorService, EventSourceService, Recorder,
        SnarkBlockVerifyService, SnarkPoolService, SnarkWorkVerifyService,
        TransitionFrontierGenesisService,
    },
    snark::{
        block_verify::{SnarkBlockVerifyId, VerifiableBlockWithHash},
        work_verify::SnarkWorkVerifyId,
        VerifierIndex, VerifierSRS,
    },
    stats::Stats,
    transition_frontier::genesis::GenesisConfig,
};
use openmina_node_common::{
    block_producer::BlockProducerService as BlockProducerServiceImpl,
    p2p::{
        webrtc::{Cmd, P2pServiceWebrtc, PeerState},
        webrtc_with_libp2p::{P2pServiceCtx, P2pServiceWebrtcWithLibp2p},
    },
    replay::ReplayerState,
    rpc::{NodeRpcRequest, RpcReceiver, RpcSender, RpcService},
    EventReceiver, EventSender, NodeServiceCommon,
};
use rand::{rngs::StdRng, SeedableRng};
use sha3::{
    digest::{ExtendableOutput, Update},
    Shake256,
};

use super::ExternalSnarkWorkerFacade;

pub struct NodeService {
    pub(super) common: NodeServiceCommon,
    pub(super) snark_worker_sender: Option<ExternalSnarkWorkerFacade>,
    pub(super) recorder: Recorder,
}

impl NodeService {
    pub fn event_sender(&self) -> &EventSender {
        self.common.event_sender()
    }

    pub fn event_receiver_with_rpc_receiver(&mut self) -> (&mut EventReceiver, &mut RpcReceiver) {
        self.common.event_receiver_with_rpc_receiver()
    }

    pub fn event_receiver(&mut self) -> &mut EventReceiver {
        self.common.event_receiver_with_rpc_receiver().0
    }

    pub fn rpc_receiver(&mut self) -> &mut RpcReceiver {
        self.common.event_receiver_with_rpc_receiver().1
    }

    pub fn rpc_sender(&self) -> RpcSender {
        self.common.rpc_sender()
    }

    pub fn process_rpc_request(&mut self, req: NodeRpcRequest) {
        self.common.process_rpc_request(req)
    }

    pub fn ledger_manager(&self) -> &LedgerManager {
        &self.common.ledger_manager
    }

    pub fn block_producer(&self) -> Option<&BlockProducerServiceImpl> {
        self.common.block_producer.as_ref()
    }

    pub fn stats(&mut self) -> Option<&mut Stats> {
        self.common.stats()
    }

    pub fn replayer(&mut self) -> Option<&mut ReplayerState> {
        self.common.replayer.as_mut()
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
            common: NodeServiceCommon {
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
                ledger_manager: LedgerManager::spawn(Default::default()),
                block_producer: None,
                p2p: P2pServiceCtx::mocked(p2p_sec_key),
                stats: Some(Stats::new()),
                rpc: RpcService::new(),
                replayer: Some(ReplayerState {
                    initial_monotonic: redux::Instant::now(),
                    initial_time,
                    expected_actions: Default::default(),
                    replay_dynamic_effects_lib: dynamic_effects_lib.unwrap_or_default(),
                }),
                invariants_state: Default::default(),
            },
            snark_worker_sender: None,
            recorder: Recorder::None,
        }
    }
}

impl AsMut<NodeServiceCommon> for NodeService {
    fn as_mut(&mut self) -> &mut NodeServiceCommon {
        &mut self.common
    }
}

impl LedgerService for NodeService {
    fn ledger_manager(&self) -> &LedgerManager {
        LedgerService::ledger_manager(&self.common)
    }

    fn force_sync_calls(&self) -> bool {
        LedgerService::force_sync_calls(&self.common)
    }
}

impl redux::TimeService for NodeService {
    fn monotonic_time(&mut self) -> redux::Instant {
        redux::TimeService::monotonic_time(&mut self.common)
    }
}

impl redux::Service for NodeService {}

impl node::Service for NodeService {
    fn stats(&mut self) -> Option<&mut Stats> {
        self.common.stats()
    }

    fn recorder(&mut self) -> &mut Recorder {
        &mut self.recorder
    }
}

impl P2pCryptoService for NodeService {
    fn generate_random_nonce(&mut self) -> [u8; 24] {
        P2pCryptoService::generate_random_nonce(&mut self.common)
    }

    fn ephemeral_sk(&mut self) -> [u8; 32] {
        P2pCryptoService::ephemeral_sk(&mut self.common)
    }

    fn static_sk(&mut self) -> [u8; 32] {
        P2pCryptoService::static_sk(&mut self.common)
    }

    fn sign_key(&mut self, key: &[u8; 32]) -> Vec<u8> {
        P2pCryptoService::sign_key(&mut self.common, key)
    }

    fn sign_publication(&mut self, publication: &[u8]) -> Vec<u8> {
        P2pCryptoService::sign_publication(&mut self.common, publication)
    }
}

impl EventSourceService for NodeService {
    fn next_event(&mut self) -> Option<Event> {
        EventSourceService::next_event(&mut self.common)
    }
}

impl P2pServiceWebrtc for NodeService {
    type Event = <NodeServiceCommon as P2pServiceWebrtc>::Event;

    fn random_pick(
        &mut self,
        list: &[P2pConnectionOutgoingInitOpts],
    ) -> P2pConnectionOutgoingInitOpts {
        P2pServiceWebrtc::random_pick(&mut self.common, list)
    }

    fn event_sender(&self) -> &mpsc::UnboundedSender<Self::Event> {
        P2pServiceWebrtc::event_sender(&self.common)
    }

    fn cmd_sender(&self) -> &mpsc::UnboundedSender<Cmd> {
        P2pServiceWebrtc::cmd_sender(&self.common)
    }

    fn peers(&mut self) -> &mut BTreeMap<PeerId, PeerState> {
        P2pServiceWebrtc::peers(&mut self.common)
    }
}

impl P2pServiceWebrtcWithLibp2p for NodeService {
    #[cfg(feature = "p2p-libp2p")]
    fn mio(&mut self) -> &mut node::p2p::service_impl::mio::MioService {
        P2pServiceWebrtcWithLibp2p::mio(&mut self.common)
    }
}

impl SnarkBlockVerifyService for NodeService {
    fn verify_init(
        &mut self,
        req_id: SnarkBlockVerifyId,
        verifier_index: Arc<VerifierIndex>,
        verifier_srs: Arc<Mutex<VerifierSRS>>,
        block: VerifiableBlockWithHash,
    ) {
        SnarkBlockVerifyService::verify_init(
            &mut self.common,
            req_id,
            verifier_index,
            verifier_srs,
            block,
        )
    }
}

impl SnarkWorkVerifyService for NodeService {
    fn verify_init(
        &mut self,
        req_id: SnarkWorkVerifyId,
        verifier_index: Arc<VerifierIndex>,
        verifier_srs: Arc<Mutex<VerifierSRS>>,
        work: Vec<Snark>,
    ) {
        SnarkWorkVerifyService::verify_init(
            &mut self.common,
            req_id,
            verifier_index,
            verifier_srs,
            work,
        )
    }
}

use node::snark::user_command_verify_effectful::SnarkUserCommandVerifyService;

impl SnarkUserCommandVerifyService for NodeService {
    fn verify_init(
        &mut self,
        _req_id: node::snark::user_command_verify::SnarkUserCommandVerifyId,
        _verifier_index: Arc<VerifierIndex>,
        _verifier_srs: Arc<Mutex<VerifierSRS>>,
        _commands: mina_p2p_messages::list::List<mina_p2p_messages::v2::MinaBaseUserCommandStableV2>,
    ) {
        todo!()
    }
}

impl SnarkPoolService for NodeService {
    fn random_choose<'a>(
        &mut self,
        iter: impl Iterator<Item = &'a SnarkJobId>,
        n: usize,
    ) -> Vec<SnarkJobId> {
        SnarkPoolService::random_choose(&mut self.common, iter, n)
    }
}

impl BlockProducerVrfEvaluatorService for NodeService {
    fn evaluate(&mut self, data: VrfEvaluatorInput) {
        BlockProducerVrfEvaluatorService::evaluate(&mut self.common, data)
    }
}

impl BlockProducerService for NodeService {
    fn prove(&mut self, block_hash: StateHash, input: Box<ProverExtendBlockchainInputStableV2>) {
        BlockProducerService::prove(&mut self.common, block_hash, input)
    }
}

impl TransitionFrontierGenesisService for NodeService {
    fn load_genesis(&mut self, config: Arc<GenesisConfig>) {
        TransitionFrontierGenesisService::load_genesis(&mut self.common, config)
    }
}

impl InvariantService for NodeService {
    fn invariants_state(&mut self) -> &mut InvariantsState {
        InvariantService::invariants_state(&mut self.common)
    }
}
