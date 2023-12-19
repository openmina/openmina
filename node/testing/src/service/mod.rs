mod rpc_service;

use std::sync::Mutex;
use std::time::Duration;
use std::{collections::BTreeMap, ffi::OsStr, sync::Arc};

use ledger::dummy::dummy_transaction_proof;
use ledger::scan_state::scan_state::transaction_snark::SokMessage;
use ledger::Mask;
use mina_p2p_messages::v2::{
    CurrencyFeeStableV1, LedgerHash, LedgerProofProdStableV2, MinaBaseZkappAccountZkappUriStableV1,
    MinaStateSnarkedLedgerStateWithSokStableV2, NonZeroCurvePoint,
    SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Single, TransactionSnarkStableV2,
    TransactionSnarkWorkTStableV2Proofs,
};
use node::account::AccountPublicKey;
use node::block_producer::vrf_evaluator::VrfEvaluatorInput;
use node::core::channels::mpsc;
use node::core::requests::{PendingRequests, RequestId};
use node::core::snark::{Snark, SnarkJobId};
use node::external_snark_worker::ExternalSnarkWorkerEvent;
use node::recorder::Recorder;
use node::service::BlockProducerVrfEvaluatorService;
use node::snark::block_verify::{
    SnarkBlockVerifyId, SnarkBlockVerifyService, VerifiableBlockWithHash,
};
use node::snark::work_verify::{SnarkWorkVerifyId, SnarkWorkVerifyService};
use node::snark::{SnarkEvent, VerifierIndex, VerifierSRS};
use node::snark_pool::{JobState, SnarkPoolService};
use node::stats::Stats;
use node::{
    event_source::Event,
    external_snark_worker::{ExternalSnarkWorkerService, SnarkWorkSpec},
    ledger::LedgerCtx,
    p2p::{
        connection::outgoing::P2pConnectionOutgoingInitOpts,
        service_impl::{
            libp2p::Libp2pService,
            webrtc::{Cmd, P2pServiceWebrtc, PeerState},
            webrtc_with_libp2p::P2pServiceWebrtcWithLibp2p,
        },
        webrtc, PeerId,
    },
};
use node::{ActionWithMeta, State};
use openmina_node_native::NodeService;
use redux::Instant;

use crate::cluster::ClusterNodeId;
use crate::node::NonDeterministicEvent;

pub type DynEffects = Box<dyn FnMut(&State, &NodeTestingService, &ActionWithMeta) + Send>;

#[derive(Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct PendingEventIdType;
impl openmina_core::requests::RequestIdType for PendingEventIdType {
    fn request_id_type() -> &'static str {
        "PendingEventId"
    }
}
pub type PendingEventId = RequestId<PendingEventIdType>;

pub struct NodeTestingService {
    real: NodeService,
    id: ClusterNodeId,
    /// Use webrtc p2p between Rust nodes.
    rust_to_rust_use_webrtc: bool,
    /// We are replaying this node so disable some non-deterministic services.
    is_replay: bool,
    monotonic_time: Instant,
    /// Events sent by the real service not yet received by state machine.
    pending_events: PendingRequests<PendingEventIdType, Event>,
    dyn_effects: Option<DynEffects>,

    snarker_sok_digest: Option<MinaBaseZkappAccountZkappUriStableV1>,
    /// Once dropped, it will cause all threads associated to shutdown.
    _shutdown: mpsc::Receiver<()>,
}

impl NodeTestingService {
    pub fn new(real: NodeService, id: ClusterNodeId, _shutdown: mpsc::Receiver<()>) -> Self {
        Self {
            real,
            id,
            rust_to_rust_use_webrtc: false,
            is_replay: false,
            monotonic_time: Instant::now(),
            pending_events: PendingRequests::new(),
            dyn_effects: None,
            snarker_sok_digest: None,
            _shutdown,
        }
    }

    pub fn node_id(&self) -> ClusterNodeId {
        self.id
    }

    pub fn rust_to_rust_use_webrtc(&self) -> bool {
        self.rust_to_rust_use_webrtc
    }

    pub fn set_rust_to_rust_use_webrtc(&mut self) -> &mut Self {
        assert!(cfg!(feature = "p2p-webrtc"));
        self.rust_to_rust_use_webrtc = true;
        self
    }

    pub fn set_replay(&mut self) -> &mut Self {
        self.is_replay = true;
        self
    }

    pub fn advance_time(&mut self, by_nanos: u64) {
        self.monotonic_time += Duration::from_nanos(by_nanos);
    }

    pub fn dyn_effects(&mut self, state: &State, action: &ActionWithMeta) {
        if let Some(mut dyn_effects) = self.dyn_effects.take() {
            (dyn_effects)(state, self, action);
            self.dyn_effects = Some(dyn_effects);
        }
    }

    pub fn set_dyn_effects(&mut self, effects: DynEffects) {
        self.dyn_effects = Some(effects);
    }

    pub fn remove_dyn_effects(&mut self) -> Option<DynEffects> {
        self.dyn_effects.take()
    }

