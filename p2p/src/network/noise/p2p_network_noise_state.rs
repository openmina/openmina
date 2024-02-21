use std::collections::VecDeque;

use redux::ActionMeta;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::Zeroize;

use chacha20poly1305::{aead::generic_array::GenericArray, AeadInPlace, ChaCha20Poly1305, KeyInit};
use hkdf::{hmac::Hmac, Hkdf};
use sha2::{
    digest::{FixedOutput, Update},
    Sha256,
};

use crate::{identity::PublicKey, PeerId};

use super::{super::*, *};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkNoiseState {
    pub buffer: Vec<u8>,
    pub incoming_chunks: VecDeque<Vec<u8>>,
    pub outgoing_chunks: VecDeque<Vec<u8>>,
    pub decrypted_chunks: VecDeque<Data>,

    pub inner: Option<P2pNetworkNoiseStateInner>,
    pub handshake_optimized: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkNoiseStateInner {
    Initiator(P2pNetworkNoiseStateInitiator),
    Responder(P2pNetworkNoiseStateResponder),
    Done {
        incoming: bool,
        send_key: DataSized<32>,
        recv_key: DataSized<32>,
        // noise_hash: DataSized<32>,
        recv_nonce: u64,
        send_nonce: u64,
        remote_pk: PublicKey,
        remote_peer_id: PeerId,
    },
    Error(NoiseError),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkNoiseStateInitiator {
    i_esk: Sk,
    i_spk: Pk,
    i_ssk: Sk,
    r_epk: Option<Pk>,
    payload: Data,
    noise: NoiseState,
    remote_pk: Option<PublicKey>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkNoiseStateResponder {
    Init {
        r_esk: Sk,
        r_spk: Pk,
        r_ssk: Sk,
        buffer: Vec<u8>,
        payload: Data,
        noise: NoiseState,
    },
    Middle {
        r_esk: Sk,
        noise: NoiseState,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NoiseState {
    hash: DataSized<32>,
    chaining_key: DataSized<32>,
    aead_key: DataSized<32>,
}

impl NoiseState {
    pub fn new(key: [u8; 32]) -> Self {
        NoiseState {
            hash: DataSized(key),
            chaining_key: DataSized(key),
            aead_key: DataSized([0; 32]),
        }
    }

    pub fn mix_hash(&mut self, data: &[u8]) {
        self.hash = DataSized(
            Sha256::default()
                .chain(&self.hash.0)
                .chain(data)
                .finalize_fixed()
                .into(),
        );
    }

    pub fn mix_secret(&mut self, mut secret: [u8; 32]) {
        let hkdf = Hkdf::<Sha256, Hmac<Sha256>>::new(Some(&self.chaining_key.0), &secret);
        secret.zeroize();
        let mut okm = [0; 64];
        hkdf.expand(&[], &mut okm).unwrap();
        self.chaining_key.0.clone_from_slice(&okm[..32]);
        self.aead_key.0.clone_from_slice(&okm[32..]);
    }

    pub fn decrypt<const NONCE: u64>(&mut self, data: &mut [u8], tag: &[u8]) -> Result<(), ()> {
        let mut nonce = GenericArray::default();
        nonce[4..].clone_from_slice(&NONCE.to_le_bytes());

        let hash = Sha256::default()
            .chain(&self.hash.0)
            .chain(&*data)
            .chain(tag)
            .finalize_fixed();

        ChaCha20Poly1305::new(GenericArray::from_slice(&self.aead_key.0))
            .decrypt_in_place_detached(&nonce, &self.hash.0, data, GenericArray::from_slice(tag))
            .map_err(|_| ())
            .map(|()| self.hash.0 = hash.into())
    }

    pub fn encrypt<const NONCE: u64>(&mut self, data: &mut [u8]) -> [u8; 16] {
        let mut nonce = GenericArray::default();
        nonce[4..].clone_from_slice(&NONCE.to_le_bytes());

        let tag = ChaCha20Poly1305::new(GenericArray::from_slice(&self.aead_key.0))
            .encrypt_in_place_detached(&nonce, &self.hash.0, data)
            .unwrap();
        let hash = Sha256::default()
            .chain(&self.hash.0)
            .chain(&*data)
            .chain(tag)
            .finalize_fixed();
        self.hash.0 = hash.into();

        tag.into()
    }

    pub fn finish(&self) -> (DataSized<32>, DataSized<32>) {
        let mut fst = [0; 32];
        let mut scd = [0; 32];

        let hkdf = Hkdf::<Sha256, Hmac<Sha256>>::new(Some(&self.chaining_key.0), b"");
        let mut okm = [0; 64];
        hkdf.expand(&[], &mut okm).unwrap();
        fst.clone_from_slice(&okm[..32]);
        scd.clone_from_slice(&okm[32..]);
        (DataSized(fst), DataSized(scd))
    }
}

impl P2pNetworkNoiseAction {
    pub fn effects<Store, S>(&self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        P2pNetworkPnetOutgoingDataAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseIncomingChunkAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseOutgoingDataAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseOutgoingChunkAction: redux::EnablingCondition<S>,
        P2pNetworkSelectIncomingDataAction: redux::EnablingCondition<S>,
        P2pNetworkSelectInitAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseHandshakeDoneAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseDecryptedDataAction: redux::EnablingCondition<S>,
    {
        let state = store.state();
        let Some(state) = state.network.scheduler.connections.get(&self.addr()) else {
            return;
        };
        let Some(P2pNetworkAuthState::Noise(state)) = &state.auth else {
            return;
        };

        let incoming = state.incoming_chunks.front().cloned().map(Into::into);
        let outgoing = state.outgoing_chunks.front().cloned().map(Into::into);
        let decrypted = state.decrypted_chunks.front().cloned();
        let remote_peer_id =
            if let Some(P2pNetworkNoiseStateInner::Done { remote_peer_id, .. }) = &state.inner {
                Some(remote_peer_id.clone())
            } else {
                None
            };
        let handshake_done = if let Some(P2pNetworkNoiseStateInner::Done {
            remote_peer_id,
            incoming,
            send_nonce,
            recv_nonce,
            ..
        }) = &state.inner
        {
            if ((matches!(self, Self::IncomingChunk(..)) && *incoming)
                || (matches!(self, Self::OutgoingChunk(..)) && !*incoming))
                && *send_nonce == 0
                && *recv_nonce == 0
            {
                Some((remote_peer_id.clone(), *incoming))
            } else {
                None
            }
        } else {
            None
        };
        let handshake_optimized = state.handshake_optimized;
        let middle_initiator =
            matches!(&state.inner, Some(P2pNetworkNoiseStateInner::Initiator(..)));
        let middle_responder = matches!(
            &state.inner,
            Some(P2pNetworkNoiseStateInner::Responder(
                P2pNetworkNoiseStateResponder::Init { .. },
            ))
        );

        if let Self::HandshakeDone(a) = self {
            store.dispatch(P2pNetworkSelectInitAction {
                addr: a.addr,
                kind: SelectKind::Multiplexing(a.peer_id.clone()),
                incoming: a.incoming,
                send_handshake: true,
            });
            return;
        }

        if let Self::DecryptedData(a) = self {
            let kind = match &a.peer_id {
                Some(peer_id) => SelectKind::Multiplexing(peer_id.clone()),
                None => SelectKind::MultiplexingNoPeerId,
            };
            if handshake_optimized && middle_initiator {
                store.dispatch(P2pNetworkSelectInitAction {
                    addr: self.addr(),
                    kind,
                    // it is not a mistake, if we are initiator of noise, the select will be incoming
                    // because noise is
                    // initiator -> responder (ephemeral key)
                    // initiator <- responder (ephemeral key, encrypted static kay and **encrypted payload**)
                    // initiator -> responder (encrypted static kay and **encrypted payload**)
                    // so the responder is sending payload first, hence responder will be initiator of underlying protocol
                    incoming: true,
                    send_handshake: false,
                });
            }
            store.dispatch(P2pNetworkSelectIncomingDataAction {
                addr: self.addr(),
                kind,
                data: a.data.clone(),
            });
            return;
        }

        match self {
            Self::Init(_) | Self::OutgoingData(_) => {
                if let Some(data) = outgoing {
                    store.dispatch(P2pNetworkNoiseOutgoingChunkAction {
                        addr: self.addr(),
                        data,
                    });
                }
            }
            Self::IncomingData(_) => {
                if let Some(data) = incoming {
                    store.dispatch(P2pNetworkNoiseIncomingChunkAction {
                        addr: self.addr(),
                        data,
                    });
                }
            }
            Self::IncomingChunk(_) => {
                if handshake_optimized && middle_responder {
                    store.dispatch(P2pNetworkSelectInitAction {
                        addr: self.addr(),
                        kind: SelectKind::MultiplexingNoPeerId,
                        incoming: false,
                        send_handshake: false,
                    });
                }

                if let Some(data) = outgoing {
                    store.dispatch(P2pNetworkNoiseOutgoingChunkAction {
                        addr: self.addr(),
                        data,
                    });
                }
                if let Some(data) = decrypted {
                    store.dispatch(P2pNetworkNoiseDecryptedDataAction {
                        addr: self.addr(),
                        peer_id: remote_peer_id,
                        data,
                    });
                }
                if let Some(data) = incoming {
                    store.dispatch(P2pNetworkNoiseIncomingChunkAction {
                        addr: self.addr(),
                        data,
                    });
                }
            }
            Self::OutgoingChunk(a) => {
                store.dispatch(P2pNetworkPnetOutgoingDataAction {
                    addr: a.addr,
                    data: a.data.clone(),
                });
            }
            _ => {}
        }

        if !handshake_optimized {
            if (middle_initiator || middle_responder) && matches!(self, Self::IncomingChunk(..)) {
                store.dispatch(P2pNetworkNoiseOutgoingDataAction {
                    addr: self.addr(),
                    data: Data(vec![].into_boxed_slice()),
                });
            } else {
                if let Some((peer_id, incoming)) = handshake_done {
                    store.dispatch(P2pNetworkNoiseHandshakeDoneAction {
                        addr: self.addr(),
                        peer_id,
                        incoming,
                    });
                }
            }
        }
    }
}

impl P2pNetworkNoiseState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkNoiseAction>) {
        match action.action() {
            P2pNetworkNoiseAction::Init(a) => {
                let esk = Sk::from(a.ephemeral_sk.clone());
                let epk = Pk::from_sk(&esk);
                let ssk = Sk::from(a.static_sk.clone());
                let spk = Pk::from_sk(&ssk);
                let payload = a.signature.clone();

                self.inner = if a.incoming {
                    // Luckily the name is 32 bytes long, if it were longer you would have to take a sha2_256 hash of it.
                    let mut noise = NoiseState::new(*b"Noise_XX_25519_ChaChaPoly_SHA256");
                    noise.mix_hash(b"");

                    Some(P2pNetworkNoiseStateInner::Responder(
                        P2pNetworkNoiseStateResponder::Init {
                            r_esk: esk,
                            r_spk: spk,
                            r_ssk: ssk,
                            buffer: vec![],
                            payload,
                            noise,
                        },
                    ))
                } else {
                    let mut chunk = vec![0, 32];
                    chunk.extend_from_slice(epk.0.as_bytes());
                    self.outgoing_chunks.push_back(chunk);

                    let mut noise = NoiseState::new(*b"Noise_XX_25519_ChaChaPoly_SHA256");
                    noise.mix_hash(b"");
                    noise.mix_hash(epk.0.as_bytes());
                    noise.mix_hash(b"");

                    Some(P2pNetworkNoiseStateInner::Initiator(
                        P2pNetworkNoiseStateInitiator {
                            i_esk: esk,
                            i_spk: spk,
                            i_ssk: ssk,
                            r_epk: None,
                            payload,
                            noise,
                            remote_pk: None,
                        },
                    ))
                }
            }
            P2pNetworkNoiseAction::IncomingData(a) => {
                self.buffer.extend_from_slice(&a.data);
                let mut offset = 0;
                loop {
                    let buf = &self.buffer[offset..];
                    if buf.len() >= 2 {
                        let len = u16::from_be_bytes(buf[..2].try_into().expect("cannot fail"));
                        let full_len = 2 + len as usize;
                        if buf.len() >= full_len {
                            self.incoming_chunks.push_back(buf[..full_len].to_vec());
                            offset += full_len;

                            continue;
                        }
                    }
                    break;
                }
                self.buffer = self.buffer[offset..].to_vec();
            }
            P2pNetworkNoiseAction::IncomingChunk(_) => {
                let Some(state) = &mut self.inner else {
                    return;
                };
                if let Some(mut chunk) = self.incoming_chunks.pop_front() {
                    match state {
                        P2pNetworkNoiseStateInner::Initiator(i) => match i.consume(&mut chunk) {
                            Ok(remote_payload) => {
                                self.handshake_optimized = remote_payload.is_some();
                                if let Some(remote_payload) = remote_payload {
                                    self.decrypted_chunks
                                        .push_back(remote_payload.to_vec().into());
                                }
                            }
                            Err(err) => *state = P2pNetworkNoiseStateInner::Error(dbg!(err)),
                        },
                        P2pNetworkNoiseStateInner::Responder(o) => match o.consume(&mut chunk) {
                            Ok(None) => {}
                            Ok(Some((
                                ResponderOutput {
                                    send_key,
                                    recv_key,
                                    remote_pk,
                                    ..
                                },
                                remote_payload,
                            ))) => {
                                let remote_peer_id = remote_pk.peer_id();
                                *state = P2pNetworkNoiseStateInner::Done {
                                    incoming: true,
                                    send_key,
                                    recv_key,
                                    recv_nonce: 0,
                                    send_nonce: 0,
                                    remote_pk,
                                    remote_peer_id,
                                };
                                self.handshake_optimized = remote_payload.is_some();
                                if let Some(remote_payload) = remote_payload {
                                    self.decrypted_chunks
                                        .push_back(remote_payload.to_vec().into());
                                }
                            }
                            Err(err) => {
                                *state = P2pNetworkNoiseStateInner::Error(dbg!(err));
                            }
                        },
                        P2pNetworkNoiseStateInner::Done {
                            recv_key,
                            recv_nonce,
                            ..
                        } => {
                            let aead = ChaCha20Poly1305::new(&recv_key.0.into());
                            let mut chunk = chunk;
                            let mut nonce = GenericArray::default();
                            nonce[4..].clone_from_slice(&recv_nonce.to_le_bytes());
                            *recv_nonce += 1;
                            if chunk.len() < 18 {
                                *state = P2pNetworkNoiseStateInner::Error(NoiseError::ChunkTooShort)
                            } else {
                                let data = &mut chunk[2..];
                                let (data, tag) = data.split_at_mut(data.len() - 16);
                                let tag = GenericArray::from_slice(&*tag);
                                if let Err(_) =
                                    aead.decrypt_in_place_detached(&nonce, &[], data, tag)
                                {
                                    *state = P2pNetworkNoiseStateInner::Error(dbg!(
                                        NoiseError::FirstMacMismatch
                                    ));
                                } else {
                                    self.decrypted_chunks.push_back(data.to_vec().into());
                                }
                            }
                        }
                        P2pNetworkNoiseStateInner::Error(_) => {}
                    }
                }
            }
            P2pNetworkNoiseAction::OutgoingChunk(_) => {
                self.outgoing_chunks.pop_front();
            }
            P2pNetworkNoiseAction::OutgoingData(a) => {
                let Some(state) = &mut self.inner else {
                    return;
                };
                if a.data.is_empty() && self.handshake_optimized {
                    return;
                }
                match state {
                    P2pNetworkNoiseStateInner::Done {
                        send_key,
                        send_nonce,
                        ..
                    } => {
                        let aead = ChaCha20Poly1305::new(&send_key.0.into());
                        let chunk_max_size = u16::MAX as usize - 18;
                        for data in a.data.chunks(chunk_max_size) {
                            let mut chunk = Vec::with_capacity(18 + data.len());
                            chunk.extend_from_slice(&((data.len() + 16) as u16).to_be_bytes());
                            chunk.extend_from_slice(data);

                            let mut nonce = GenericArray::default();
                            nonce[4..].clone_from_slice(&send_nonce.to_le_bytes());
                            *send_nonce += 1;

                            let tag = aead
                                .encrypt_in_place_detached(
                                    &nonce,
                                    &[],
                                    &mut chunk[2..(2 + data.len())],
                                )
                                .expect("cannot fail");
                            chunk.extend_from_slice(&tag);
                            self.outgoing_chunks.push_back(chunk);
                        }
                    }
                    P2pNetworkNoiseStateInner::Initiator(i) => {
                        if let (Some((chunk, (send_key, recv_key))), Some(remote_pk)) =
                            (i.generate(&a.data), i.remote_pk.clone())
                        {
                            self.outgoing_chunks.push_back(chunk);
                            let remote_peer_id = remote_pk.peer_id();
                            *state = P2pNetworkNoiseStateInner::Done {
                                incoming: false,
                                send_key,
                                recv_key,
                                recv_nonce: 0,
                                send_nonce: 0,
                                remote_pk,
                                remote_peer_id,
                            };
                        }
                    }
                    P2pNetworkNoiseStateInner::Responder(r) => {
                        if let Some(chunk) = r.generate(&a.data) {
                            self.outgoing_chunks.push_back(chunk);
                        }
                    }
                    // TODO: report error
                    _ => {}
                }
            }
            P2pNetworkNoiseAction::DecryptedData(_) => {
                self.decrypted_chunks.pop_front();
            }
            P2pNetworkNoiseAction::HandshakeDone(_) => {}
        }
    }
}

#[derive(Debug, Error, Serialize, Deserialize, Clone)]
pub enum NoiseError {
    #[error("chunk too short")]
    ChunkTooShort,
    #[error("first MAC mismatch")]
    FirstMacMismatch,
    #[error("second MAC mismatch")]
    SecondMacMismatch,
    #[error("failed to parse public key")]
    BadPublicKey,
    #[error("invalid signature")]
    InvalidSignature,
}

struct ResponderOutput {
    send_key: DataSized<32>,
    recv_key: DataSized<32>,
    remote_pk: PublicKey,
}

impl P2pNetworkNoiseStateInitiator {
    fn generate(&mut self, data: &[u8]) -> Option<(Vec<u8>, (DataSized<32>, DataSized<32>))> {
        let Self {
            i_spk,
            i_ssk,
            r_epk,
            noise,
            payload,
            ..
        } = self;

        let r_epk = r_epk.as_ref()?;

        let mut i_spk_bytes = i_spk.0.to_bytes();
        let tag = noise.encrypt::<1>(&mut i_spk_bytes);
        noise.mix_secret(&*i_ssk * r_epk);
        let mut payload = payload.0.to_vec();
        // if handshake is optimized by early mux negotiation
        if !data.is_empty() {
            payload.extend_from_slice(b"\x22\x13");
            payload.push(data.len() as u8);
            payload.extend_from_slice(data);
        }
        let payload_tag = noise.encrypt::<0>(&mut payload);

        let mut chunk = vec![0; 2];
        chunk.extend_from_slice(&i_spk_bytes);
        chunk.extend_from_slice(&tag);
        chunk.extend_from_slice(&*payload);
        chunk.extend_from_slice(&payload_tag);
        let l = (chunk.len() - 2) as u16;
        chunk[..2].clone_from_slice(&l.to_be_bytes());

        Some((chunk, noise.finish()))
    }

    fn consume<'a>(&'_ mut self, chunk: &'a mut [u8]) -> Result<Option<&'a mut [u8]>, NoiseError> {
        use self::NoiseError::*;

        let Self {
            i_esk,
            noise,
            remote_pk,
            ..
        } = self;

        let msg = &mut chunk[2..];
        let len = msg.len();
        if len < 200 {
            return Err(ChunkTooShort);
        }
        let r_epk = Pk::from_bytes(msg[..32].try_into().expect("cannot fail"));
        let mut r_spk_bytes =
            <[u8; 32]>::try_from(&msg[32..64]).expect("cannot fail, checked above");
        let tag = &msg[64..80];
        let r_spk;

        noise.mix_hash(r_epk.0.as_bytes());
        noise.mix_secret(&*i_esk * &r_epk);
        noise
            .decrypt::<0>(&mut r_spk_bytes, tag)
            .map_err(|_| FirstMacMismatch)?;

        r_spk = Pk::from_bytes(r_spk_bytes);
        noise.mix_secret(&*i_esk * &r_spk);

        let (msg, tag) = msg.split_at_mut(len - 16);
        let remote_payload = &mut msg[80..];
        noise
            .decrypt::<0>(remote_payload, &*tag)
            .map_err(|_| SecondMacMismatch)?;

        let pk = libp2p_identity::PublicKey::try_decode_protobuf(&remote_payload[2..38])
            .map_err(|_| BadPublicKey)?;
        let msg = &[b"noise-libp2p-static-key:", r_spk.0.as_bytes().as_ref()].concat();
        if !pk.verify(msg, &remote_payload[40..(40 + 64)]) {
            Err(InvalidSignature)
        } else {
            self.r_epk = Some(r_epk);

            let remote_payload = &mut remote_payload[104..];
            let remote_payload = if remote_payload.len() > 3 {
                Some(&mut remote_payload[3..])
            } else {
                None
            };
            let pk = pk.try_into_ed25519().map_err(|_| BadPublicKey)?;
            *remote_pk = Some(PublicKey::from_bytes(pk.to_bytes()).map_err(|_| BadPublicKey)?);

            Ok(remote_payload)
        }
    }
}

impl P2pNetworkNoiseStateResponder {
    fn generate(&mut self, data: &[u8]) -> Option<Vec<u8>> {
        let Self::Init {
            buffer,
            payload,
            noise,
            r_esk,
            ..
        } = self
        else {
            return None;
        };

        let mut payload = payload.0.to_vec();
        if !data.is_empty() {
            payload.extend_from_slice(b"\x22\x13");
            payload.push(data.len() as u8);
            payload.extend_from_slice(data);
        }
        let payload_tag = noise.encrypt::<0>(&mut payload);

        buffer.extend_from_slice(&*payload);
        buffer.extend_from_slice(&payload_tag);
        let l = (buffer.len() - 2) as u16;
        buffer[..2].clone_from_slice(&l.to_be_bytes());

        let noise = noise.clone();
        let r_esk = r_esk.clone();
        let new_chunk = std::mem::take(buffer);

        *self = Self::Middle { r_esk, noise };

        Some(new_chunk)
    }

    fn consume<'a>(
        &'_ mut self,
        chunk: &'a mut [u8],
    ) -> Result<Option<(ResponderOutput, Option<&'a mut [u8]>)>, NoiseError> {
        use self::NoiseError::*;

        match self {
            Self::Init {
                r_esk,
                r_spk,
                r_ssk,
                buffer,
                noise,
                ..
            } => {
                let msg = &mut chunk[2..];
                let len = msg.len();
                if len < 32 {
                    return Err(ChunkTooShort);
                }
                let i_epk = Pk::from_bytes(msg[..32].try_into().expect("cannot fail"));

                let r_epk = Pk::from_sk(&*r_esk);

                let mut r_spk_bytes = r_spk.0.to_bytes();

                noise.mix_hash(i_epk.0.as_bytes());
                noise.mix_hash(b"");
                noise.mix_hash(r_epk.0.as_bytes());
                noise.mix_secret(&*r_esk * &i_epk);
                let tag = noise.encrypt::<0>(&mut r_spk_bytes);
                noise.mix_secret(&*r_ssk * &i_epk);
                r_ssk.zeroize();

                *buffer = vec![0; 2];
                buffer.extend_from_slice(r_epk.0.as_bytes());
                buffer.extend_from_slice(&r_spk_bytes);
                buffer.extend_from_slice(&tag);

                Ok(None)
            }
            Self::Middle { r_esk, noise } => {
                let msg = &mut chunk[2..];
                let len = msg.len();
                if len < 152 {
                    return Err(ChunkTooShort);
                }

                // TODO: refactor obscure arithmetics
                let mut i_spk_bytes = <[u8; 32]>::try_from(&msg[..32]).expect("cannot fail");
                let (tag, msg) = msg[32..].split_at_mut(16);
                let len = msg.len();
                let (remote_payload, payload_tag) = msg.split_at_mut(len - 16);

                noise
                    .decrypt::<1>(&mut i_spk_bytes, &tag)
                    .map_err(|()| FirstMacMismatch)?;
                let i_spk = Pk::from_bytes(i_spk_bytes);
                noise.mix_secret(&*r_esk * &i_spk);
                r_esk.zeroize();

                noise
                    .decrypt::<0>(remote_payload, &payload_tag)
                    .map_err(|_| SecondMacMismatch)?;
                let (recv_key, send_key) = noise.finish();

                let pk = libp2p_identity::PublicKey::try_decode_protobuf(&remote_payload[2..38])
                    .map_err(|_| BadPublicKey)?;
                let msg = &[b"noise-libp2p-static-key:", i_spk.0.as_bytes().as_ref()].concat();
                if !pk.verify(msg, &remote_payload[40..(40 + 64)]) {
                    Err(InvalidSignature)
                } else {
                    let pk = pk.try_into_ed25519().map_err(|_| BadPublicKey)?;
                    let remote_pk =
                        PublicKey::from_bytes(pk.to_bytes()).map_err(|_| BadPublicKey)?;

                    let remote_payload = &mut remote_payload[104..];
                    let remote_payload = if remote_payload.len() > 3 {
                        Some(&mut remote_payload[3..])
                    } else {
                        None
                    };

                    Ok(Some((
                        ResponderOutput {
                            send_key,
                            recv_key,
                            remote_pk,
                        },
                        remote_payload,
                    )))
                }
            }
        }
    }
}

pub use self::wrapper::{Pk, Sk};
mod wrapper {
    use std::ops::Mul;

    use curve25519_dalek::{MontgomeryPoint, Scalar};
    use serde::{Deserialize, Serialize};
    use zeroize::Zeroize;

    use crate::DataSized;

    impl<'a, 'b> Mul<&'b Pk> for &'a Sk {
        type Output = [u8; 32];

        fn mul(self, rhs: &'b Pk) -> Self::Output {
            (&self.0 * &rhs.0).0
        }
    }

    #[derive(Debug, Clone)]
    pub struct Pk(pub MontgomeryPoint);

    impl Pk {
        pub fn from_bytes(bytes: [u8; 32]) -> Self {
            Pk(MontgomeryPoint(bytes))
        }

        pub fn from_sk(sk: &Sk) -> Self {
            let t = curve25519_dalek::constants::ED25519_BASEPOINT_TABLE;
            Pk((t * &sk.0).to_montgomery())
        }
    }

    impl Serialize for Pk {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            hex::encode(self.0.as_bytes()).serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Pk {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use serde::de::Error;

            let str = <&'de str>::deserialize(deserializer)?;
            hex::decode(str)
                .map_err(Error::custom)
                .and_then(|b| b.try_into().map_err(|_| Error::custom("wrong length")))
                .map(MontgomeryPoint)
                .map(Self)
        }
    }

    #[derive(Debug, Clone)]
    pub struct Sk(pub Scalar);

    impl From<DataSized<32>> for Sk {
        fn from(value: DataSized<32>) -> Self {
            Sk(Scalar::from_bytes_mod_order(value.0))
        }
    }

    impl Zeroize for Sk {
        fn zeroize(&mut self) {
            self.0.zeroize();
        }
    }

    impl Serialize for Sk {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            hex::encode(self.0.as_bytes()).serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Sk {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use serde::de::Error;

            let str = <&'de str>::deserialize(deserializer)?;
            hex::decode(str)
                .map_err(Error::custom)
                .and_then(|b| b.try_into().map_err(|_| Error::custom("wrong length")))
                .map(Scalar::from_bytes_mod_order)
                .map(Self)
        }
    }
}
