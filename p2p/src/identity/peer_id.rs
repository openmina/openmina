use std::{fmt, str::FromStr};

use binprot::{BinProtRead, BinProtWrite, Nat0};
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

    pub fn to_public_key(self) -> Result<PublicKey, ed25519_dalek::SignatureError> {
        PublicKey::from_bytes(self.to_bytes())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn to_libp2p_string(self) -> String {
        libp2p_identity::PeerId::from(self).to_string()
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

#[derive(Clone, Debug, PartialEq, thiserror::Error, Serialize, Deserialize)]
pub enum PeerIdFromLibp2pPeerId {
    #[error("error decoding public key from protobuf: {0}")]
    Protobuf(String),
    #[error("error converting public key to ed25519: {0}")]
    Ed25519(String),
    #[error("peer_id with unsupported multihash code")]
    Code,
}

impl From<libp2p_identity::DecodingError> for PeerIdFromLibp2pPeerId {
    fn from(value: libp2p_identity::DecodingError) -> Self {
        PeerIdFromLibp2pPeerId::Protobuf(value.to_string())
    }
}

impl From<libp2p_identity::OtherVariantError> for PeerIdFromLibp2pPeerId {
    fn from(value: libp2p_identity::OtherVariantError) -> Self {
        PeerIdFromLibp2pPeerId::Ed25519(value.to_string())
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

impl TryFrom<&libp2p_identity::PeerId> for PeerId {
    type Error = PeerIdFromLibp2pPeerId;

    fn try_from(value: &libp2p_identity::PeerId) -> Result<Self, Self::Error> {
        let slice = value.as_ref().digest();
        if value.as_ref().code() == 0x12 {
            return Err(PeerIdFromLibp2pPeerId::Code);
        };
        let key = libp2p_identity::PublicKey::try_decode_protobuf(slice)?;
        let bytes = key.try_into_ed25519()?.to_bytes();
        Ok(PeerId::from_bytes(bytes))
    }
}

impl From<libp2p_identity::PeerId> for PeerId {
    fn from(value: libp2p_identity::PeerId) -> Self {
        let slice = value.as_ref().digest();
        if value.as_ref().code() == 0x12 {
            todo!("store such kind of key in our `PeerId`");
        } else {
            let key = libp2p_identity::PublicKey::try_decode_protobuf(slice).unwrap();
            let bytes = key.try_into_ed25519().unwrap().to_bytes();
            PeerId::from_bytes(bytes)
        }
    }
}

impl From<PeerId> for libp2p_identity::PeerId {
    fn from(value: PeerId) -> Self {
        let key = libp2p_identity::ed25519::PublicKey::try_from_bytes(&value.to_bytes()).unwrap();
        #[allow(deprecated)]
        let key = libp2p_identity::PublicKey::from(key);
        key.to_peer_id()
    }
}

impl PartialEq<libp2p_identity::PeerId> for PeerId {
    fn eq(&self, other: &libp2p_identity::PeerId) -> bool {
        let key = libp2p_identity::PublicKey::try_decode_protobuf(other.as_ref().digest()).unwrap();
        let bytes = key.try_into_ed25519().unwrap().to_bytes();
        self == &PeerId::from_bytes(bytes)
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

impl BinProtWrite for PeerId {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        for v in self.0 {
            Nat0(v).binprot_write(w)?;
        }
        Ok(())
    }
}

impl BinProtRead for PeerId {
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        let mut iter = std::iter::repeat(()).map(|_| Nat0::binprot_read(r));

        Ok(Self([
            iter.next().unwrap()?.0,
            iter.next().unwrap()?.0,
            iter.next().unwrap()?.0,
            iter.next().unwrap()?.0,
        ]))
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
        let id: libp2p_identity::PeerId = s.parse().unwrap();
        let conv: PeerId = id.into();
        let id_conv: libp2p_identity::PeerId = conv.into();
        assert_eq!(id_conv, id);
    }

    #[test]
    #[ignore = "doesn't work"]
    fn test_bare_base58btc_pk() {
        let s = "QmSXffHzFVSEoQCYBS1bPpCn4vgGEpQnCA9NLYuhamPBU3";
        let id: libp2p_identity::PeerId = s.parse().unwrap();
        let conv: PeerId = id.into();
        let id_conv: libp2p_identity::PeerId = conv.into();
        assert_eq!(id_conv, id);
    }
}
