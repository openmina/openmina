use mina_p2p_messages::v2::{
    MinaBaseProtocolConstantsCheckedValueStableV1, StateHash, UnsignedExtendedUInt32StableV1,
};
use multihash::{Blake2b256, Hasher};
use time::macros::format_description;
use time::OffsetDateTime;

use std::fmt::{self, Debug, Display, Formatter};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, PartialEq, Eq)]
pub struct ChainId([u8; 32]);

fn md5_hash(data: u8) -> String {
    let mut hasher = md5::Context::new();
    hasher.consume(data.to_string().as_bytes());
    let hash: Md5 = *hasher.compute();
    hex::encode(hash)
}

type Md5 = [u8; 16];

fn hash_genesis_constants(
    constants: &MinaBaseProtocolConstantsCheckedValueStableV1,
    tx_pool_max_size: &UnsignedExtendedUInt32StableV1,
) -> [u8; 32] {
    let mut hasher = Blake2b256::default();
    let genesis_timestamp = OffsetDateTime::from_unix_timestamp_nanos(
        (constants.genesis_state_timestamp.as_u64() * 1000000) as i128,
    )
    .unwrap();
    let time_format =
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:6]Z");
    hasher.update(constants.k.to_string().as_bytes());
    hasher.update(constants.slots_per_epoch.to_string().as_bytes());
    hasher.update(constants.slots_per_sub_window.to_string().as_bytes());
    hasher.update(constants.delta.to_string().as_bytes());
    hasher.update(tx_pool_max_size.to_string().as_bytes());
    hasher.update(genesis_timestamp.format(&time_format).unwrap().as_bytes());
    hasher.finalize().try_into().unwrap()
}

impl ChainId {
    pub fn compute(
        constraint_system_digests: &[Md5],
        genesis_state_hash: &StateHash,
        genesis_constants: &MinaBaseProtocolConstantsCheckedValueStableV1,
        protocol_transaction_version: u8,
        protocol_network_version: u8,
        tx_max_pool_size: &UnsignedExtendedUInt32StableV1,
    ) -> ChainId {
        let mut hasher = Blake2b256::default();
        let constraint_system_hash = constraint_system_digests
            .iter()
            .map(hex::encode)
            .reduce(|acc, el| acc + &el)
            .unwrap_or_default();
        let genesis_constants_hash = hash_genesis_constants(genesis_constants, tx_max_pool_size);
        hasher.update(genesis_state_hash.to_string().as_bytes());
        hasher.update(constraint_system_hash.to_string().as_bytes());
        hasher.update(hex::encode(genesis_constants_hash).as_bytes());
        hasher.update(md5_hash(protocol_transaction_version).as_bytes());
        hasher.update(md5_hash(protocol_network_version).as_bytes());
        ChainId(hasher.finalize().try_into().unwrap())
    }

    /// Computes shared key for libp2p Pnet protocol.
    pub fn preshared_key(&self) -> [u8; 32] {
        let mut hasher = Blake2b256::default();
        hasher.update(b"/coda/0.0.1/");
        hasher.update(self.to_hex().as_bytes());
        let hash = hasher.finalize();
        let mut psk_fixed: [u8; 32] = Default::default();
        psk_fixed.copy_from_slice(hash.as_ref());
        psk_fixed
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_hex(s: &str) -> Result<ChainId, hex::FromHexError> {
        let h = hex::decode(s)?;
        let bs = h[..32]
            .try_into()
            .or(Err(hex::FromHexError::InvalidStringLength))?;
        Ok(ChainId(bs))
    }

    pub fn from_bytes(bytes: &[u8]) -> ChainId {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes[..32]);
        ChainId(arr)
    }
}

impl Serialize for ChainId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
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
        write!(f, "{}", self.to_hex())?;
        Ok(())
    }
}

impl Debug for ChainId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "ChainId({})", self)
    }
}

