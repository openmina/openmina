use std::collections::{BTreeMap, VecDeque};

use std::sync::{Arc, Mutex};

use ledger::scan_state::scan_state::transaction_snark::{SokDigest, Statement};
use libp2p_identity::Keypair;
use mina_p2p_messages::v2::{LedgerProofProdStableV2, TransactionSnarkWorkTStableV2Proofs};
#[cfg(feature = "p2p-libp2p")]
use node::p2p::service_impl::mio::MioService;
use node::p2p::service_impl::services::NativeP2pNetworkService;
use rand::prelude::*;
use redux::ActionMeta;
use serde::Serialize;

use node::core::channels::{mpsc, oneshot};
use node::core::invariants::InvariantsState;
use node::core::snark::{Snark, SnarkJobId};
use node::event_source::Event;
use node::ledger::ledger_manager::LedgerManager;
use node::ledger::LedgerService;
use node::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
use node::p2p::service_impl::webrtc::{Cmd, P2pServiceWebrtc, PeerState};
use node::p2p::service_impl::webrtc_with_libp2p::P2pServiceWebrtcWithLibp2p;
use node::p2p::service_impl::TaskSpawner;
use node::p2p::{P2pCryptoService, P2pNetworkService, P2pNetworkServiceError, PeerId};
use node::rpc::{RpcP2pConnectionOutgoingResponse, RpcRequest};
use node::service::{EventSourceService, Recorder, TransitionFrontierGenesisService};
use node::snark::block_verify::{
    SnarkBlockVerifyError, SnarkBlockVerifyId, SnarkBlockVerifyService, VerifiableBlockWithHash,
};
use node::snark::work_verify::{SnarkWorkVerifyError, SnarkWorkVerifyId, SnarkWorkVerifyService};
use node::snark::{SnarkEvent, VerifierIndex, VerifierSRS};
use node::snark_pool::SnarkPoolService;
use node::stats::Stats;
use node::transition_frontier::genesis::GenesisConfig;
use node::ActionKind;

use crate::block_producer::BlockProducerService;
use crate::ext_snark_worker;
use crate::rpc::RpcService;

pub struct NodeService {
    pub rng: StdRng,
    /// Events sent on this channel are retrieved and processed in the
    /// `event_source` state machine defined in the `openmina-node` crate.
    pub event_sender: mpsc::UnboundedSender<Event>,
    pub event_receiver: EventReceiver,
    pub cmd_sender: mpsc::UnboundedSender<Cmd>,
    pub ledger_manager: LedgerManager,
    pub peers: BTreeMap<PeerId, PeerState>,
    #[cfg(feature = "p2p-libp2p")]
    pub mio: MioService,
    pub network: NativeP2pNetworkService,
    pub block_producer: Option<BlockProducerService>,
    pub keypair: Keypair,
    pub snark_worker_sender: Option<ext_snark_worker::ExternalSnarkWorkerFacade>,
    pub rpc: RpcService,
    pub stats: Stats,
    pub recorder: Recorder,
    pub replayer: Option<ReplayerState>,
    pub invariants_state: InvariantsState,
}

pub struct ReplayerState {
    pub initial_monotonic: redux::Instant,
    pub initial_time: redux::Timestamp,
    pub expected_actions: VecDeque<(ActionKind, ActionMeta)>,
    pub replay_dynamic_effects_lib: String,
}

impl ReplayerState {
    pub fn next_monotonic_time(&self) -> redux::Instant {
        self.expected_actions
            .front()
            .map(|(_, meta)| meta.time())
            .map(|expected_time| {
                let time_passed = expected_time.checked_sub(self.initial_time).unwrap();
                self.initial_monotonic + time_passed
            })
            .unwrap_or(self.initial_monotonic)
    }
}

impl LedgerService for NodeService {
    fn ledger_manager(&self) -> &LedgerManager {
        &self.ledger_manager
    }

    fn force_sync_calls(&self) -> bool {
        self.replayer.is_some()
    }
}

impl redux::TimeService for NodeService {
    fn monotonic_time(&mut self) -> redux::Instant {
        self.replayer
            .as_ref()
            .map(|v| v.next_monotonic_time())
            .unwrap_or_else(|| redux::Instant::now())
    }
}

impl redux::Service for NodeService {}

impl node::Service for NodeService {
    fn stats(&mut self) -> Option<&mut Stats> {
        Some(&mut self.stats)
    }

    fn recorder(&mut self) -> &mut Recorder {
        &mut self.recorder
    }
}

