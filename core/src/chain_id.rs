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
    hex::encode(&hash)
}

type Md5 = [u8; 16];

fn hash_genesis_constants(
    constants: &MinaBaseProtocolConstantsCheckedValueStableV1,
    tx_pool_max_size: &UnsignedExtendedUInt32StableV1,
) -> [u8; 32] {
    let mut hasher = Blake2b256::default();
    let genesis_timestamp = OffsetDateTime::from_unix_timestamp_nanos(
        constants.genesis_state_timestamp.0 .0 .0 as i128,
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
            .map(|md5| hex::encode(md5))
            .reduce(|acc, el| acc + &el)
            .unwrap_or_else(|| String::new());
        let genesis_constants_hash = hash_genesis_constants(genesis_constants, tx_max_pool_size);
            hasher.update(genesis_state_hash.to_string().as_bytes());
            hasher.update(constraint_system_hash.to_string().as_bytes());
            hasher.update(hex::encode(&genesis_constants_hash).as_bytes());
            hasher.update(md5_hash(protocol_transaction_version).as_bytes());
            hasher.update(md5_hash(protocol_network_version).as_bytes());
        ChainId(hasher.finalize().try_into().unwrap())
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
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

pub const BERKELEY_CHAIN_ID: ChainId = ChainId([
    0xfd, 0x7d, 0x11, 0x19, 0x73, 0xbf, 0x5a, 0x9e, 0x3e, 0x87, 0x38, 0x4f, 0x56, 0x0f, 0xde, 0xad,
    0x2f, 0x27, 0x25, 0x89, 0xca, 0x00, 0xb6, 0xd9, 0xe3, 0x57, 0xfc, 0xa9, 0x83, 0x96, 0x31, 0xda,
]);

#[cfg(test)]
mod test {
    use super::*;
    use crate::constants::*;

    #[test]
    fn test_berkeley_chain_id() {
        // Compute the chain id for the Berkeley network and compare it the real one.
        let chain_id = ChainId::compute(
            CONSTRAINT_SYSTEM_DIGESTS.as_slice(),
            &GENESIS_STATE_HASH,
            &PROTOCOL_CONSTANTS,
            PROTOCOL_TRANSACTION_VERSION,
            PROTOCOL_NETWORK_VERSION,
            &UnsignedExtendedUInt32StableV1::from(TX_POOL_MAX_SIZE),
        );
        assert_eq!(chain_id, BERKELEY_CHAIN_ID);
    }

    #[test]
    fn test_chain_id_as_hex() {
        assert_eq!(
            BERKELEY_CHAIN_ID.to_hex(),
            "fd7d111973bf5a9e3e87384f560fdead2f272589ca00b6d9e357fca9839631da"
        );
    }
}
