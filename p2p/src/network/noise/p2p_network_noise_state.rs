use std::collections::VecDeque;

use redux::ActionMeta;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::Zeroize;

use chacha20poly1305::{AeadInPlace, ChaCha20Poly1305, KeyInit};
use sha2::Sha256;
use vru_noise::{
    generic_array::{typenum, GenericArray},
    hkdf::hmac::Hmac,
    ChainingKey, OutputRaw, SymmetricState,
};

use crate::{identity::PublicKey, PeerId};

use super::{super::*, *};

type C = (Hmac<Sha256>, Sha256, typenum::B0, ChaCha20Poly1305);

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkNoiseState {
    pub buffer: Vec<u8>,
    pub incoming_chunks: VecDeque<Vec<u8>>,
    pub outgoing_chunks: VecDeque<Vec<u8>>,
    pub decrypted_chunks: VecDeque<Data>,

    pub inner: Option<P2pNetworkNoiseStateInner>,
    pub handshake_done_reported: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkNoiseStateInner {
    Initiator(P2pNetworkNoiseStateInitiator),
    Responder(P2pNetworkNoiseStateResponder),
    Done {
        incoming: bool,
        output: OutputRaw<C>,
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
    payload: Data,
    state: Option<SymmetricState<C, ChainingKey<C>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkNoiseStateResponder {
    Init {
        r_esk: Sk,
        r_spk: Pk,
        r_ssk: Sk,
        payload: Data,
        state: Option<SymmetricState<C, ChainingKey<C>>>,
    },
    Middle {
        r_esk: Sk,
        state: Option<SymmetricState<C, ChainingKey<C>>>,
        shared_secrets: [[u8; 32]; 2],
        hash: [u8; 32],
    },
}

impl P2pNetworkNoiseAction {
    pub fn effects<Store, S>(&self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        P2pNetworkPnetOutgoingDataAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseIncomingChunkAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseOutgoingChunkAction: redux::EnablingCondition<S>,
        P2pNetworkSelectIncomingDataAction: redux::EnablingCondition<S>,
        P2pNetworkSelectInitAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseHandshakeDoneAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseDecryptedDataAction: redux::EnablingCondition<S>,
    {
        if let Self::HandshakeDone(a) = self {
            store.dispatch(P2pNetworkSelectInitAction {
                addr: a.addr,
                kind: SelectKind::Multiplexing(a.peer_id),
                incoming: a.incoming,
            });
            return;
        } else if let Self::DecryptedData(a) = self {
            store.dispatch(P2pNetworkSelectIncomingDataAction {
                addr: self.addr(),
                kind: SelectKind::Multiplexing(a.peer_id),
                data: a.data.clone(),
            });
            return;
        }

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
            ..
        }) = &state.inner
        {
            if state.handshake_done_reported {
                None
            } else {
                Some((remote_peer_id.clone(), *incoming))
            }
        } else {
            None
        };

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
                if let Some(data) = outgoing {
                    store.dispatch(P2pNetworkNoiseOutgoingChunkAction {
                        addr: self.addr(),
                        data,
                    });
                }
                if let (Some(data), Some(peer_id)) = (decrypted, remote_peer_id) {
                    store.dispatch(P2pNetworkNoiseDecryptedDataAction {
                        addr: self.addr(),
                        peer_id,
                        data,
                    });
                }
                if let Some((peer_id, incoming)) = handshake_done {
                    store.dispatch(P2pNetworkNoiseHandshakeDoneAction {
                        addr: self.addr(),
                        peer_id,
                        incoming,
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
                    let state =
                        Some(SymmetricState::new("Noise_XX_25519_ChaChaPoly_SHA256").mix_hash(&[]));

                    Some(P2pNetworkNoiseStateInner::Responder(
                        P2pNetworkNoiseStateResponder::Init {
                            r_esk: esk,
                            r_spk: spk,
                            r_ssk: ssk,
                            payload,
                            state,
                        },
                    ))
                } else {
                    let mut chunk = vec![0, 32];
                    chunk.extend_from_slice(epk.0.as_bytes());
                    self.outgoing_chunks.push_back(chunk);
                    let state = Some(
                        SymmetricState::new("Noise_XX_25519_ChaChaPoly_SHA256")
                            .mix_hash(&[])
                            .mix_hash(epk.0.as_bytes())
                            .mix_hash(&[]),
                    );
                    Some(P2pNetworkNoiseStateInner::Initiator(
                        P2pNetworkNoiseStateInitiator {
                            i_esk: esk,
                            i_spk: spk,
                            i_ssk: ssk,
                            payload,
                            state,
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
                if let Some(chunk) = self.incoming_chunks.pop_front() {
                    match state {
                        P2pNetworkNoiseStateInner::Initiator(i) => match i.process(chunk) {
                            Ok((chunk, output, remote_pk)) => {
                                let remote_peer_id = remote_pk.peer_id();
                                *state = P2pNetworkNoiseStateInner::Done {
                                    incoming: false,
                                    output,
                                    recv_nonce: 0,
                                    send_nonce: 0,
                                    remote_pk,
                                    remote_peer_id,
                                };
                                self.outgoing_chunks.push_back(chunk);
                            }
                            Err(err) => *state = P2pNetworkNoiseStateInner::Error(err),
                        },
                        P2pNetworkNoiseStateInner::Responder(o) => match o.process(chunk) {
                            Ok(ResponderOutput::Chunk(chunk)) => {
                                self.outgoing_chunks.push_back(chunk)
                            }
                            Ok(ResponderOutput::Done(output, remote_pk)) => {
                                let remote_peer_id = remote_pk.peer_id();
                                *state = P2pNetworkNoiseStateInner::Done {
                                    incoming: true,
                                    output,
                                    recv_nonce: 0,
                                    send_nonce: 0,
                                    remote_pk,
                                    remote_peer_id,
                                };
                            }
                            Err(err) => *state = P2pNetworkNoiseStateInner::Error(dbg!(err)),
                        },
                        P2pNetworkNoiseStateInner::Done {
                            output, recv_nonce, ..
                        } => {
                            let aead = ChaCha20Poly1305::new(&output.receiver);
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
                                    *state = P2pNetworkNoiseStateInner::Error(
                                        NoiseError::FirstMacMismatch,
                                    );
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
                match state {
                    P2pNetworkNoiseStateInner::Done {
                        output, send_nonce, ..
                    } => {
                        let aead = ChaCha20Poly1305::new(&output.sender);
                        let chunk_max_size = u16::MAX as usize - 18;
                        for data in a.data.chunks(chunk_max_size) {
                            let mut chunk = Vec::with_capacity(18 + data.len());
                            chunk.extend_from_slice(&((data.len() + 16) as u16).to_be_bytes());
                            chunk.extend_from_slice(data);

                            let mut nonce = GenericArray::default();
                            nonce[..8].clone_from_slice(&send_nonce.to_le_bytes());
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
                    // TODO: report error
                    _ => {}
                }
            }
            P2pNetworkNoiseAction::DecryptedData(_) => {
                self.decrypted_chunks.pop_front();
            }
            P2pNetworkNoiseAction::HandshakeDone(_) => {
                self.handshake_done_reported = true;
            }
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

impl P2pNetworkNoiseStateInitiator {
    fn process(
        &mut self,
        mut chunk: Vec<u8>,
    ) -> Result<(Vec<u8>, OutputRaw<C>, PublicKey), NoiseError> {
        use self::NoiseError::*;

        let Self {
            i_esk,
            i_spk,
            i_ssk,
            payload,
            state,
        } = self;

        let msg = &mut chunk[2..];
        let len = msg.len();
        if len < 96 {
            return Err(ChunkTooShort);
        }
        let r_epk = Pk::from_bytes(msg[..32].try_into().expect("cannot fail"));
        let mut r_spk_bytes =
            <[u8; 32]>::try_from(&msg[32..64]).expect("cannot fail, checked above");
        let tag = *GenericArray::from_slice(&msg[64..80]);
        let r_spk;
        let payload_tag = *GenericArray::from_slice(&msg[(len - 16)..]);

        let mut i_spk_bytes = i_spk.0.to_bytes();

        let state = state
            .take()
            .expect("should not fail")
            .mix_hash(r_epk.0.as_bytes())
            .mix_shared_secret(&*i_esk * &r_epk)
            .decrypt(&mut r_spk_bytes, &tag)
            .map_err(|_| FirstMacMismatch)?
            .mix_shared_secret({
                r_spk = Pk::from_bytes(r_spk_bytes);
                &*i_esk * &r_spk
            })
            .decrypt(&mut msg[80..(len - 16)], &payload_tag)
            .map_err(|_| SecondMacMismatch)?;
        let (state, tag) = state.encrypt(&mut i_spk_bytes);
        let (state, payload_tag) = state.mix_shared_secret(&*i_ssk * &r_epk).encrypt(payload);

        let output = state.finish_raw::<1, false>();

        i_esk.0.zeroize();
        i_ssk.0.zeroize();

        let mut chunk = vec![0; 2];
        chunk.extend_from_slice(&i_spk_bytes);
        chunk.extend_from_slice(&tag);
        chunk.extend_from_slice(&*payload);
        chunk.extend_from_slice(&payload_tag);
        let l = (chunk.len() - 2) as u16;
        chunk[..2].clone_from_slice(&l.to_be_bytes());

        let remote_payload = msg[80..(len - 16)].to_vec().into_boxed_slice();

        let pk = libp2p_identity::PublicKey::try_decode_protobuf(&remote_payload[2..38])
            .map_err(|_| BadPublicKey)?;
        let msg = &[b"noise-libp2p-static-key:", r_spk.0.as_bytes().as_ref()].concat();
        if !pk.verify(msg, &remote_payload[40..]) {
            Err(InvalidSignature)
        } else {
            let pk = pk.try_into_ed25519().map_err(|_| BadPublicKey)?;
            let remote_pk = PublicKey::from_bytes(pk.to_bytes()).map_err(|_| BadPublicKey)?;
            Ok((chunk, output, remote_pk))
        }
    }
}

enum ResponderOutput {
    Chunk(Vec<u8>),
    Done(OutputRaw<C>, PublicKey),
}

impl P2pNetworkNoiseStateResponder {
    fn process(&mut self, mut chunk: Vec<u8>) -> Result<ResponderOutput, NoiseError> {
        use self::NoiseError::*;

        match self {
            Self::Init {
                r_esk,
                r_spk,
                r_ssk,
                payload,
                state,
            } => {
                let msg = &mut chunk[2..];
                let len = msg.len();
                if len < 32 {
                    return Err(ChunkTooShort);
                }
                let i_epk = Pk::from_bytes(msg[..32].try_into().expect("cannot fail"));

                let r_epk = Pk::from_sk(&*r_esk);

                let mut r_spk_bytes = r_spk.0.to_bytes();

                let shared_secret_0 = &*r_esk * &i_epk;
                let shared_secret_1 = &*r_ssk * &i_epk;

                let state = state
                    .take()
                    .expect("should not fail")
                    .mix_hash(i_epk.0.as_bytes())
                    .mix_hash(&[])
                    .mix_hash(r_epk.0.as_bytes());
                let state_stored = state.clone();
                let (state, tag) = state
                    .mix_shared_secret(shared_secret_0)
                    .encrypt(&mut r_spk_bytes);
                let (state, payload_tag) = state
                    .mix_shared_secret(shared_secret_1)
                    .encrypt(&mut *payload);

                let hash = state.hash();

                // r_esk.0.zeroize();
                r_ssk.0.zeroize();

                let mut chunk = vec![0; 2];
                chunk.extend_from_slice(r_epk.0.as_bytes());
                chunk.extend_from_slice(&r_spk_bytes);
                chunk.extend_from_slice(&tag);
                chunk.extend_from_slice(&*payload);
                chunk.extend_from_slice(&payload_tag);
                let l = (chunk.len() - 2) as u16;
                chunk[..2].clone_from_slice(&l.to_be_bytes());

                *self = Self::Middle {
                    r_esk: r_esk.clone(),
                    state: Some(state_stored),
                    hash: hash.into(),
                    shared_secrets: [shared_secret_0, shared_secret_1],
                };

                Ok(ResponderOutput::Chunk(chunk))
            }
            Self::Middle {
                r_esk,
                state,
                hash,
                shared_secrets,
            } => {
                let msg = &mut chunk[2..];
                let len = msg.len();
                if len < 64 {
                    return Err(ChunkTooShort);
                }

                let mut i_spk_bytes = <[u8; 32]>::try_from(&msg[..32]).expect("cannot fail");
                let tag = *GenericArray::from_slice(&msg[32..48]);
                let i_spk;
                let mut remote_payload = msg[48..(len - 16)].to_vec().into_boxed_slice();
                let payload_tag = *GenericArray::from_slice(&msg[(len - 16)..]);

                let state = state
                    .take()
                    .expect("should not fail")
                    .mix_shared_secret(shared_secrets[0])
                    .mix_shared_secret(shared_secrets[1])
                    .unsafe_set_hash((*hash).into());
                shared_secrets[0].zeroize();
                shared_secrets[1].zeroize();

                let state = state
                    .increase()
                    .decrypt(&mut i_spk_bytes, &tag)
                    .map_err(|_| FirstMacMismatch)?
                    .mix_shared_secret({
                        i_spk = Pk::from_bytes(i_spk_bytes);
                        &*r_esk * &i_spk
                    });
                r_esk.0.zeroize();

                let output = state
                    .decrypt(&mut remote_payload, &payload_tag)
                    .map_err(|_| SecondMacMismatch)?
                    .finish_raw::<1, true>();

                let pk = libp2p_identity::PublicKey::try_decode_protobuf(&remote_payload[2..38])
                    .map_err(|_| BadPublicKey)?;
                let msg = &[b"noise-libp2p-static-key:", i_spk.0.as_bytes().as_ref()].concat();
                if !pk.verify(msg, &remote_payload[40..]) {
                    Err(InvalidSignature)
                } else {
                    let pk = pk.try_into_ed25519().map_err(|_| BadPublicKey)?;
                    let remote_pk =
                        PublicKey::from_bytes(pk.to_bytes()).map_err(|_| BadPublicKey)?;
                    Ok(ResponderOutput::Done(output, remote_pk))
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
