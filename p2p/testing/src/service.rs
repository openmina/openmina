use std::{net::IpAddr, time::Instant};

use p2p::{
    identity::SecretKey,
    service_impl::{
        mio::MioService, services::NativeP2pNetworkService, webrtc::P2pServiceWebrtc,
        webrtc_with_libp2p::P2pServiceWebrtcWithLibp2p,
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
    keypair: libp2p::identity::Keypair,

    rust_node_event: Option<RustNodeEvent>,
    network_service: NativeP2pNetworkService,
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
            MioService::run(move |mio_event| {
                event_sender
                    .send(mio_event.into())
                    .expect("cannot send mio event")
            })
        };
        let keypair = libp2p::identity::Keypair::ed25519_from_bytes(secret_key.to_bytes())
            .expect("secret key should be valid");
        Self {
            rng: StdRng::seed_from_u64(node_idx as u64),
            event_sender,
            cmd_sender,
            mio,
            peers: Default::default(),
            time,
            keypair,

            rust_node_event: None,
            network_service: Default::default(),
        }
    }

    pub(crate) fn advance_time(&mut self, duration: std::time::Duration) {
        self.time += duration
    }

    pub(crate) fn peek_rust_node_event(&self) -> Option<&RustNodeEvent> {
        self.rust_node_event.as_ref()
    }

    pub(crate) fn rust_node_event(&mut self) -> Option<RustNodeEvent> {
        self.rust_node_event.take()
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

    fn event_sender(&mut self) -> &mut mpsc::UnboundedSender<Self::Event> {
        &mut self.event_sender
    }

    fn cmd_sender(&mut self) -> &mut mpsc::UnboundedSender<p2p::service_impl::webrtc::Cmd> {
        &mut self.cmd_sender
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
        let msg = &[b"noise-libp2p-static-key:", key.as_ref()].concat();
        let sig = self.keypair.sign(msg).expect("unable to create signature");

        let mut payload = vec![];
        payload.extend_from_slice(b"\x0a\x24");
        payload.extend_from_slice(&self.keypair.public().encode_protobuf());
        payload.extend_from_slice(b"\x12\x40");
        payload.extend_from_slice(&sig);
        payload
    }
}

impl p2p::P2pNetworkService for ClusterService {
    fn resolve_name(&mut self, host: &str) -> Result<Vec<IpAddr>, p2p::P2pNetworkServiceError> {
        self.network_service.resolve_name(host)
    }

    fn detect_local_ip(&mut self) -> Result<Vec<IpAddr>, p2p::P2pNetworkServiceError> {
        self.network_service.detect_local_ip()
    }
}

impl RustNodeEventStore for ClusterService {
    fn store_event(&mut self, event: RustNodeEvent) {
        assert!(
            self.rust_node_event.is_none(),
            "can't store event: {event:?}\nanother event: {:?}",
            self.rust_node_event
        );
        self.rust_node_event = Some(event);
    }
}