pub const DEVNET_CHAIN_ID: ChainId = ChainId([
    0x29, 0x93, 0x61, 0x04, 0x44, 0x3a, 0xaf, 0x26, 0x4a, 0x7f, 0x01, 0x92, 0xac, 0x64, 0xb1, 0xc7,
    0x17, 0x31, 0x98, 0xc1, 0xed, 0x40, 0x4c, 0x1b, 0xcf, 0xf5, 0xe5, 0x62, 0xe0, 0x5e, 0xb7, 0xf6,
]);

pub const MAINNET_CHAIN_ID: ChainId = ChainId([
    0xa7, 0x35, 0x1a, 0xbc, 0x7d, 0xdf, 0x2e, 0xa9, 0x2d, 0x1b, 0x38, 0xcc, 0x8e, 0x63, 0x6c, 0x27,
    0x1c, 0x1d, 0xfd, 0x2c, 0x08, 0x1c, 0x63, 0x7f, 0x62, 0xeb, 0xc2, 0xaf, 0x34, 0xeb, 0x7c, 0xc1,
]);

#[cfg(test)]
mod test {
    use time::format_description::well_known::Rfc3339;

    use super::*;
    use crate::constants::*;

    #[test]
    fn test_devnet_chain_id() {
        // First block after fork: https://devnet.minaexplorer.com/block/3NL93SipJfAMNDBRfQ8Uo8LPovC74mnJZfZYB5SK7mTtkL72dsPx
        let genesis_state_hash = "3NL93SipJfAMNDBRfQ8Uo8LPovC74mnJZfZYB5SK7mTtkL72dsPx"
            .parse()
            .unwrap();

        let mut protocol_constants = PROTOCOL_CONSTANTS.clone();
        protocol_constants.genesis_state_timestamp =
            OffsetDateTime::parse("2024-04-09T21:00:00Z", &Rfc3339)
                .unwrap()
                .into();

        // Compute the chain id for the Devnet network and compare it the real one.
        let chain_id = ChainId::compute(
            crate::network::devnet::CONSTRAINT_SYSTEM_DIGESTS.as_slice(),
            &genesis_state_hash,
            &protocol_constants,
            PROTOCOL_TRANSACTION_VERSION,
            PROTOCOL_NETWORK_VERSION,
            &UnsignedExtendedUInt32StableV1::from(TX_POOL_MAX_SIZE),
        );
        assert_eq!(chain_id, DEVNET_CHAIN_ID);
    }

    #[test]
    fn test_mainnet_chain_id() {
        // First block after fork: https://www.minaexplorer.com/block/3NK4BpDSekaqsG6tx8Nse2zJchRft2JpnbvMiog55WCr5xJZaKeP
        let genesis_state_hash = "3NK4BpDSekaqsG6tx8Nse2zJchRft2JpnbvMiog55WCr5xJZaKeP"
            .parse()
            .unwrap();

        let mut protocol_constants = PROTOCOL_CONSTANTS.clone();
        protocol_constants.genesis_state_timestamp =
            OffsetDateTime::parse("2024-06-05T00:00:00Z", &Rfc3339)
                .unwrap()
                .into();

        // Compute the chain id for the Mainnet network and compare it the real one.
        let chain_id = ChainId::compute(
            crate::network::mainnet::CONSTRAINT_SYSTEM_DIGESTS.as_slice(),
            &genesis_state_hash,
            &protocol_constants,
            PROTOCOL_TRANSACTION_VERSION,
            PROTOCOL_NETWORK_VERSION,
            &UnsignedExtendedUInt32StableV1::from(TX_POOL_MAX_SIZE),
        );
        assert_eq!(chain_id, MAINNET_CHAIN_ID);
    }

    #[test]
    fn test_devnet_chain_id_as_hex() {
        assert_eq!(
            DEVNET_CHAIN_ID.to_hex(),
            "29936104443aaf264a7f0192ac64b1c7173198c1ed404c1bcff5e562e05eb7f6"
        );
    }

    #[test]
    fn test_mainnet_chain_id_as_hex() {
        assert_eq!(
            MAINNET_CHAIN_ID.to_hex(),
            "a7351abc7ddf2ea92d1b38cc8e636c271c1dfd2c081c637f62ebc2af34eb7cc1"
        );
    }
}
