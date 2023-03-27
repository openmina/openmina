use std::{fmt, str::FromStr};

use ed25519_dalek::PublicKey as Ed25519PublicKey;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Clone)]
pub struct PublicKey(Ed25519PublicKey);

impl PublicKey {
    const BASE58_CHECK_VERSION: u8 = 0x16; // 'P'

    pub fn from_bytes(bytes: [u8; 32]) -> Result<Self, ed25519_dalek::SignatureError> {
        Ed25519PublicKey::from_bytes(&bytes).map(|v| Self(v))
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = bs58::encode(&self.to_bytes())
            .with_check_version(Self::BASE58_CHECK_VERSION)
            .into_string();
        write!(f, "{}", s)
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PublicKey({})", self)
    }
}

#[derive(thiserror::Error, Serialize, Deserialize, Debug, Clone)]
pub enum PublicKeyFromStrError {
    #[error("Base58 decode error: {0}")]
    Bs58(String),
    #[error("Ed25519 key uncompress error: {0}")]
    Ed25519(String),
}

impl FromStr for PublicKey {
    type Err = PublicKeyFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0u8; 37];
        let size = bs58::decode(s)
            .with_check(Some(Self::BASE58_CHECK_VERSION))
            .into(&mut bytes)
            .map_err(|err| PublicKeyFromStrError::Bs58(err.to_string()))?;
        if size != 33 {
            return Err(PublicKeyFromStrError::Bs58(
                bs58::decode::Error::BufferTooSmall.to_string(),
            ));
        }
        Self::from_bytes(bytes[1..33].try_into().unwrap())
            .map_err(|err| PublicKeyFromStrError::Ed25519(err.to_string()))
    }
}

impl From<PublicKey> for [u8; 32] {
    fn from(value: PublicKey) -> Self {
        value.to_bytes()
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_string())
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de> serde::Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let b58: String = Deserialize::deserialize(deserializer)?;
            Ok(b58.parse().map_err(|err| serde::de::Error::custom(err))?)
        } else {
            Ok(Self(Deserialize::deserialize(deserializer)?))
        }
    }
}
