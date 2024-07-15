use binprot_derive::BinProtWrite;
use mina_hasher::Fp;
use mina_p2p_messages::{bigint, number, v2};

pub const GENESIS_PRODUCER_SK: &str = "EKFKgDtU3rcuFTVSEpmpXSkukjmX4cKefYREi6Sdsk7E7wsT7KRw";

pub const PROTOCOL_VERSION: v2::ProtocolVersionStableV2 = v2::ProtocolVersionStableV2 {
    transaction: number::Number(3),
    network: number::Number(0),
    patch: number::Number(0),
};

pub fn constraint_constants() -> &'static ConstraintConstants {
    NetworkConfig::global().constraint_constants
}

#[derive(Clone, Debug)]
pub struct ForkConstants {
    pub state_hash: Fp,
    pub blockchain_length: u32,
    pub global_slot_since_genesis: u32,
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
    genesis_slot: number::Int32,
}

impl From<&ForkConstants> for ForkConstantsUnversioned {
    fn from(fork_constants: &ForkConstants) -> Self {
        Self {
            previous_state_hash: fork_constants.state_hash.into(),
            previous_length: fork_constants.blockchain_length.into(),
            genesis_slot: fork_constants.global_slot_since_genesis.into(),
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
    constants.slots_per_sub_window.as_u32() * (constraint_constants().sub_windows_per_window as u32)
}

fn days_to_ms(days: u64) -> u64 {
    days * 24 * 60 * 60 * 1000
}

pub fn checkpoint_window_size_in_slots() -> u32 {
    let one_year_ms = days_to_ms(365);
    let slots_per_year = one_year_ms / constraint_constants().block_window_duration_ms;
    let size_in_slots = slots_per_year / 12;
    assert_eq!(slots_per_year % 12, 0);
    size_in_slots as u32
}

pub fn grace_period_end(constants: &v2::MinaBaseProtocolConstantsCheckedValueStableV1) -> u32 {
    let slots = {
        const NUM_DAYS: u64 = 3;
        let n_days_ms = days_to_ms(NUM_DAYS);
        let n_days = n_days_ms / constraint_constants().block_window_duration_ms;
        (n_days as u32).min(constants.slots_per_epoch.as_u32())
    };
    match constraint_constants().fork.as_ref() {
        None => slots,
        Some(fork) => slots + fork.global_slot_since_genesis,
    }
}

pub const DEFAULT_GENESIS_TIMESTAMP_MILLISECONDS: u64 = 1707157200000;

pub const PROTOCOL_TRANSACTION_VERSION: u8 = 3;
pub const PROTOCOL_NETWORK_VERSION: u8 = 3;
pub const TX_POOL_MAX_SIZE: u32 = 3000;

pub use v2::PROTOCOL_CONSTANTS;

use crate::NetworkConfig;
