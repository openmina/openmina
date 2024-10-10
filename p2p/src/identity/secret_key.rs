use std::{fmt, str::FromStr};

use ed25519_dalek::SigningKey as Ed25519SecretKey;
use rand::{CryptoRng, Rng};
use serde::{Deserialize, Serialize};

use crate::identity::PublicKey;

#[derive(Clone)]
pub struct SecretKey(Ed25519SecretKey);

impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SecretKey").field(&"***").finish()
    }
}

impl SecretKey {
    const BASE58_CHECK_VERSION: u8 = 0x80;

    pub fn rand() -> Self {
        Self::rand_with(&mut rand::thread_rng())
    }

    pub fn rand_with(mut rng: impl Rng) -> Self {
        Self::from_bytes(rng.gen())
    }

    pub fn deterministic(i: usize) -> Self {
        let mut bytes = [0; 32];
        let bytes_len = bytes.len();
        let i_bytes = i.to_be_bytes();
        let i = bytes_len - i_bytes.len();
        bytes[i..bytes_len].copy_from_slice(&i_bytes);
        Self::from_bytes(bytes)
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(Ed25519SecretKey::from_bytes(&bytes))
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.0.verifying_key())
    }

    pub fn to_x25519(&self) -> x25519_dalek::StaticSecret {
        self.0.to_scalar_bytes().into()
    }
}

use aes_gcm::{
    aead::{Aead, AeadCore, AeadInPlace},
    Aes256Gcm, KeyInit,
};
impl SecretKey {
    fn shared_key(&self, other_pk: &PublicKey) -> Result<Aes256Gcm, ()> {
        let key = self.to_x25519().diffie_hellman(&other_pk.to_x25519());
        if !key.was_contributory() {
            return Err(());
        }
        let key = key.to_bytes();
        let key: &aes_gcm::Key<Aes256Gcm> = (&key).into();
        Ok(Aes256Gcm::new(key))
    }

    pub fn encrypt_raw(
        &self,
        other_pk: &PublicKey,
        rng: impl Rng + CryptoRng,
        data: &[u8],
    ) -> Result<Vec<u8>, ()> {
        let shared_key = self.shared_key(other_pk)?;
        let nonce = Aes256Gcm::generate_nonce(rng);
        let mut buffer = Vec::from(AsRef::<[u8]>::as_ref(&nonce));
        shared_key
            .encrypt_in_place(&nonce, data, &mut buffer)
            .or(Err(()))?;
        Ok(buffer)
    }

    pub fn encrypt<M: EncryptableType>(
        &self,
        other_pk: &PublicKey,
        rng: impl Rng + CryptoRng,
        data: &M,
    ) -> Result<M::Encrypted, ()> {
        let data = serde_json::to_vec(data).or(Err(()))?;
        self.encrypt_raw(other_pk, rng, &data).map(Into::into)
    }

    pub fn decrypt_raw(&self, other_pk: &PublicKey, ciphertext: &[u8]) -> Result<Vec<u8>, ()> {
        let shared_key = self.shared_key(other_pk)?;
        let (nonce, ciphertext) = ciphertext.split_at_checked(12).ok_or(())?;
        shared_key.decrypt(nonce.into(), ciphertext).or(Err(()))
    }

    pub fn decrypt<M: EncryptableType>(
        &self,
        other_pk: &PublicKey,
        ciphertext: &M::Encrypted,
    ) -> Result<M, ()> {
        let data = self.decrypt_raw(other_pk, ciphertext.as_ref())?;
        serde_json::from_slice(&data).or(Err(()))
    }
}

pub trait EncryptableType: Serialize + for<'a> Deserialize<'a> {
    type Encrypted: From<Vec<u8>> + AsRef<[u8]>;
}

impl fmt::Display for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = bs58::encode(&self.to_bytes())
            .with_check_version(Self::BASE58_CHECK_VERSION)
            .into_string();
        write!(f, "{}", s)
    }
}

#[derive(thiserror::Error, Serialize, Deserialize, Debug, Clone)]
pub enum SecretKeyFromStrError {
    #[error("Base58 decode error: {0}")]
    Bs58(String),
}

impl FromStr for SecretKey {
    type Err = SecretKeyFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0u8; 37];
        let size = bs58::decode(s)
            .with_check(Some(Self::BASE58_CHECK_VERSION))
            .into(&mut bytes)
            .map_err(|err| SecretKeyFromStrError::Bs58(err.to_string()))?;
        if size != 33 {
            return Err(SecretKeyFromStrError::Bs58(
                bs58::decode::Error::BufferTooSmall.to_string(),
            ));
        }
        Ok(Self::from_bytes(
            bytes[1..33].try_into().expect("Size checked above"),
        ))
    }
}

#[cfg(feature = "p2p-libp2p")]
impl TryFrom<SecretKey> for libp2p_identity::Keypair {
    type Error = libp2p_identity::DecodingError;

    fn try_from(value: SecretKey) -> Result<Self, Self::Error> {
        Self::ed25519_from_bytes(value.to_bytes())
    }
}

impl Serialize for SecretKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for SecretKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let b58: String = Deserialize::deserialize(deserializer)?;
        b58.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::SecretKey;

    #[test]
    fn secret_key_to_string_roundtrip() {
        let s = "5K9G39rCgREFMCk7S739JZprwpMsiLRQXcELErUSwhHwfdVR8HT";
        let sk = s.parse::<SecretKey>().expect("should be parseable");
        let unparsed = sk.to_string();
        assert_eq!(s, &unparsed);
    }
}
