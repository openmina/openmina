mod rpc_service;

use std::time::Duration;
use std::{collections::BTreeMap, ffi::OsStr, sync::Arc};

use mina_p2p_messages::v2::{CurrencyFeeStableV1, NonZeroCurvePoint};
use node::core::channels::mpsc;
use node::core::requests::{PendingRequests, RequestId};
use node::core::snark::{Snark, SnarkJobId};
use node::recorder::Recorder;
use node::snark::block_verify::{
    SnarkBlockVerifyId, SnarkBlockVerifyService, VerifiableBlockWithHash,
};
use node::snark::work_verify::{SnarkWorkVerifyId, SnarkWorkVerifyService};
use node::snark::{VerifierIndex, VerifierSRS};
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
use openmina_node_native::NodeService;
use redux::Instant;

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
    // Use webrtc p2p between Rust nodes.
    rust_to_rust_use_webrtc: bool,
    monotonic_time: Instant,
    /// Events sent by the real service not yet received by state machine.
    pending_events: PendingRequests<PendingEventIdType, Event>,
    /// Once dropped, it will cause all threads associated to shutdown.
    _shutdown: mpsc::Receiver<()>,
}

impl NodeTestingService {
    pub fn new(real: NodeService, _shutdown: mpsc::Receiver<()>) -> Self {
        Self {
            real,
            rust_to_rust_use_webrtc: false,
            monotonic_time: Instant::now(),
            pending_events: PendingRequests::new(),
            _shutdown,
        }
    }

    pub fn rust_to_rust_use_webrtc(&self) -> bool {
        self.rust_to_rust_use_webrtc
    }

    pub fn set_rust_to_rust_use_webrtc(&mut self) {
        assert!(cfg!(feature = "p2p-webrtc"));
        self.rust_to_rust_use_webrtc = true;
    }

    pub fn advance_time(&mut self, by_nanos: u64) {
        self.monotonic_time += Duration::from_nanos(by_nanos);
    }

    pub fn pending_events(&mut self) -> impl Iterator<Item = (PendingEventId, &Event)> {
        while let Ok(req) = self.real.rpc.req_receiver().try_recv() {
            self.real.process_rpc_request(req);
        }
        while let Some(event) = self.real.event_receiver.try_next() {
            self.pending_events.add(event);
        }
        self.pending_events.iter()
    }

    pub async fn next_pending_event(&mut self) -> Option<(PendingEventId, &Event)> {
        tokio::select! {
            Some(rpc) = self.real.rpc.req_receiver().recv() => {
                self.real.process_rpc_request(rpc);
            }
            res = self.real.event_receiver.wait_for_events() => {
                res.ok()?;
            }
        }
        let event = self.real.event_receiver.try_next().unwrap();
        let id = self.pending_events.add(event);
        Some((id, self.pending_events.get(id).unwrap()))
    }

    pub fn take_pending_event(&mut self, id: PendingEventId) -> Option<Event> {
        self.pending_events.remove(id)
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
        self.real.find_random_peer()
    }

    fn start_discovery(&mut self, peers: Vec<P2pConnectionOutgoingInitOpts>) {
        self.real.start_discovery(peers)
    }
}

impl SnarkBlockVerifyService for NodeTestingService {
    fn verify_init(
        &mut self,
        req_id: SnarkBlockVerifyId,
        verifier_index: Arc<VerifierIndex>,
        verifier_srs: Arc<VerifierSRS>,
        block: VerifiableBlockWithHash,
    ) {
        SnarkBlockVerifyService::verify_init(
            &mut self.real,
            req_id,
            verifier_index,
            verifier_srs,
            block,
        )
    }
}

impl SnarkWorkVerifyService for NodeTestingService {
    fn verify_init(
        &mut self,
        req_id: SnarkWorkVerifyId,
        verifier_index: Arc<VerifierIndex>,
        verifier_srs: Arc<VerifierSRS>,
        work: Vec<Snark>,
    ) {
        SnarkWorkVerifyService::verify_init(
            &mut self.real,
            req_id,
            verifier_index,
            verifier_srs,
            work,
        )
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

impl ExternalSnarkWorkerService for NodeTestingService {
    fn start<P: AsRef<OsStr>>(
        &mut self,
        path: P,
        public_key: NonZeroCurvePoint,
        fee: CurrencyFeeStableV1,
    ) -> Result<(), node::external_snark_worker::ExternalSnarkWorkerError> {
        self.real.start(path, public_key, fee)
    }

    fn submit(
        &mut self,
        spec: SnarkWorkSpec,
    ) -> Result<(), node::external_snark_worker::ExternalSnarkWorkerError> {
        self.real.submit(spec)
    }

    fn cancel(&mut self) -> Result<(), node::external_snark_worker::ExternalSnarkWorkerError> {
        self.real.cancel()
    }

    fn kill(&mut self) -> Result<(), node::external_snark_worker::ExternalSnarkWorkerError> {
        self.real.kill()
    }
}
