use binprot_derive::BinProtWrite;
use mina_hasher::Fp;
use mina_p2p_messages::{bigint, number, v2};

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
    pub pending_coinbase_depth: u64,
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
            pending_coinbase_depth: constraints.pending_coinbase_depth.into(),
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

pub fn in_seed_update_range(
    slot: u32,
    constants: &v2::MinaBaseProtocolConstantsCheckedValueStableV1,
) -> bool {
    let third_epoch = constants.slots_per_epoch.as_u32() / 3;
    assert_eq!(constants.slots_per_epoch.as_u32(), third_epoch * 3);
    slot < third_epoch * 2
}

pub fn in_same_checkpoint_window(
    slot1: &v2::ConsensusGlobalSlotStableV1,
    slot2: &v2::ConsensusGlobalSlotStableV1,
) -> bool {
    checkpoint_window(slot1) == checkpoint_window(slot2)
}

pub fn checkpoint_window(slot: &v2::ConsensusGlobalSlotStableV1) -> u32 {
    slot.slot_number.as_u32() / checkpoint_window_size_in_slots()
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

pub fn global_sub_window(
    slot: &v2::ConsensusGlobalSlotStableV1,
    constants: &v2::MinaBaseProtocolConstantsCheckedValueStableV1,
) -> u32 {
    slot.slot_number.as_u32() / constants.slots_per_sub_window.as_u32()
}

pub fn relative_sub_window(global_sub_window: u32) -> u32 {
    global_sub_window % CONSTRAINT_CONSTANTS.sub_windows_per_window as u32
}
