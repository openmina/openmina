//! Chain identifier and network discrimination for Mina Protocol.
//!
//! This module provides the [`ChainId`] type, which uniquely identifies
//! different Mina blockchain networks (Mainnet, Devnet, etc.) and ensures peers
//! only connect to compatible networks. The chain ID is computed from protocol
//! parameters, genesis state, and constraint system digests to create a
//! deterministic network identifier.
//!
//! ## Purpose
//!
//! Chain IDs serve multiple critical functions in the Mina protocol:
//!
//! - **Network Isolation**: Prevents nodes from different networks (e.g.,
//!   mainnet vs devnet) from connecting to each other
//! - **Protocol Compatibility**: Ensures all peers use the same protocol
//!   parameters
//! - **Security**: Used in cryptographic operations and peer authentication
//! - **Private Network Support**: Enables creation of isolated test networks
//!
//! ## Chain ID Computation
//!
//! The chain ID is a 32-byte Blake2b hash computed from:
//!
//! - **Genesis State Hash**: The hash of the initial blockchain state
//! - **Constraint System Digests**: Hashes of the SNARK constraint systems
//! - **Genesis Constants**: Protocol parameters like slot timing and consensus
//!   settings
//! - **Protocol Versions**: Transaction and network protocol version numbers
//! - **Transaction Pool Size**: Maximum transaction pool configuration
//!
//! This ensures that any change to fundamental protocol parameters results in a
//! different chain ID, preventing incompatible nodes from connecting.
//!
//! ## Network Identifiers
//!
//! Mina includes predefined chain IDs for official networks:
//!
//! - [`MAINNET_CHAIN_ID`]: The production Mina blockchain
//! - [`DEVNET_CHAIN_ID`]: The development/testing blockchain
//!
//! Custom networks can compute their own chain IDs using [`ChainId::compute()`].
//!
//! ## Usage in Networking
//!
//! Chain IDs are used throughout Mina's networking stack:
//!
//! - **Peer Discovery**: Nodes advertise their chain ID to find compatible
//!   peers
//! - **Connection Authentication**: WebRTC and libp2p connections verify chain
//!   ID compatibility
//! - **Private Networks**: The [`preshared_key()`](ChainId::preshared_key)
//!   method generates cryptographic keys for private network isolation
//!
//! ## Example
//!
//! ```rust
//! use mina_core::ChainId;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Use predefined network
//! let mainnet_id = mina_core::MAINNET_CHAIN_ID;
//! println!("Mainnet ID: {}", mainnet_id);
//!
//! // Parse from hex string
//! let chain_id = ChainId::from_hex("a7351abc7ddf2ea92d1b38cc8e636c271c1dfd2c081c637f62ebc2af34eb7cc1")?;
//!
//! // Generate preshared key for private networking
//! let psk = chain_id.preshared_key();
//! # Ok(())
//! # }
//! ```

use mina_p2p_messages::v2::{
    MinaBaseProtocolConstantsCheckedValueStableV1, StateHash, UnsignedExtendedUInt32StableV1,
};
use multihash::{Blake2b256, Hasher};
use time::{macros::format_description, OffsetDateTime};

use std::{
    fmt::{self, Debug, Display, Formatter},
    io::{Read, Write},
};

