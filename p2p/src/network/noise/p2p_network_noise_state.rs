use std::collections::VecDeque;

use redux::ActionMeta;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::Zeroize;

use chacha20poly1305::ChaCha20Poly1305;
use sha2::Sha256;
use vru_noise::{
    generic_array::{typenum, GenericArray},
    hkdf::hmac::Hmac,
    ChainingKey, OutputRaw, SymmetricState,
};

use crate::{identity::PublicKey, P2pCryptoService, P2pMioService};

use super::{super::*, *};

type C = (Hmac<Sha256>, Sha256, typenum::B0, ChaCha20Poly1305);

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkNoiseState {
    pub buffer: Vec<u8>,
    pub incoming_chunks: VecDeque<Vec<u8>>,
    pub outgoing_chunks: VecDeque<Vec<u8>>,

    pub inner: Option<P2pNetworkNoiseStateInner>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkNoiseStateInner {
    Initiator(P2pNetworkNoiseStateInitiator),
    Responder(P2pNetworkNoiseStateResponder),
    Done {
        output: OutputRaw<C>,
        remote_pk: PublicKey,
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
    Init,
}

impl P2pNetworkNoiseAction {
    pub fn effects<Store, S>(&self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService + P2pCryptoService,
        P2pNetworkPnetOutgoingDataAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseIncomingChunkAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseOutgoingChunkAction: redux::EnablingCondition<S>,
    {
        let state = store.state();
        let Some(state) = state.network.connection.connections.get(&self.addr()) else {
            return;
        };
        let Some(P2pNetworkAuthState::Noise(state)) = &state.auth else {
            return;
        };

        let incoming = state.incoming_chunks.front().cloned().map(Into::into);
        let outgoing = state.outgoing_chunks.front().cloned().map(Into::into);

        if let Self::OutgoingChunk(a) = self {
            store.dispatch(P2pNetworkPnetOutgoingDataAction {
                addr: a.addr,
                data: a.data.clone(),
            });
        }
        if let Some(data) = incoming {
            store.dispatch(P2pNetworkNoiseIncomingChunkAction {
                addr: self.addr(),
                data,
            });
        }
        if let Some(data) = outgoing {
            store.dispatch(P2pNetworkNoiseOutgoingChunkAction {
                addr: self.addr(),
                data,
            });
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
                    Some(P2pNetworkNoiseStateInner::Responder(
                        P2pNetworkNoiseStateResponder::Init,
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
            }
            P2pNetworkNoiseAction::IncomingChunk(_) => {
                let Some(state) = &mut self.inner else {
                    return;
                };
                if let Some(chunk) = self.incoming_chunks.pop_front() {
                    match state {
                        P2pNetworkNoiseStateInner::Initiator(i) => match i.process(chunk) {
                            Ok((chunk, output, remote_pk)) => {
                                *state = P2pNetworkNoiseStateInner::Done { output, remote_pk };
                                self.outgoing_chunks.push_back(chunk);
                            }
                            Err(err) => *state = P2pNetworkNoiseStateInner::Error(err),
                        },
                        P2pNetworkNoiseStateInner::Responder(state) => {
                            if let Some(outgoing) = state.process(chunk) {
                                self.outgoing_chunks.push_back(outgoing);
                            }
                        }
                        P2pNetworkNoiseStateInner::Done { output, remote_pk } => {
                            let _ = (output, remote_pk);
                            unimplemented!();
                        }
                        P2pNetworkNoiseStateInner::Error(_) => {}
                    }
                }
            }
            P2pNetworkNoiseAction::OutgoingChunk(_) => {
                self.outgoing_chunks.pop_front();
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
        let remote_payload = msg[80..(len - 16)].to_vec().into_boxed_slice();

        let pk = libp2p_identity::PublicKey::try_decode_protobuf(&remote_payload[2..38])
            .map_err(|_| BadPublicKey)?;
        let msg = &[b"noise-libp2p-static-key:", r_spk.0.as_bytes().as_ref()].concat();
        if !pk.verify(msg, &remote_payload[40..]) {
            Err(InvalidSignature)
        } else {
            let pk = pk.try_into_ed25519().map_err(|_| BadPublicKey)?;
            let remote_pk = PublicKey::from_bytes(pk.to_bytes()).map_err(|_| BadPublicKey)?;
            let len = (chunk.len() - 2) as u16;
            chunk[..2].clone_from_slice(&len.to_be_bytes());
            Ok((chunk, output, remote_pk))
        }
    }
}

impl P2pNetworkNoiseStateResponder {
    fn process(&mut self, chunk: Vec<u8>) -> Option<Vec<u8>> {
        let _ = chunk;
        unimplemented!()
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