impl P2pCryptoService for NodeService {
    fn generate_random_nonce(&mut self) -> [u8; 24] {
        self.rng.gen()
    }

    fn ephemeral_sk(&mut self) -> [u8; 32] {
        // TODO: make deterministic
        // TODO: make network debugger to use seed to derive the same key
        //let mut r = [0; 32];
        //getrandom::getrandom(&mut r).unwrap();
        //r
        self.rng.gen()
    }

    fn static_sk(&mut self) -> [u8; 32] {
        // TODO: make deterministic
        // TODO: make network debugger to use seed to derive the same key
        //let mut r = [0; 32];
        //getrandom::getrandom(&mut r).unwrap();
        //r
        self.rng.gen()
    }

    fn sign_key(&mut self, key: &[u8; 32]) -> Vec<u8> {
        // TODO: make deterministic
        let msg = [b"noise-libp2p-static-key:", key.as_slice()].concat();
        let sig = self.keypair.sign(&msg).expect("unable to create signature");

        let mut payload = vec![];
        payload.extend_from_slice(b"\x0a\x24");
        payload.extend_from_slice(&self.keypair.public().encode_protobuf());
        payload.extend_from_slice(b"\x12\x40");
        payload.extend_from_slice(&sig);
        payload
    }

    fn sign_publication(&mut self, publication: &[u8]) -> Vec<u8> {
        let msg = [b"libp2p-pubsub:", publication].concat();
        self.keypair.sign(&msg).expect("unable to create signature")
    }
}

impl P2pNetworkService for NodeService {
    fn resolve_name(
        &mut self,
        host: &str,
    ) -> Result<Vec<std::net::IpAddr>, P2pNetworkServiceError> {
        self.network.resolve_name(host)
    }

    fn detect_local_ip(&mut self) -> Result<Vec<std::net::IpAddr>, P2pNetworkServiceError> {
        self.network.detect_local_ip()
    }
}

impl EventSourceService for NodeService {
    fn next_event(&mut self) -> Option<Event> {
        self.event_receiver.try_next()
    }
}

impl P2pServiceWebrtc for NodeService {
    type Event = Event;

    fn random_pick(
        &mut self,
        list: &[P2pConnectionOutgoingInitOpts],
    ) -> P2pConnectionOutgoingInitOpts {
        list.choose(&mut self.rng).unwrap().clone()
    }

    fn event_sender(&mut self) -> &mut mpsc::UnboundedSender<Self::Event> {
        &mut self.event_sender
    }

    fn cmd_sender(&mut self) -> &mut mpsc::UnboundedSender<Cmd> {
        &mut self.cmd_sender
    }

    fn peers(&mut self) -> &mut BTreeMap<PeerId, PeerState> {
        &mut self.peers
    }
}

#[cfg(feature = "p2p-libp2p")]
impl P2pServiceWebrtcWithLibp2p for NodeService {
    fn mio(&mut self) -> &mut MioService {
        &mut self.mio
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
        if self.replayer.is_some() {
            return;
        }
        let tx = self.event_sender.clone();
        eprintln!("rayon::spawn_fifo");
        std::thread::spawn(move || {
            eprintln!("verify({}) - start", block.hash_ref());
            let header = block.header_ref();
            let result = {
                let verifier_srs = verifier_srs.lock().expect("Failed to lock the SRS");
                if !ledger::proofs::verification::verify_block(
                    header,
                    &verifier_index,
                    &verifier_srs,
                ) {
                    Err(SnarkBlockVerifyError::VerificationFailed)
                } else {
                    Ok(())
                }
            };
            eprintln!("verify({}) - end", block.hash_ref());

            let _ = tx.send(SnarkEvent::BlockVerify(req_id, result).into());
        });
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
        if self.replayer.is_some() {
            return;
        }
        let tx = self.event_sender.clone();
        rayon::spawn_fifo(move || {
            let result = {
                let conv = |proof: &LedgerProofProdStableV2| {
                    (
                        Statement::<SokDigest>::from(&proof.0.statement),
                        proof.proof.clone(),
                    )
                };
                let works = work
                    .into_iter()
                    .flat_map(|work| match &*work.proofs {
                        TransactionSnarkWorkTStableV2Proofs::One(v) => [Some(conv(v)), None],
                        TransactionSnarkWorkTStableV2Proofs::Two((v1, v2)) => {
                            [Some(conv(v1)), Some(conv(v2))]
                        }
                    })
                    .filter_map(|v| v)
                    .collect::<Vec<_>>();
                let verifier_srs = verifier_srs.lock().expect("Failed to lock SRS");
                if !ledger::proofs::verification::verify_transaction(
                    works.iter().map(|(v1, v2)| (v1, v2)),
                    &verifier_index,
                    &verifier_srs,
                ) {
                    Err(SnarkWorkVerifyError::VerificationFailed)
                } else {
                    Ok(())
                }
            };

            let _ = tx.send(SnarkEvent::WorkVerify(req_id, result).into());
        });
    }
}

