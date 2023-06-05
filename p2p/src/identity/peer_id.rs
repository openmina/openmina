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
            .map(u64::from_be_bytes);
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
        let mut bytes = [0u8; 37];

        let size = bs58::decode(s)
            .with_check(Some(Self::BASE58_CHECK_VERSION))
            .into(&mut bytes)?;
        if size != 33 {
            return Err(bs58::decode::Error::BufferTooSmall);
        }
        Ok(Self::from_bytes(bytes[1..33].try_into().unwrap()))
    }
}

impl From<PeerId> for [u8; 32] {
    fn from(value: PeerId) -> Self {
        value.to_bytes()
    }
}

impl From<libp2p::PeerId> for PeerId {
    fn from(value: libp2p::PeerId) -> Self {
        let protobuf = value.as_ref().digest();
        let key = libp2p::identity::PublicKey::from_protobuf_encoding(protobuf).unwrap();
        let bytes = key.into_ed25519().unwrap().encode();
        PeerId::from_bytes(bytes)
    }
}

impl From<PeerId> for libp2p::PeerId {
    fn from(value: PeerId) -> Self {
        let key = libp2p::identity::ed25519::PublicKey::decode(&value.to_bytes()).unwrap();
        let key = libp2p::identity::PublicKey::Ed25519(key);
        key.into()
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
            Ok(b58.parse().map_err(serde::de::Error::custom)?)
        } else {
            Ok(Self(Deserialize::deserialize(deserializer)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_id_bs58() {
        let s = "2bEgBrPTzL8wov2D4Kz34WVLCxR4uCarsBmHYXWKQA5wvBQzd9H";
        let id: PeerId = s.parse().unwrap();
        assert_eq!(s, id.to_string());
    }

    #[test]
    fn test_libp2p_peer_id_conv() {
        let s = "12D3KooWEiGVAFC7curXWXiGZyMWnZK9h8BKr88U8D5PKV3dXciv";
        let id: libp2p::PeerId = s.parse().unwrap();
        let conv: PeerId = id.into();
        let id_conv: libp2p::PeerId = conv.into();
        assert_eq!(id_conv, id);
    }
}