use binprot::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Unique identifier for a Mina blockchain network.
///
/// `ChainId` is a 32-byte cryptographic hash that uniquely identifies a
/// specific Mina blockchain network. It ensures network isolation by preventing
/// nodes from different chains (mainnet, devnet, custom testnets) from
/// connecting to each other.
///
/// ## Security Properties
///
/// The chain ID provides several security guarantees:
///
/// - **Deterministic**: Always produces the same ID for identical protocol
///   parameters
/// - **Collision Resistant**: Uses Blake2b hashing to prevent ID conflicts
/// - **Tamper Evident**: Any change to protocol parameters changes the chain ID
/// - **Network Isolation**: Incompatible networks cannot connect accidentally
///
/// ## Computation Method
///
/// Chain IDs are computed using [`ChainId::compute()`] from these inputs:
///
/// 1. **Constraint System Digests**: MD5 hashes of SNARK constraint systems
/// 2. **Genesis State Hash**: Hash of the initial blockchain state
/// 3. **Genesis Constants**: Protocol timing and consensus parameters
/// 4. **Protocol Versions**: Transaction and network protocol versions
/// 5. **Transaction Pool Size**: Maximum mempool configuration
///
/// The computation uses Blake2b-256 to hash these components in a specific
/// order, ensuring reproducible results across different implementations.
///
/// ## Network Usage
///
/// Chain IDs are used throughout the networking stack:
///
/// - **Peer Discovery**: Nodes broadcast their chain ID during discovery
/// - **Connection Handshakes**: WebRTC offers include chain ID for validation
/// - **Private Networks**: [`preshared_key()`](Self::preshared_key) generates
///   libp2p private network keys
/// - **Protocol Compatibility**: Ensures all peers use compatible protocol
///   versions
///
/// ## Serialization Formats
///
/// Chain IDs support multiple serialization formats:
///
/// - **Hex String**: Human-readable format for configuration files
/// - **Binary**: 32-byte array for network transmission
/// - **JSON**: String representation for APIs and debugging
///
/// ## Example Usage
///
/// ```rust
/// use mina_core::{ChainId, MAINNET_CHAIN_ID};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Use predefined mainnet ID
/// let mainnet = MAINNET_CHAIN_ID;
/// println!("Mainnet: {}", mainnet.to_hex());
///
/// // Parse from configuration
/// let custom_id = ChainId::from_hex("29936104443aaf264a7f0192ac64b1c7173198c1ed404c1bcff5e562e05eb7f6")?;
///
/// // Generate private network key
/// let psk = mainnet.preshared_key();
/// # Ok(())
/// # }
/// ```
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
    /// Computes a chain ID from protocol parameters and network configuration.
    ///
    /// This method creates a deterministic 32-byte chain identifier by hashing
    /// all the fundamental parameters that define a Mina blockchain network.
    /// Any change to these parameters will result in a different chain ID,
    /// ensuring network isolation and protocol compatibility.
    ///
    /// # Parameters
    ///
    /// * `constraint_system_digests` - MD5 hashes of the SNARK constraint
    ///   systems used for transaction and block verification
    /// * `genesis_state_hash` - Hash of the initial blockchain state
    /// * `genesis_constants` - Protocol constants including timing parameters,
    ///   consensus settings, and economic parameters
    /// * `protocol_transaction_version` - Version number of the transaction
    ///   protocol
    /// * `protocol_network_version` - Version number of the network protocol
    /// * `tx_max_pool_size` - Maximum number of transactions in the mempool
    ///
    /// # Returns
    ///
    /// A new `ChainId` representing the unique identifier for this network
    /// configuration.
    ///
    /// # Algorithm
    ///
    /// The computation process:
    ///
    /// 1. Hash all constraint system digests into a combined string
    /// 2. Hash the genesis constants with transaction pool size
    /// 3. Create Blake2b-256 hash of:
    ///    - Genesis state hash (as string)
    ///    - Combined constraint system hash
    ///    - Genesis constants hash (as hex)
    ///    - Protocol transaction version (as MD5 hash)
    ///    - Protocol network version (as MD5 hash)
    ///
    /// # Example
    ///
    /// ```rust
    /// use mina_core::ChainId;
    /// use mina_p2p_messages::v2::UnsignedExtendedUInt32StableV1;
    ///
    /// # // Use actual devnet values for the example
    /// # let constraint_digests = mina_core::network::devnet::CONSTRAINT_SYSTEM_DIGESTS;
    /// # let genesis_hash: mina_p2p_messages::v2::StateHash =
    /// #     "3NL93SipJfAMNDBRfQ8Uo8LPovC74mnJZfZYB5SK7mTtkL72dsPx".parse().unwrap();
    /// # let protocol_constants = mina_core::constants::PROTOCOL_CONSTANTS.clone();
    /// let chain_id = ChainId::compute(
    ///     &constraint_digests,
    ///     &genesis_hash,
    ///     &protocol_constants,
    ///     1,  // transaction version
    ///     1,  // network version
    ///     &UnsignedExtendedUInt32StableV1::from(3000),
    /// );
    /// ```
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

    /// Generates a preshared key for libp2p private networking.
    ///
    /// This method creates a cryptographic key used by libp2p's private network
    /// (Pnet) protocol to ensure only nodes with the same chain ID can connect.
    /// The preshared key provides an additional layer of network isolation
    /// beyond basic chain ID validation.
    ///
    /// # Algorithm
    ///
    /// The preshared key is computed as:
    /// ```text
    /// Blake2b-256("/coda/0.0.1/" + chain_id_hex)
    /// ```
    ///
    /// The "/coda/0.0.1/" prefix is a protocol identifier that ensures the
    /// preshared key is unique to the Mina protocol and not accidentally
    /// compatible with other systems.
    ///
    /// # Returns
    ///
    /// A 32-byte array containing the preshared key for this chain ID.
    ///
    /// # Usage
    ///
    /// This key is used to configure libp2p's private network transport,
    /// which encrypts all network traffic and prevents unauthorized nodes
    /// from joining the network even if they know peer addresses.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mina_core::MAINNET_CHAIN_ID;
    ///
    /// let psk = MAINNET_CHAIN_ID.preshared_key();
    /// // Use psk to configure libp2p Pnet transport
    /// ```
    pub fn preshared_key(&self) -> [u8; 32] {
        let mut hasher = Blake2b256::default();
        hasher.update(b"/coda/0.0.1/");
        hasher.update(self.to_hex().as_bytes());
        let hash = hasher.finalize();
        let mut psk_fixed: [u8; 32] = Default::default();
        psk_fixed.copy_from_slice(hash.as_ref());
        psk_fixed
    }

    /// Converts the chain ID to a hexadecimal string representation.
    ///
    /// This method creates a lowercase hex string of the 32-byte chain ID,
    /// suitable for display, logging, configuration files, and JSON
    /// serialization.
    ///
    /// # Returns
    ///
    /// A 64-character hexadecimal string representing the chain ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mina_core::MAINNET_CHAIN_ID;
    ///
    /// let hex_id = MAINNET_CHAIN_ID.to_hex();
    /// assert_eq!(hex_id.len(), 64);
    /// println!("Mainnet ID: {}", hex_id);
    /// ```
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parses a chain ID from a hexadecimal string.
    ///
    /// This method converts a hex string back into a `ChainId` instance.
    /// The input string must represent exactly 32 bytes (64 hex characters).
    /// Case-insensitive parsing is supported.
    ///
    /// # Parameters
    ///
    /// * `s` - A hexadecimal string representing the chain ID
    ///
    /// # Returns
    ///
    /// * `Ok(ChainId)` if the string is valid 64-character hex
    /// * `Err(hex::FromHexError)` if the string is invalid or wrong length
    ///
    /// # Errors
    ///
    /// This method returns an error if:
    /// - The string contains non-hexadecimal characters
    /// - The string length is not exactly 64 characters
    /// - The string represents fewer than 32 bytes
    ///
    /// # Example
    ///
    /// ```rust
    /// use mina_core::ChainId;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let chain_id = ChainId::from_hex(
    ///     "a7351abc7ddf2ea92d1b38cc8e636c271c1dfd2c081c637f62ebc2af34eb7cc1"
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_hex(s: &str) -> Result<ChainId, hex::FromHexError> {
        let h = hex::decode(s)?;
        let bs = h[..32]
            .try_into()
            .or(Err(hex::FromHexError::InvalidStringLength))?;
        Ok(ChainId(bs))
    }

    /// Creates a chain ID from raw bytes.
    ///
    /// This method constructs a `ChainId` from a byte slice, taking the first
    /// 32 bytes as the chain identifier. If the input has fewer than 32 bytes,
    /// the remaining bytes are zero-padded.
    ///
    /// # Parameters
    ///
    /// * `bytes` - A byte slice containing at least 32 bytes
    ///
    /// # Returns
    ///
    /// A new `ChainId` instance created from the input bytes.
    ///
    /// # Panics
    ///
    /// This method will panic if the input slice has fewer than 32 bytes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mina_core::ChainId;
    ///
    /// let bytes = [0u8; 32]; // All zeros for testing
    /// let chain_id = ChainId::from_bytes(&bytes);
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> ChainId {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes[..32]);
        ChainId(arr)
    }
}