    pub fn set_snarker_sok_digest(&mut self, digest: MinaBaseZkappAccountZkappUriStableV1) {
        self.snarker_sok_digest = Some(digest);
    }

    pub fn pending_events(&mut self) -> impl Iterator<Item = (PendingEventId, &Event)> {
        while let Ok(req) = self.real.rpc.req_receiver().try_recv() {
            self.real.process_rpc_request(req);
        }
        while let Some(event) = self.real.event_receiver.try_next() {
            // Drop non-deterministic events during replay. We
            // have those recorded as `ScenarioStep::NonDeterministicEvent`.
            if self.is_replay && NonDeterministicEvent::should_drop_event(&event) {
                eprintln!("dropping non-deterministic event: {event:?}");
                continue;
            }
            self.pending_events.add(event);
        }
        self.pending_events.iter()
    }

    pub async fn next_pending_event(&mut self) -> Option<(PendingEventId, &Event)> {
        let event = loop {
            tokio::select! {
                Some(rpc) = self.real.rpc.req_receiver().recv() => {
                    self.real.process_rpc_request(rpc);
                    break self.real.event_receiver.try_next().unwrap();
                }
                res = self.real.event_receiver.wait_for_events() => {
                    res.ok()?;
                    let event = self.real.event_receiver.try_next().unwrap();
                    // Drop non-deterministic events during replay. We
                    // have those recorded as `ScenarioStep::NonDeterministicEvent`.
                    if self.is_replay && NonDeterministicEvent::should_drop_event(&event) {
                        eprintln!("dropping non-deterministic event: {event:?}");
                        continue;
                    }
                    break event;
                }
            }
        };
        let id = self.pending_events.add(event);
        Some((id, self.pending_events.get(id).unwrap()))
    }

    pub fn get_pending_event(&self, id: PendingEventId) -> Option<&Event> {
        self.pending_events.get(id)
    }

    pub fn take_pending_event(&mut self, id: PendingEventId) -> Option<Event> {
        self.pending_events.remove(id)
    }

    pub fn ledger(&self, ledger_hash: &LedgerHash) -> Option<Mask> {
        self.real.ledger.mask(ledger_hash).map(|(mask, _)| mask)
    }
}

impl redux::Service for NodeTestingService {}

impl node::Service for NodeTestingService {
    fn stats(&mut self) -> Option<&mut Stats> {
        self.real.stats()
    }

    fn recorder(&mut self) -> &mut Recorder {
        self.real.recorder()
    }
}

impl node::ledger::LedgerService for NodeTestingService {
    fn ctx(&self) -> &LedgerCtx {
        &self.real.ledger
    }

    fn ctx_mut(&mut self) -> &mut LedgerCtx {
        &mut self.real.ledger
    }
}

impl redux::TimeService for NodeTestingService {
    fn monotonic_time(&mut self) -> Instant {
        self.monotonic_time
    }
}

impl node::event_source::EventSourceService for NodeTestingService {
    fn next_event(&mut self) -> Option<Event> {
        None
    }
}

impl P2pServiceWebrtc for NodeTestingService {
    type Event = Event;

    fn random_pick(
        &mut self,
        list: &[P2pConnectionOutgoingInitOpts],
    ) -> P2pConnectionOutgoingInitOpts {
        self.real.random_pick(list)
    }

    fn event_sender(&mut self) -> &mut mpsc::UnboundedSender<Event> {
        &mut self.real.event_sender
    }

    fn cmd_sender(&mut self) -> &mut mpsc::UnboundedSender<Cmd> {
        &mut self.real.cmd_sender
    }

    fn peers(&mut self) -> &mut BTreeMap<PeerId, PeerState> {
        &mut self.real.peers
    }

    fn outgoing_init(&mut self, peer_id: PeerId) {
        self.real.outgoing_init(peer_id);
    }

    fn incoming_init(&mut self, peer_id: PeerId, offer: webrtc::Offer) {
        self.real.incoming_init(peer_id, offer);
    }
}

impl P2pServiceWebrtcWithLibp2p for NodeTestingService {
    fn libp2p(&mut self) -> &mut Libp2pService {
        &mut self.real.libp2p
    }

    fn find_random_peer(&mut self) {
        use node::p2p::identity::SecretKey as P2pSecretKey;
        use node::p2p::service_impl::libp2p::Cmd;

        if self.is_replay {
            return;
        }

        let secret_key = P2pSecretKey::from_bytes({
            let mut bytes = [1; 32];
            let bytes_len = bytes.len();
            let i_bytes = self.id.index().to_be_bytes();
            let i = bytes_len - i_bytes.len();
            bytes[i..bytes_len].copy_from_slice(&i_bytes);
            bytes
        });
        let peer_id = secret_key.public_key().peer_id();

        self.libp2p()
            .cmd_sender()
            .send(Cmd::FindNode(peer_id.into()))
            .unwrap_or_default();
    }

    fn start_discovery(&mut self, peers: Vec<P2pConnectionOutgoingInitOpts>) {
        if self.is_replay {
            return;
        }
        self.real.start_discovery(peers)
    }
}

