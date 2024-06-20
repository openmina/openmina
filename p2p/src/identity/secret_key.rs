use std::{fmt, str::FromStr};

use ed25519_dalek::SigningKey as Ed25519SecretKey;
use rand::Rng;
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
        Ok(Self::from_bytes(bytes[1..33].try_into().unwrap()))
    }
}

#[cfg(feature = "p2p-libp2p")]
impl From<SecretKey> for libp2p_identity::Keypair {
    fn from(value: SecretKey) -> Self {
        Self::ed25519_from_bytes(value.to_bytes()).unwrap()
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
