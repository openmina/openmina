use std::collections::VecDeque;

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

use super::super::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkNoiseState {
    pub local_pk: PublicKey,
    pub buffer: Vec<u8>,
    pub incoming_chunks: Vec<Data>,
    pub outgoing_chunks: VecDeque<Vec<Data>>,
    pub decrypted_chunks: VecDeque<Data>,

    pub inner: Option<P2pNetworkNoiseStateInner>,
    pub handshake_optimized: bool,
    pub handshake_reported: bool,
}

impl P2pNetworkNoiseState {
    pub fn peer_id(&self) -> Option<&PeerId> {
        self.inner.as_ref().and_then(|inner| {
            if let P2pNetworkNoiseStateInner::Done { remote_peer_id, .. } = inner {
                Some(remote_peer_id)
            } else {
                None
            }
        })
    }
}

impl P2pNetworkNoiseState {
    pub fn new(local_pk: PublicKey, handshake_optimized: bool) -> Self {
        P2pNetworkNoiseState {
            local_pk,
            buffer: Default::default(),
            incoming_chunks: Default::default(),
            outgoing_chunks: Default::default(),
            decrypted_chunks: Default::default(),
            inner: Default::default(),
            handshake_optimized,
            handshake_reported: false,
        }
    }
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
    pub i_esk: Sk,
    pub i_spk: Pk,
    pub i_ssk: Sk,
    pub r_epk: Option<Pk>,
    pub payload: Data,
    pub noise: NoiseState,
    pub remote_pk: Option<PublicKey>,
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
                .chain(self.hash.0)
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
            .chain(self.hash.0)
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
            .chain(self.hash.0)
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

#[derive(Debug, Error, Serialize, Deserialize, Clone, PartialEq, Eq)]
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
    #[error("remote and local public keys are same")]
    SelfConnection,
}

pub struct ResponderOutput {
    pub send_key: DataSized<32>,
    pub recv_key: DataSized<32>,
    pub remote_pk: PublicKey,
}

impl P2pNetworkNoiseStateInitiator {
    pub fn generate(&mut self, data: &[u8]) -> Option<(Vec<u8>, (DataSized<32>, DataSized<32>))> {
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
        chunk.extend_from_slice(&payload);
        chunk.extend_from_slice(&payload_tag);
        let l = (chunk.len() - 2) as u16;
        chunk[..2].clone_from_slice(&l.to_be_bytes());

        Some((chunk, noise.finish()))
    }

    pub fn consume<'a>(
        &'_ mut self,
        chunk: &'a mut [u8],
    ) -> Result<Option<&'a mut [u8]>, NoiseError> {
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

        noise.mix_hash(r_epk.0.as_bytes());
        noise.mix_secret(&*i_esk * &r_epk);
        noise
            .decrypt::<0>(&mut r_spk_bytes, tag)
            .map_err(|_| FirstMacMismatch)?;

        let r_spk = Pk::from_bytes(r_spk_bytes);
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

pub struct ResponderConsumeOutput<'a> {
    pub output: ResponderOutput,
    pub payload: Option<&'a mut [u8]>,
}

impl P2pNetworkNoiseStateResponder {
    pub fn generate(&mut self, data: &[u8]) -> Option<Vec<u8>> {
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

        buffer.extend_from_slice(&payload);
        buffer.extend_from_slice(&payload_tag);
        let l = (buffer.len() - 2) as u16;
        buffer[..2].clone_from_slice(&l.to_be_bytes());

        let noise = noise.clone();
        let r_esk = r_esk.clone();
        let new_chunk = std::mem::take(buffer);

        *self = Self::Middle { r_esk, noise };

        Some(new_chunk)
    }

    pub fn consume<'a>(
        &'_ mut self,
        chunk: &'a mut [u8],
    ) -> Result<Option<ResponderConsumeOutput<'a>>, NoiseError> {
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

                let r_epk = r_esk.pk();

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
                    .decrypt::<1>(&mut i_spk_bytes, tag)
                    .map_err(|()| FirstMacMismatch)?;
                let i_spk = Pk::from_bytes(i_spk_bytes);
                noise.mix_secret(&*r_esk * &i_spk);
                r_esk.zeroize();

                noise
                    .decrypt::<0>(remote_payload, payload_tag)
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

                    Ok(Some(ResponderConsumeOutput {
                        output: ResponderOutput {
                            send_key,
                            recv_key,
                            remote_pk,
                        },
                        payload: remote_payload,
                    }))
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

    impl<'a, 'b> Mul<&'b Pk> for &'a Sk {
        type Output = [u8; 32];

        fn mul(self, rhs: &'b Pk) -> Self::Output {
            (self.0 * rhs.0).0
        }
    }

    #[derive(Debug, Clone)]
    pub struct Pk(pub MontgomeryPoint);

    impl Pk {
        pub fn from_bytes(bytes: [u8; 32]) -> Self {
            Pk(MontgomeryPoint(bytes))
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

    impl Sk {
        pub fn from_random(mut bytes: [u8; 32]) -> Self {
            bytes[0] &= 248;
            bytes[31] |= 64;
            #[allow(deprecated)]
            Self(Scalar::from_bits(bytes))
        }

        pub fn pk(&self) -> Pk {
            let t = curve25519_dalek::constants::ED25519_BASEPOINT_TABLE;
            Pk((t * &self.0).to_montgomery())
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
            #[allow(deprecated)]
            hex::decode(str)
                .map_err(Error::custom)
                .and_then(|b| b.try_into().map_err(|_| Error::custom("wrong length")))
                .map(Scalar::from_bits)
                .map(Self)
        }
    }
}
