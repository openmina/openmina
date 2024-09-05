use std::{collections::VecDeque, time::Instant};

use p2p::{
    identity::SecretKey,
    service_impl::{
        mio::MioService, webrtc::P2pServiceWebrtc, webrtc_with_libp2p::P2pServiceWebrtcWithLibp2p,
    },
    P2pCryptoService, P2pEvent,
};
use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};
use redux::{Service, TimeService};
use tokio::sync::mpsc;

use crate::event::{RustNodeEvent, RustNodeEventStore};

pub struct ClusterService {
    pub rng: StdRng,
    pub event_sender: mpsc::UnboundedSender<P2pEvent>,
    pub cmd_sender: mpsc::UnboundedSender<p2p::service_impl::webrtc::Cmd>,
    mio: MioService,
    peers: std::collections::BTreeMap<p2p::PeerId, p2p::service_impl::webrtc::PeerState>,
    time: Instant,

    rust_node_events: VecDeque<RustNodeEvent>,
}

impl ClusterService {
    pub fn new(
        node_idx: usize,
        secret_key: SecretKey,
        event_sender: mpsc::UnboundedSender<P2pEvent>,
        cmd_sender: mpsc::UnboundedSender<p2p::service_impl::webrtc::Cmd>,
        time: Instant,
    ) -> Self {
        let mio = {
            let event_sender = event_sender.clone();
            let mut mio = MioService::pending(secret_key.into());
            mio.run(move |mio_event| {
                let _ = event_sender.send(mio_event.into());
                //.expect("cannot send mio event")
            });
            mio
        };
        Self {
            rng: StdRng::seed_from_u64(node_idx as u64),
            event_sender,
            cmd_sender,
            mio,
            peers: Default::default(),
            time,

            rust_node_events: Default::default(),
        }
    }

    pub(crate) fn advance_time(&mut self, duration: std::time::Duration) {
        self.time += duration
    }

    pub(crate) fn rust_node_event(&mut self) -> Option<RustNodeEvent> {
        self.rust_node_events.pop_front()
    }
}

impl TimeService for ClusterService {
    fn monotonic_time(&mut self) -> redux::Instant {
        self.time
    }
}

impl Service for ClusterService {}

impl P2pServiceWebrtcWithLibp2p for ClusterService {
    fn mio(&mut self) -> &mut p2p::service_impl::mio::MioService {
        &mut self.mio
    }
}

impl P2pServiceWebrtc for ClusterService {
    type Event = P2pEvent;

    fn random_pick(
        &mut self,
        list: &[p2p::connection::outgoing::P2pConnectionOutgoingInitOpts],
    ) -> p2p::connection::outgoing::P2pConnectionOutgoingInitOpts {
        list.choose(&mut self.rng).unwrap().clone()
    }

    fn event_sender(&self) -> &mpsc::UnboundedSender<Self::Event> {
        &self.event_sender
    }

    fn cmd_sender(&self) -> &mpsc::UnboundedSender<p2p::service_impl::webrtc::Cmd> {
        &self.cmd_sender
    }

    fn peers(
        &mut self,
    ) -> &mut std::collections::BTreeMap<p2p::PeerId, p2p::service_impl::webrtc::PeerState> {
        &mut self.peers
    }
}

impl P2pCryptoService for ClusterService {
    fn generate_random_nonce(&mut self) -> [u8; 24] {
        self.rng.gen()
    }

    fn ephemeral_sk(&mut self) -> [u8; 32] {
        self.rng.gen()
    }

    fn static_sk(&mut self) -> [u8; 32] {
        self.rng.gen()
    }

    // TODO: move it to statemachine.
    fn sign_key(&mut self, key: &[u8; 32]) -> Vec<u8> {
        let msg = [b"noise-libp2p-static-key:", key.as_ref()].concat();
        let sig = self
            .mio
            .keypair()
            .sign(&msg)
            .expect("unable to create signature");

        let mut payload = vec![];
        payload.extend_from_slice(b"\x0a\x24");
        payload.extend_from_slice(&self.mio.keypair().public().encode_protobuf());
        payload.extend_from_slice(b"\x12\x40");
        payload.extend_from_slice(&sig);
        payload
    }

    fn sign_publication(&mut self, publication: &[u8]) -> Vec<u8> {
        let msg: Vec<u8> = [b"libp2p-pubsub:", publication].concat();
        self.mio
            .keypair()
            .sign(&msg)
            .expect("unable to create signature")
    }

    fn verify_publication(&mut self, pk: &[u8], publication: &[u8], sig: &[u8]) -> bool {
        let Ok(pk) = libp2p_identity::PublicKey::try_decode_protobuf(pk) else {
            return false;
        };
        let msg: Vec<u8> = [b"libp2p-pubsub:", publication].concat();
        pk.verify(&msg, sig)
    }
}

impl RustNodeEventStore for ClusterService {
    fn store_event(&mut self, event: RustNodeEvent) {
        self.rust_node_events.push_back(event);
    }
}
