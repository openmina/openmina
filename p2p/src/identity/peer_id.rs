use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use super::PublicKey;

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub struct PeerId([u64; 4]);

impl PeerId {
    const BASE58_CHECK_VERSION: u8 = 0x2F; // 'p'

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        let mut iter = bytes
            .chunks(8)
            .map(|v| <[u8; 8]>::try_from(v).unwrap())
            .map(|b| u64::from_be_bytes(b));
        Self([
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
        ])
    }

    pub fn to_bytes(self) -> [u8; 32] {
        // Not the most optimal way.
        self.0
            .into_iter()
            .flat_map(|v| v.to_be_bytes())
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap()
    }

    pub fn from_public_key(key: PublicKey) -> Self {
        Self::from_bytes(key.to_bytes())
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = bs58::encode(&self.to_bytes())
            .with_check_version(Self::BASE58_CHECK_VERSION)
            .into_string();
        write!(f, "{}", s)
    }
}

impl fmt::Debug for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PeerId({})", self)
    }
}

impl FromStr for PeerId {
    type Err = bs58::decode::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0u8; 32];
        let size = bs58::decode(s)
            .with_check(Some(Self::BASE58_CHECK_VERSION))
            .into(&mut bytes)?;
        if size != 32 {
            return Err(bs58::decode::Error::BufferTooSmall);
        }
        Ok(Self::from_bytes(bytes))
    }
}

impl From<PeerId> for [u8; 32] {
    fn from(value: PeerId) -> Self {
        value.to_bytes()
    }
}

impl Serialize for PeerId {
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

impl<'de> serde::Deserialize<'de> for PeerId {
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
