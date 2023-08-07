use std::{fmt, str::FromStr};

use ed25519_dalek::SigningKey as Ed25519SecretKey;
use serde::{Deserialize, Serialize};

use crate::identity::PublicKey;

#[derive(Clone)]
pub struct SecretKey(Ed25519SecretKey);

impl SecretKey {
    const BASE58_CHECK_VERSION: u8 = 0x80;

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
