///! Mina wire types, represented in Rust.
///!
///! This crate contains gossip network messages and RPCs.
pub mod bigint;
pub mod char;
pub mod common;
pub mod core;
pub mod gossip;
pub mod keys;
pub mod number;
pub mod phantom;
pub mod rpc;
pub mod rpc_kernel;
pub mod string;
pub mod utils;
#[macro_use]
pub mod versioned;
pub mod pseq;
pub mod v1;
pub mod v2;
#[cfg(feature = "hashing")]
mod hash_input;
#[cfg(feature = "hashing")]
pub mod hash;


pub use gossip::GossipNetMessageV1;

pub use rpc::JSONifyPayloadRegistry;
pub use rpc_kernel::JSONinifyPayloadReader;

mod b58;

pub mod b58version {
    pub const LEDGER_HASH: u8 = 0x05;
    pub const RECEIPT_CHAIN_HASH: u8 = 0x0c;
    pub const EPOCH_SEED: u8 = 0x0d;
    pub const STAGED_LEDGER_HASH_AUX_HASH: u8 = 0x0e;
    pub const STAGED_LEDGER_HASH_PENDING_COINBASE_AUX: u8 = 0x0f;
    pub const STATE_HASH: u8 = 0x10;
    pub const STATE_BODY_HASH: u8 = 0x11;
    pub const VRF_TRUNCATED_OUTPUT: u8 = 0x15;
    pub const COINBASE_STACK_DATA: u8 = 0x17;
    pub const COINBASE_STACK_HASH: u8 = 0x18;
    pub const PENDING_COINBASE_HASH_BUILDER: u8 = 0x19;
    pub const TOKEN_ID_KEY: u8 = 0x1c;
    pub const NON_ZERO_CURVE_POINT_COMPRESSED: u8 = 0xcb;
    pub const SIGNATURE: u8 = 0x9a;
}