impl SnarkPoolService for NodeService {
    fn random_choose<'a>(
        &mut self,
        iter: impl Iterator<Item = &'a SnarkJobId>,
        n: usize,
    ) -> Vec<SnarkJobId> {
        iter.choose_multiple(&mut self.rng, n)
            .into_iter()
            .map(|job_id| job_id.clone())
            .collect()
    }
}

impl TransitionFrontierGenesisService for NodeService {
    fn load_genesis(&mut self, config: Arc<GenesisConfig>) {
        let res = match config.load() {
            Err(err) => Err(err.to_string()),
            Ok((mask, data)) => {
                self.ledger_manager.insert_genesis_ledger(mask);
                Ok(data)
            }
        };
        let _ = self.event_sender.send(Event::GenesisLoad(res));
    }
}

pub struct EventReceiver {
    rx: mpsc::UnboundedReceiver<Event>,
    queue: Vec<Event>,
}

impl EventReceiver {
    /// If `Err(())`, `mpsc::Sender` for this channel was dropped.
    pub async fn wait_for_events(&mut self) -> Result<(), ()> {
        if !self.queue.is_empty() {
            return Ok(());
        }
        let next = self.rx.recv().await.ok_or(())?;
        self.queue.push(next);
        Ok(())
    }

    pub fn has_next(&mut self) -> bool {
        if self.queue.is_empty() {
            if let Some(event) = self.try_next() {
                self.queue.push(event);
                true
            } else {
                false
            }
        } else {
            true
        }
    }

    pub fn try_next(&mut self) -> Option<Event> {
        if !self.queue.is_empty() {
            Some(self.queue.remove(0))
        } else {
            self.rx.try_recv().ok()
        }
    }
}

impl From<mpsc::UnboundedReceiver<Event>> for EventReceiver {
    fn from(rx: mpsc::UnboundedReceiver<Event>) -> Self {
        Self {
            rx,
            queue: Vec::with_capacity(1),
        }
    }
}

pub struct NodeRpcRequest {
    pub req: RpcRequest,
    pub responder: Box<dyn Send + std::any::Any>,
}

#[derive(Clone)]
pub struct RpcSender {
    tx: mpsc::Sender<NodeRpcRequest>,
}

impl RpcSender {
    pub fn new(tx: mpsc::Sender<NodeRpcRequest>) -> Self {
        Self { tx }
    }

    pub async fn oneshot_request<T>(&self, req: RpcRequest) -> Option<T>
    where
        T: 'static + Send + Serialize,
    {
        let (tx, rx) = oneshot::channel::<T>();
        let responder = Box::new(tx);
        let sender = self.tx.clone();
        let _ = sender.send(NodeRpcRequest { req, responder }).await;

        rx.await.ok()
    }

    pub async fn multishot_request<T>(
        &self,
        expected_messages: usize,
        req: RpcRequest,
    ) -> mpsc::Receiver<T>
    where
        T: 'static + Send + Serialize,
    {
        let (tx, rx) = mpsc::channel::<T>(expected_messages);
        let responder = Box::new(tx);
        let sender = self.tx.clone();
        let _ = sender.send(NodeRpcRequest { req, responder }).await;

        rx
    }

    pub async fn peer_connect(
        &self,
        opts: P2pConnectionOutgoingInitOpts,
    ) -> Result<String, String> {
        let peer_id = opts.peer_id().to_string();
        let req = RpcRequest::P2pConnectionOutgoing(opts);
        self.oneshot_request::<RpcP2pConnectionOutgoingResponse>(req)
            .await
            .ok_or_else(|| "state machine shut down".to_owned())??;

        Ok(peer_id)
    }
}

#[derive(Clone)]
pub struct P2pTaskSpawner {}

impl TaskSpawner for P2pTaskSpawner {
    fn spawn_main<F>(&self, name: &str, fut: F)
    where
        F: 'static + Send + std::future::Future,
    {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        std::thread::Builder::new()
            .name(format!("openmina_p2p_{name}"))
            .spawn(move || {
                let local_set = tokio::task::LocalSet::new();
                local_set.block_on(&runtime, fut);
            })
            .unwrap();
    }
}
