use std::collections::BTreeMap;

use node::{
    core::channels::mpsc,
    event_source::Event,
    p2p::{
        connection::outgoing::P2pConnectionOutgoingInitOpts,
        identity::{EncryptableType, PublicKey},
        webrtc::ConnectionAuth,
        PeerId,
    },
};
use rand::prelude::*;
#[cfg(feature = "p2p-libp2p")]
use sha3::digest::XofReader;

pub use node::p2p::{service::*, service_impl::*};

use crate::NodeService;

impl webrtc::P2pServiceWebrtc for NodeService {
    type Event = Event;

    fn random_pick(
        &mut self,
        list: &[P2pConnectionOutgoingInitOpts],
    ) -> Option<P2pConnectionOutgoingInitOpts> {
        list.choose(&mut self.rng).cloned()
    }

    fn event_sender(&self) -> &mpsc::UnboundedSender<Self::Event> {
        self.event_sender()
    }

    fn cmd_sender(&self) -> &mpsc::TrackedUnboundedSender<webrtc::Cmd> {
        &self.p2p.webrtc.cmd_sender
    }

    fn peers(&mut self) -> &mut BTreeMap<PeerId, webrtc::PeerState> {
        &mut self.p2p.webrtc.peers
    }

    fn encrypt<T: EncryptableType>(
        &mut self,
        other_pk: &PublicKey,
        message: &T,
    ) -> Result<T::Encrypted, Box<dyn std::error::Error>> {
        let rng = &mut self.rng;
        self.p2p.sec_key.encrypt(other_pk, rng, message)
    }

    fn decrypt<T: EncryptableType>(
        &mut self,
        other_pk: &PublicKey,
        encrypted: &T::Encrypted,
    ) -> Result<T, Box<dyn std::error::Error>> {
        self.p2p.sec_key.decrypt(other_pk, encrypted)
    }

    #[cfg(not(feature = "p2p-webrtc"))]
    fn auth_encrypt_and_send(
        &mut self,
        peer_id: PeerId,
        other_pub_key: &PublicKey,
        auth: ConnectionAuth,
    ) {
        let _ = (peer_id, other_pub_key, auth);
    }

    #[cfg(feature = "p2p-webrtc")]
    fn auth_encrypt_and_send(
        &mut self,
        peer_id: PeerId,
        other_pub_key: &PublicKey,
        auth: ConnectionAuth,
    ) {
        let encrypted = auth.encrypt(&self.p2p.sec_key, other_pub_key, &mut self.rng);
        Self::auth_send(self, peer_id, other_pub_key, encrypted);
    }

    fn auth_decrypt(
        &mut self,
        other_pub_key: &PublicKey,
        auth: node::p2p::webrtc::ConnectionAuthEncrypted,
    ) -> Option<ConnectionAuth> {
        auth.decrypt(&self.p2p.sec_key, other_pub_key)
    }
}

impl webrtc_with_libp2p::P2pServiceWebrtcWithLibp2p for NodeService {
    #[cfg(feature = "p2p-libp2p")]
    fn mio(&mut self) -> &mut mio::MioService {
        &mut self.p2p.mio
    }

    fn connections(&self) -> std::collections::BTreeSet<PeerId> {
        self.p2p.webrtc.peers.keys().copied().collect()
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
        let sig = self.p2p.sec_key.sign(&msg);
        let libp2p_sec_key = libp2p_identity::Keypair::try_from(self.p2p.sec_key.clone()).unwrap();

        let mut payload = vec![];
        payload.extend_from_slice(b"\x0a\x24");
        payload.extend_from_slice(&libp2p_sec_key.public().encode_protobuf());
        payload.extend_from_slice(b"\x12\x40");
        payload.extend_from_slice(&sig.to_bytes());
        payload
    }

    fn sign_publication(&mut self, publication: &[u8]) -> Vec<u8> {
        self.p2p
            .sec_key
            .libp2p_pubsub_sign(publication)
            .to_bytes()
            .to_vec()
    }

    fn verify_publication(
        &mut self,
        pk: &libp2p_identity::PublicKey,
        publication: &[u8],
        sig: &[u8],
    ) -> bool {
        let msg: Vec<u8> = [b"libp2p-pubsub:", publication].concat();
        pk.verify(&msg, sig)
    }
}