impl SnarkBlockVerifyService for NodeTestingService {
    fn verify_init(
        &mut self,
        req_id: SnarkBlockVerifyId,
        verifier_index: Arc<VerifierIndex>,
        verifier_srs: Arc<Mutex<VerifierSRS>>,
        block: VerifiableBlockWithHash,
    ) {
        let _ = (verifier_index, verifier_srs, block);
        let _ = self
            .real
            .event_sender
            .send(SnarkEvent::BlockVerify(req_id, Ok(())).into());
        // SnarkBlockVerifyService::verify_init(
        //     &mut self.real,
        //     req_id,
        //     verifier_index,
        //     verifier_srs,
        //     block,
        // )
    }
}

impl SnarkWorkVerifyService for NodeTestingService {
    fn verify_init(
        &mut self,
        req_id: SnarkWorkVerifyId,
        verifier_index: Arc<VerifierIndex>,
        verifier_srs: Arc<Mutex<VerifierSRS>>,
        work: Vec<Snark>,
    ) {
        let _ = (verifier_index, verifier_srs, work);
        let _ = self
            .real
            .event_sender
            .send(SnarkEvent::WorkVerify(req_id, Ok(())).into());
        // SnarkWorkVerifyService::verify_init(
        //     &mut self.real,
        //     req_id,
        //     verifier_index,
        //     verifier_srs,
        //     work,
        // )
    }
}

impl SnarkPoolService for NodeTestingService {
    fn random_choose<'a>(
        &mut self,
        iter: impl Iterator<Item = &'a JobState>,
        n: usize,
    ) -> Vec<SnarkJobId> {
        self.real.random_choose(iter, n)
    }
}

impl BlockProducerVrfEvaluatorService for NodeTestingService {
    fn evaluate(&mut self, data: VrfEvaluatorInput) {
        BlockProducerVrfEvaluatorService::evaluate(&mut self.real, data)
    }
}

impl ExternalSnarkWorkerService for NodeTestingService {
    fn start<P: AsRef<OsStr>>(
        &mut self,
        path: P,
        public_key: NonZeroCurvePoint,
        fee: CurrencyFeeStableV1,
    ) -> Result<(), node::external_snark_worker::ExternalSnarkWorkerError> {
        let _ = path;

        let pub_key = AccountPublicKey::from(public_key);
        let sok_message = SokMessage::create((&fee).into(), pub_key.into());
        self.set_snarker_sok_digest((&sok_message.digest()).into());
        let _ = self
            .real
            .event_sender
            .send(ExternalSnarkWorkerEvent::Started.into());
        Ok(())
        // self.real.start(path, public_key, fee)
    }

    fn submit(
        &mut self,
        spec: SnarkWorkSpec,
    ) -> Result<(), node::external_snark_worker::ExternalSnarkWorkerError> {
        let sok_digest = self.snarker_sok_digest.clone().unwrap();
        let make_dummy_proof = |spec| {
            let statement = match spec {
                SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Single::Transition(v, _) => v.0,
                SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponseA0Single::Merge(v) => v.0 .0,
            };

            LedgerProofProdStableV2(TransactionSnarkStableV2 {
                statement: MinaStateSnarkedLedgerStateWithSokStableV2 {
                    source: statement.source,
                    target: statement.target,
                    connecting_ledger_left: statement.connecting_ledger_left,
                    connecting_ledger_right: statement.connecting_ledger_right,
                    supply_increase: statement.supply_increase,
                    fee_excess: statement.fee_excess,
                    sok_digest: sok_digest.clone(),
                },
                proof: (*dummy_transaction_proof()).clone(),
            })
        };
        let res = match spec {
            SnarkWorkSpec::One(v) => TransactionSnarkWorkTStableV2Proofs::One(make_dummy_proof(v)),
            SnarkWorkSpec::Two((v1, v2)) => TransactionSnarkWorkTStableV2Proofs::Two((
                make_dummy_proof(v1),
                make_dummy_proof(v2),
            )),
        };
        let _ = self
            .real
            .event_sender
            .send(ExternalSnarkWorkerEvent::WorkResult(Arc::new(res)).into());
        Ok(())
        // self.real.submit(spec)
    }

    fn cancel(&mut self) -> Result<(), node::external_snark_worker::ExternalSnarkWorkerError> {
        let _ = self
            .real
            .event_sender
            .send(ExternalSnarkWorkerEvent::WorkCancelled.into());
        Ok(())
        // self.real.cancel()
    }

    fn kill(&mut self) -> Result<(), node::external_snark_worker::ExternalSnarkWorkerError> {
        let _ = self
            .real
            .event_sender
            .send(ExternalSnarkWorkerEvent::Killed.into());
        Ok(())
        // self.real.kill()
    }
}

impl node::core::invariants::InvariantService for NodeTestingService {
    fn invariants_state(&mut self) -> &mut openmina_core::invariants::InvariantsState {
        &mut self.real.invariants_state
    }
}
