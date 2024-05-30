use lazy_static::lazy_static;
use std::str::FromStr;

use binprot_derive::BinProtWrite;
use mina_hasher::Fp;
use mina_p2p_messages::{
    bigint, number,
    v2::{
        self, BlockTimeTimeStableV1, MinaBaseProtocolConstantsCheckedValueStableV1, StateHash,
        UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
    },
};

pub const GENESIS_PRODUCER_SK: &str = "EKFKgDtU3rcuFTVSEpmpXSkukjmX4cKefYREi6Sdsk7E7wsT7KRw";

pub const PROTOCOL_VERSION: v2::ProtocolVersionStableV2 = v2::ProtocolVersionStableV2 {
    transaction: number::Number(2),
    network: number::Number(0),
    patch: number::Number(0),
};

// TODO(tizoc): this should be configurable at compile time
pub const CONSTRAINT_CONSTANTS: ConstraintConstants = ConstraintConstants {
    sub_windows_per_window: 11,
    ledger_depth: 35,
    work_delay: 2,
    block_window_duration_ms: 180000,
    transaction_capacity_log_2: 7,
    pending_coinbase_depth: 5,
    coinbase_amount: 720000000000,
    supercharged_coinbase_factor: 1,
    account_creation_fee: 1000000000,
    fork: None,
};

#[derive(Clone, Debug)]
pub struct ForkConstants {
    pub previous_state_hash: Fp,
    pub previous_length: u32,
    pub previous_global_slot: u32,
}

#[derive(Clone, Debug)]
pub struct ConstraintConstants {
    pub sub_windows_per_window: u64,
    pub ledger_depth: u64,
    pub work_delay: u64,
    pub block_window_duration_ms: u64,
    pub transaction_capacity_log_2: u64,
    pub pending_coinbase_depth: usize,
    pub coinbase_amount: u64,
    pub supercharged_coinbase_factor: u64,
    pub account_creation_fee: u64,
    pub fork: Option<ForkConstants>,
}
#[derive(Clone, Debug, BinProtWrite)]
pub struct ForkConstantsUnversioned {
    previous_state_hash: bigint::BigInt,
    previous_length: number::Int32,
    previous_global_slot: number::Int32,
}

impl From<&ForkConstants> for ForkConstantsUnversioned {
    fn from(fork_constants: &ForkConstants) -> Self {
        Self {
            previous_state_hash: fork_constants.previous_state_hash.into(),
            previous_length: fork_constants.previous_length.into(),
            previous_global_slot: fork_constants.previous_global_slot.into(),
        }
    }
}

#[derive(Clone, Debug, BinProtWrite)]
pub struct ConstraintConstantsUnversioned {
    pub sub_windows_per_window: number::Int64,
    pub ledger_depth: number::Int64,
    pub work_delay: number::Int64,
    pub block_window_duration_ms: number::Int64,
    pub transaction_capacity_log_2: number::Int64,
    pub pending_coinbase_depth: number::Int64,
    pub coinbase_amount: number::UInt64,
    pub supercharged_coinbase_factor: number::Int64,
    pub account_creation_fee: number::UInt64,
    pub fork: Option<ForkConstantsUnversioned>,
}

impl From<&ConstraintConstants> for ConstraintConstantsUnversioned {
    fn from(constraints: &ConstraintConstants) -> Self {
        Self {
            sub_windows_per_window: constraints.sub_windows_per_window.into(),
            ledger_depth: constraints.ledger_depth.into(),
            work_delay: constraints.work_delay.into(),
            block_window_duration_ms: constraints.block_window_duration_ms.into(),
            transaction_capacity_log_2: constraints.transaction_capacity_log_2.into(),
            pending_coinbase_depth: (constraints.pending_coinbase_depth as u64).into(),
            coinbase_amount: constraints.coinbase_amount.into(),
            supercharged_coinbase_factor: constraints.supercharged_coinbase_factor.into(),
            account_creation_fee: constraints.account_creation_fee.into(),
            fork: constraints.fork.as_ref().map(|fork| fork.into()),
        }
    }
}

impl binprot::BinProtWrite for ConstraintConstants {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let constraints: ConstraintConstantsUnversioned = self.into();
        constraints.binprot_write(w)
    }
}

pub fn slots_per_window(constants: &v2::MinaBaseProtocolConstantsCheckedValueStableV1) -> u32 {
    constants.slots_per_sub_window.as_u32() * (CONSTRAINT_CONSTANTS.sub_windows_per_window as u32)
}

fn days_to_ms(days: u64) -> u64 {
    days * 24 * 60 * 60 * 1000
}

pub fn checkpoint_window_size_in_slots() -> u32 {
    let one_year_ms = days_to_ms(365);
    let slots_per_year = one_year_ms / CONSTRAINT_CONSTANTS.block_window_duration_ms;
    let size_in_slots = slots_per_year / 12;
    assert_eq!(slots_per_year % 12, 0);
    size_in_slots as u32
}

pub fn grace_period_end(constants: &v2::MinaBaseProtocolConstantsCheckedValueStableV1) -> u32 {
    let slots = {
        const NUM_DAYS: u64 = 3;
        let n_days_ms = days_to_ms(NUM_DAYS);
        let n_days = n_days_ms / CONSTRAINT_CONSTANTS.block_window_duration_ms;
        (n_days as u32).min(constants.slots_per_epoch.as_u32())
    };
    match CONSTRAINT_CONSTANTS.fork.as_ref() {
        None => slots,
        Some(fork) => slots + fork.previous_global_slot,
    }
}

pub const DEFAULT_GENESIS_TIMESTAMP_MILLISECONDS: u64 = 1706882461000;

pub const PROTOCOL_TRANSACTION_VERSION: u8 = 2;
pub const PROTOCOL_NETWORK_VERSION: u8 = 2;
pub const TX_POOL_MAX_SIZE: u32 = 3000;

pub const CONSTRAINT_SYSTEM_DIGESTS: [[u8; 16]; 3] = [
    [
        0xb8, 0x87, 0x9f, 0x67, 0x7f, 0x62, 0x2a, 0x1d, 0x86, 0x64, 0x80, 0x30, 0x70, 0x1f, 0x43,
        0xe1,
    ],
    [
        0xc2, 0xb8, 0x0e, 0x3b, 0x10, 0x21, 0xe7, 0x78, 0x1d, 0x76, 0x8c, 0xe0, 0x31, 0x7b, 0x3f,
        0x9c,
    ],
    [
        0x7c, 0xf1, 0x07, 0x1e, 0x7e, 0x55, 0xbe, 0x3f, 0x41, 0x93, 0xbb, 0xd8, 0xa6, 0x67, 0x91,
        0x4e,
    ],
];

// TODO: This should be computed from the genesis state, rather than hard-coded like this.
lazy_static! {
    pub static ref GENESIS_STATE_HASH: StateHash =
        StateHash::from_str("3NK512ryRJvj1TUKGgPoGZeHSNbn37e9BbnpyeqHL9tvKLeD8yrY").unwrap();
    pub static ref PROTOCOL_CONSTANTS: v2::MinaBaseProtocolConstantsCheckedValueStableV1 =
        MinaBaseProtocolConstantsCheckedValueStableV1 {
            k: 290.into(),
            slots_per_epoch: 7140.into(),
            slots_per_sub_window: 7.into(),
            grace_period_slots: 2160.into(),
            delta: 0.into(),
            genesis_state_timestamp: BlockTimeTimeStableV1(
                UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
                    DEFAULT_GENESIS_TIMESTAMP_MILLISECONDS.into(),
                ),
            ),
        };
}