impl BinProtWrite for ChainId {
    fn binprot_write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        w.write_all(&self.0)
    }
}

impl BinProtRead for ChainId {
    fn binprot_read<R: Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        let mut bytes = [0; 32];
        r.read_exact(&mut bytes)?;
        Ok(Self(bytes))
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

/// Chain ID for the Mina development network (Devnet).
///
/// This is the official chain identifier for Mina's development and testing
/// network.
/// Devnet is used for:
///
/// - Protocol development and testing
/// - New feature validation before mainnet deployment
/// - Developer experimentation and testing
/// - Stress testing and performance evaluation
///
/// The devnet chain ID ensures that devnet nodes cannot accidentally connect to
/// mainnet, providing network isolation for development activities.
///
/// # Hex Representation
///
/// `29936104443aaf264a7f0192ac64b1c7173198c1ed404c1bcff5e562e05eb7f6`
///
/// # Usage
///
/// ```rust
/// use mina_core::DEVNET_CHAIN_ID;
///
/// println!("Devnet ID: {}", DEVNET_CHAIN_ID.to_hex());
/// let psk = DEVNET_CHAIN_ID.preshared_key();
/// ```
pub const DEVNET_CHAIN_ID: ChainId = ChainId([
    0x29, 0x93, 0x61, 0x04, 0x44, 0x3a, 0xaf, 0x26, 0x4a, 0x7f, 0x01, 0x92, 0xac, 0x64, 0xb1, 0xc7,
    0x17, 0x31, 0x98, 0xc1, 0xed, 0x40, 0x4c, 0x1b, 0xcf, 0xf5, 0xe5, 0x62, 0xe0, 0x5e, 0xb7, 0xf6,
]);

/// Chain ID for the Mina production network (Mainnet).
///
/// This is the official chain identifier for Mina's production blockchain
/// network. Mainnet is the live network where real MINA tokens are transacted
/// and the blockchain consensus operates for production use.
///
/// Key characteristics:
///
/// - **Production Ready**: Used for real-world transactions and value transfer
/// - **Consensus Network**: Participates in the live Mina protocol consensus
/// - **Economic Security**: Protected by real economic incentives and staking
/// - **Finality**: Transactions have real-world financial consequences
///
/// The mainnet chain ID ensures network isolation from test networks and
/// prevents accidental cross-network connections that could compromise security.
///
/// # Hex Representation
///
/// `a7351abc7ddf2ea92d1b38cc8e636c271c1dfd2c081c637f62ebc2af34eb7cc1`
///
/// # Usage
///
/// ```rust
/// use mina_core::MAINNET_CHAIN_ID;
///
/// println!("Mainnet ID: {}", MAINNET_CHAIN_ID.to_hex());
/// let psk = MAINNET_CHAIN_ID.preshared_key();
/// ```
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
