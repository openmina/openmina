use std::fmt::{self, Debug, Display, Formatter};
use std::num::ParseIntError;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone)]
pub struct ChainId([u8; 32]);

impl ChainId {
    pub fn as_hex(&self) -> String {
        format!("{}", self)
    }

    pub fn from_hex(s: &str) -> Result<ChainId, ParseIntError> {
        let mut bytes = [0u8; 32];
        for i in 0..32 {
            bytes[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)?;
        }
        Ok(ChainId(bytes))
    }

    pub fn from_bytes(bytes: &[u8]) -> ChainId {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes[..32]);
        ChainId(arr)
    }
}

impl Serialize for ChainId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.as_hex())
    }
}

impl<'de> Deserialize<'de> for ChainId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        ChainId::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

impl AsRef<[u8]> for ChainId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Display for ChainId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for b in self.0 {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

impl Debug for ChainId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "ChainId({})", self)
    }
}

pub const CHAIN_ID: ChainId = ChainId([
    0xfd, 0x7d, 0x11, 0x19, 0x73, 0xbf, 0x5a, 0x9e, 0x3e, 0x87, 0x38, 0x4f, 0x56, 0x0f, 0xde, 0xad,
    0x2f, 0x27, 0x25, 0x89, 0xca, 0x00, 0xb6, 0xd9, 0xe3, 0x57, 0xfc, 0xa9, 0x83, 0x96, 0x31, 0xda,
]);
