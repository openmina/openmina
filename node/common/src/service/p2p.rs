use std::collections::BTreeMap;

use node::{
    core::channels::mpsc,
    event_source::Event,
    p2p::{connection::outgoing::P2pConnectionOutgoingInitOpts, PeerId},
};
use rand::prelude::*;

pub use node::p2p::{service::*, service_impl::*};
use sha3::digest::XofReader;

use crate::NodeService;

impl webrtc::P2pServiceWebrtc for NodeService {
    type Event = Event;

    fn random_pick(
        &mut self,
        list: &[P2pConnectionOutgoingInitOpts],
    ) -> P2pConnectionOutgoingInitOpts {
        list.choose(&mut self.rng).unwrap().clone()
    }

    fn event_sender(&self) -> &mpsc::UnboundedSender<Self::Event> {
        self.event_sender()
    }

    fn cmd_sender(&self) -> &mpsc::UnboundedSender<webrtc::Cmd> {
        &self.p2p.webrtc.cmd_sender
    }

    fn peers(&mut self) -> &mut BTreeMap<PeerId, webrtc::PeerState> {
        &mut self.p2p.webrtc.peers
    }
}

impl webrtc_with_libp2p::P2pServiceWebrtcWithLibp2p for NodeService {
    #[cfg(feature = "p2p-libp2p")]
    fn mio(&mut self) -> &mut mio::MioService {
        &mut self.p2p.mio
    }
}

#[cfg(feature = "p2p-libp2p")]
impl P2pCryptoService for NodeService {
    fn generate_random_nonce(&mut self) -> [u8; 24] {
        self.rng.gen()
    }

    fn ephemeral_sk(&mut self) -> [u8; 32] {
        let mut r = [0; 32];
        self.rng_ephemeral.read(&mut r);
        r
    }

    fn static_sk(&mut self) -> [u8; 32] {
        let mut r = [0; 32];
        self.rng_static.read(&mut r);
        r
    }

    fn sign_key(&mut self, key: &[u8; 32]) -> Vec<u8> {
        // TODO: make deterministic
        let msg = [b"noise-libp2p-static-key:", key.as_slice()].concat();
        let sig = self
            .p2p
            .mio
            .keypair()
            .sign(&msg)
            .expect("unable to create signature");

        let mut payload = vec![];
        payload.extend_from_slice(b"\x0a\x24");
        payload.extend_from_slice(&self.p2p.mio.keypair().public().encode_protobuf());
        payload.extend_from_slice(b"\x12\x40");
        payload.extend_from_slice(&sig);
        payload
    }

    fn sign_publication(&mut self, publication: &[u8]) -> Vec<u8> {
        let msg = [b"libp2p-pubsub:", publication].concat();
        self.p2p
            .mio
            .keypair()
            .sign(&msg)
            .expect("unable to create signature")
    }
}
