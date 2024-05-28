use binprot_derive::BinProtWrite;
use mina_hasher::Fp;
use mina_p2p_messages::{bigint, number, v2};

pub const GENESIS_PRODUCER_SK: &str = "EKFKgDtU3rcuFTVSEpmpXSkukjmX4cKefYREi6Sdsk7E7wsT7KRw";

pub const PROTOCOL_VERSION: v2::ProtocolVersionStableV2 = v2::ProtocolVersionStableV2 {
    transaction: number::Number(3),
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
    // TODO(tizoc): This should come from the config file, but
    // it affects the circuits. Since we cannot produce the circuits
    // ourselves right now, we cannot react to changes in this value,
    // so it will be hardcoded for now.
    fork: Some(ForkConstants {
        state_hash: ark_ff::field_new!(
            Fp,
            "7908066420535064797069631664846455037440232590837253108938061943122344055350"
        ),
        blockchain_length: 296371,
        global_slot_since_genesis: 445860,
    }),
};

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
    constants.slots_per_sub_window.as_u32() * (CONSTRAINT_CONSTANTS.sub_windows_per_window as u32)
}

const fn days_to_ms(days: u64) -> u64 {
    days * 24 * 60 * 60 * 1000
}

pub const CHECKPOINTS_PER_YEAR: u64 = 12;

pub fn checkpoint_window_size_in_slots() -> u32 {
    let one_year_ms = days_to_ms(365);
    let slots_per_year = one_year_ms / CONSTRAINT_CONSTANTS.block_window_duration_ms;
    let size_in_slots = slots_per_year / CHECKPOINTS_PER_YEAR;
    assert_eq!(slots_per_year % CHECKPOINTS_PER_YEAR, 0);
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
        Some(fork) => slots + fork.global_slot_since_genesis,
    }
}

pub const DEFAULT_GENESIS_TIMESTAMP_MILLISECONDS: u64 = 1707157200000;

pub const PROTOCOL_TRANSACTION_VERSION: u8 = 3;
pub const PROTOCOL_NETWORK_VERSION: u8 = 3;
pub const TX_POOL_MAX_SIZE: u32 = 3000;

pub const CONSTRAINT_SYSTEM_DIGESTS: [[u8; 16]; 3] = [
    [
        0xb8, 0x87, 0x9f, 0x67, 0x7f, 0x62, 0x2a, 0x1d, 0x86, 0x64, 0x80, 0x30, 0x70, 0x1f, 0x43,
        0xe1,
    ],
    [
        0x3b, 0xf6, 0xbb, 0x8a, 0x97, 0x66, 0x5f, 0xe7, 0xa9, 0xdf, 0x6f, 0xc1, 0x46, 0xe4, 0xf9,
        0x42,
    ],
    [
        0xd0, 0x24, 0xa9, 0xac, 0x78, 0xd4, 0xc9, 0x3a, 0x88, 0x8b, 0x63, 0xfc, 0x85, 0xee, 0xb6,
        0x6a,
    ],
];

pub use v2::PROTOCOL_CONSTANTS;
