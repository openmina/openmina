use mina_p2p_messages::v2::{
    self, BlockTimeTimeStableV1,
    ConsensusProofOfStakeDataConsensusStateValueStableV2 as MinaConsensusState, StateHash,
};
use serde::{Deserialize, Serialize};

use crate::constants::constraint_constants;
pub use crate::constants::{
    checkpoint_window_size_in_slots, grace_period_end, slots_per_window, CHECKPOINTS_PER_YEAR,
};

// TODO get constants from elsewhere
const GRACE_PERIOD_END: u32 = 1440;
const SUB_WINDOWS_PER_WINDOW: u32 = 11;
const SLOTS_PER_SUB_WINDOW: u32 = 7;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConsensusShortRangeForkDecisionReason {
    ChainLength,
    Vrf,
    StateHash,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConsensusLongRangeForkDecisionReason {
    SubWindowDensity,
    ChainLength,
    Vrf,
    StateHash,
}

// TODO(binier): do we need to verify constants? Probably they are verified
// using block proof verification, but check just to be sure.
pub fn is_short_range_fork(a: &MinaConsensusState, b: &MinaConsensusState) -> bool {
    let check = |s1: &MinaConsensusState, s2: &MinaConsensusState| {
        let slots_per_epoch = s2.curr_global_slot_since_hard_fork.slots_per_epoch.as_u32();
        let s2_epoch_slot = s2.global_slot() % slots_per_epoch;
        if s1.epoch_count.as_u32() == s2.epoch_count.as_u32() + 1
            && s2_epoch_slot >= slots_per_epoch * 2 / 3
        {
            crate::log::debug!(crate::log::system_time(); kind = "is_short_range_fork", msg = format!("s2 is 1 epoch behind and not in seed update range: {} vs {}", s1.staking_epoch_data.lock_checkpoint, s2.next_epoch_data.lock_checkpoint));
            // S1 is one epoch ahead of S2 and S2 is not in the seed update range
            s1.staking_epoch_data.lock_checkpoint == s2.next_epoch_data.lock_checkpoint
        } else {
            crate::log::debug!(crate::log::system_time(); kind = "is_short_range_fork", msg = format!("chains are from different epochs"));
            false
        }
    };

    crate::log::debug!(crate::log::system_time(); kind = "is_short_range_fork", msg = format!("epoch count: {} vs {}", a.epoch_count.as_u32(), b.epoch_count.as_u32()));
    if a.epoch_count == b.epoch_count {
        let a_prev_lock_checkpoint = &a.staking_epoch_data.lock_checkpoint;
        let b_prev_lock_checkpoint = &b.staking_epoch_data.lock_checkpoint;
        // Simple case: blocks have same previous epoch, so compare previous epochs' lock_checkpoints
        crate::log::debug!(crate::log::system_time(); kind = "is_short_range_fork", msg = format!("checkpoints: {} vs {}", a_prev_lock_checkpoint, b_prev_lock_checkpoint));
        a_prev_lock_checkpoint == b_prev_lock_checkpoint
    } else {
        // Check for previous epoch case using both orientations
        check(a, b) || check(b, a)
    }
}

/// Relative minimum window density.
///
/// See [specification](https://github.com/MinaProtocol/mina/tree/develop/docs/specs/consensus#5412-relative-minimum-window-density)
pub fn relative_min_window_density(b1: &MinaConsensusState, b2: &MinaConsensusState) -> u32 {
    use std::cmp::{max, min};

    let max_slot = max(global_slot(b1), global_slot(b2));

    if max_slot < GRACE_PERIOD_END {
        return b1.min_window_density.as_u32();
    }

    let projected_window = {
        // Compute shift count
        let shift_count = (max_slot - global_slot(b1) - 1).clamp(0, SUB_WINDOWS_PER_WINDOW);

        // Initialize projected window
        let mut projected_window = b1
            .sub_window_densities
            .iter()
            .map(|d| d.as_u32())
            .collect::<Vec<_>>();

        // Ring-shift
        let mut i = relative_sub_window_from_global_slot(global_slot(b1));
        for _ in 0..=shift_count {
            i = (i + 1) % SUB_WINDOWS_PER_WINDOW;
            projected_window[i as usize] = 0;
        }

        projected_window
    };

    let projected_window_density = density(projected_window);

    min(b1.min_window_density.as_u32(), projected_window_density)
}

fn density(projected_window: Vec<u32>) -> u32 {
    projected_window.iter().sum()
}

fn relative_sub_window_from_global_slot(global_slot: u32) -> u32 {
    (global_slot / SLOTS_PER_SUB_WINDOW) % SUB_WINDOWS_PER_WINDOW
}

fn global_slot(b: &MinaConsensusState) -> u32 {
    b.curr_global_slot_since_hard_fork.slot_number.as_u32()
}

pub fn short_range_fork_take(
    tip_cs: &MinaConsensusState,
    candidate_cs: &MinaConsensusState,
    tip_hash: &StateHash,
    candidate_hash: &StateHash,
) -> (bool, ConsensusShortRangeForkDecisionReason) {
    use std::cmp::Ordering::*;
    use ConsensusShortRangeForkDecisionReason::*;

    let tip_height = &tip_cs.blockchain_length;
    let candidate_height = &candidate_cs.blockchain_length;
    match candidate_height.cmp(tip_height) {
        Greater => return (true, ChainLength),
        Less => return (false, ChainLength),
        Equal => {}
    }

    let tip_vrf = tip_cs.last_vrf_output.blake2b();
    let candidate_vrf = candidate_cs.last_vrf_output.blake2b();
    match candidate_vrf.cmp(&tip_vrf) {
        Greater => return (true, Vrf),
        Less => return (false, Vrf),
        Equal => {}
    }

    (candidate_hash > tip_hash, StateHash)
}

pub fn long_range_fork_take(
    tip_cs: &MinaConsensusState,
    candidate_cs: &MinaConsensusState,
    tip_hash: &StateHash,
    candidate_hash: &StateHash,
) -> (bool, ConsensusLongRangeForkDecisionReason) {
    use std::cmp::Ordering::*;
    use ConsensusLongRangeForkDecisionReason::*;

    let tip_density = relative_min_window_density(tip_cs, candidate_cs);
    let candidate_density = relative_min_window_density(candidate_cs, tip_cs);
    match candidate_density.cmp(&tip_density) {
        Greater => return (true, SubWindowDensity),
        Less => return (false, SubWindowDensity),
        Equal => {}
    }

    let tip_height = &tip_cs.blockchain_length;
    let candidate_height = &candidate_cs.blockchain_length;
    match candidate_height.cmp(tip_height) {
        Greater => return (true, ChainLength),
        Less => return (false, ChainLength),
        Equal => {}
    }

    let tip_vrf = tip_cs.last_vrf_output.blake2b();
    let candidate_vrf = candidate_cs.last_vrf_output.blake2b();
    match candidate_vrf.cmp(&tip_vrf) {
        Greater => return (true, Vrf),
        Less => return (false, Vrf),
        Equal => {}
    }

    (candidate_hash > tip_hash, StateHash)
}

pub fn consensus_take(
    tip_cs: &MinaConsensusState,
    candidate_cs: &MinaConsensusState,
    tip_hash: &StateHash,
    candidate_hash: &StateHash,
) -> bool {
    if is_short_range_fork(tip_cs, candidate_cs) {
        short_range_fork_take(tip_cs, candidate_cs, tip_hash, candidate_hash).0
    } else {
        long_range_fork_take(tip_cs, candidate_cs, tip_hash, candidate_hash).0
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

pub fn global_sub_window(
    slot: &v2::ConsensusGlobalSlotStableV1,
    constants: &v2::MinaBaseProtocolConstantsCheckedValueStableV1,
) -> u32 {
    slot.slot_number.as_u32() / constants.slots_per_sub_window.as_u32()
}

pub fn relative_sub_window(global_sub_window: u32) -> u32 {
    global_sub_window % constraint_constants().sub_windows_per_window as u32
}

// TODO: Move ledger/src/scan_state/currency.rs types to core and replace
// primmitive types here with thoise numeric types.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusConstants {
    pub k: u32,
    pub delta: u32,
    pub block_window_duration_ms: u64,
    pub slots_per_sub_window: u32,
    pub slots_per_window: u32,
    pub sub_windows_per_window: u32,
    pub slots_per_epoch: u32,
    pub grace_period_slots: u32,
    pub grace_period_end: u32,
    pub slot_duration_ms: u64,
    pub epoch_duration: u64,
    pub checkpoint_window_slots_per_year: u32,
    pub checkpoint_window_size_in_slots: u32,
    pub delta_duration: u64,
    pub genesis_state_timestamp: BlockTimeTimeStableV1,
}

impl ConsensusConstants {
    // We mimick the code layout of the OCaml node's here. `create_primed` could easily
    // be inlined in `create`, but OCaml code keeps them separate ans so do we for now.
    fn create_primed(
        constraint_constants: &crate::constants::ConstraintConstants,
        protocol_constants: &v2::MinaBaseProtocolConstantsCheckedValueStableV1,
    ) -> Self {
        let delta = protocol_constants.delta.as_u32();
        let slots_per_epoch = protocol_constants.slots_per_epoch.as_u32();
        let slots_per_window = protocol_constants.slots_per_sub_window.as_u32()
            * constraint_constants.sub_windows_per_window as u32;
        let grace_period_end = protocol_constants.grace_period_slots.as_u32() + slots_per_window;
        let epoch_duration =
            (slots_per_epoch as u64) * constraint_constants.block_window_duration_ms;
        let delta_duration = constraint_constants.block_window_duration_ms * (delta + 1) as u64;
        Self {
            k: protocol_constants.k.as_u32(),
            delta,
            block_window_duration_ms: constraint_constants.block_window_duration_ms,
            slots_per_sub_window: protocol_constants.slots_per_sub_window.as_u32(),
            slots_per_window,
            sub_windows_per_window: constraint_constants.sub_windows_per_window as u32,
            slots_per_epoch,
            grace_period_slots: protocol_constants.grace_period_slots.as_u32(),
            grace_period_end,
            slot_duration_ms: constraint_constants.block_window_duration_ms,
            epoch_duration,
            checkpoint_window_slots_per_year: 0,
            checkpoint_window_size_in_slots: 0,
            delta_duration,
            genesis_state_timestamp: protocol_constants.genesis_state_timestamp,
        }
    }

    pub fn assert_invariants(&self) {
        let grace_period_effective_end = self.grace_period_end - self.slots_per_window;
        assert!(grace_period_effective_end < (self.slots_per_epoch / 3));
        // Because of how these values are computed (see below), this
        // fails if and only if block_window_duration is a multiple of
        // 27 or 512, or any of these multiplied by a power of 3 or 2
        // respectively.
        // 365 * 24 * 60 * 60 * 1000 = 2^10 * 3^3 * 5^6 * 73
        // Therefore, if divided by 2^9 or 3^3, the whole value will not be
        // divisible by 12 (2^2 * 3) anymore.
        assert_eq!(
            self.checkpoint_window_slots_per_year as u64,
            self.checkpoint_window_size_in_slots as u64 * CHECKPOINTS_PER_YEAR
        )
    }

    pub fn create(
        constraint_constants: &crate::constants::ConstraintConstants,
        protocol_constants: &v2::MinaBaseProtocolConstantsCheckedValueStableV1,
    ) -> Self {
        let mut constants = Self::create_primed(constraint_constants, protocol_constants);
        const MILLISECS_PER_YEAR: u64 = 365 * 24 * 60 * 60 * 1000;
        let slots_per_year = MILLISECS_PER_YEAR / constants.block_window_duration_ms;
        constants.checkpoint_window_slots_per_year = slots_per_year as u32;
        constants.checkpoint_window_size_in_slots = (slots_per_year / CHECKPOINTS_PER_YEAR) as u32;
        constants.assert_invariants();
        constants
    }
}

#[cfg(test)]
mod tests {
    use super::{long_range_fork_take, short_range_fork_take};
    use mina_p2p_messages::v2::{MinaStateProtocolStateValueStableV2, StateHash};
    macro_rules! fork_file {
        ($prefix:expr, $tip:expr, $cnd:expr, $suffix:expr) => {
            concat!(
                "../../tests/files/forks/",
                $prefix,
                "-",
                $tip,
                "-",
                $cnd,
                "-",
                $suffix,
                ".json"
            )
        };
    }
    macro_rules! fork_test {
        ($prefix:expr, $tip:expr, $cnd:expr, $func:ident, $decision:expr) => {
            let tip_str = include_str!(fork_file!($prefix, $tip, $cnd, "tip"));
            let cnd_str = include_str!(fork_file!($prefix, $tip, $cnd, "cnd"));
            let tip_hash = $tip.parse::<StateHash>().unwrap();
            let cnd_hash = $cnd.parse::<StateHash>().unwrap();
            let tip = serde_json::from_str::<MinaStateProtocolStateValueStableV2>(tip_str).unwrap();
            let cnd = serde_json::from_str::<MinaStateProtocolStateValueStableV2>(cnd_str).unwrap();

            let (take, _) = $func(
                &tip.body.consensus_state,
                &cnd.body.consensus_state,
                &tip_hash,
                &cnd_hash,
            );
            assert_eq!(take, $decision);
        };

        (long take $prefix:expr, $tip:expr, $cnd:expr) => {
            fork_test!(
                concat!("long-take-", $prefix),
                $tip,
                $cnd,
                long_range_fork_take,
                true
            );
        };

        (long keep $prefix:expr, $tip:expr, $cnd:expr) => {
            fork_test!(
                concat!("long-keep-", $prefix),
                $tip,
                $cnd,
                long_range_fork_take,
                false
            );
        };

        (short take $prefix:expr, $tip:expr, $cnd:expr) => {
            fork_test!(
                concat!("short-take-", $prefix),
                $tip,
                $cnd,
                short_range_fork_take,
                true
            );
        };

        (short keep $prefix:expr, $tip:expr, $cnd:expr) => {
            fork_test!(
                concat!("short-keep-", $prefix),
                $tip,
                $cnd,
                short_range_fork_take,
                false
            );
        };
    }

    #[test]
    fn long_range_fork() {
        fork_test!(
            long take
                "density-92-97",
            "3NLESd9gzU52bDWSXL5uUAYbCojHXSVdeBX4sCMF3V8Ns9D1Sriy",
            "3NLQfKJ4kBagLgmiwyiVw9zbi53tiNy8TNu2ua1jmCyEecgbBJoN"
        );
        fork_test!(
            long keep
                "density-161-166",
            "3NKY1kxHMRfjBbjfAA5fsasUCWFF9B7YqYFfNH4JFku6ZCUUXyLG",
            "3NLFoBQ6y3nku79LQqPgKBmuo5Ngnpr7rfZygzdRrcPtz2gewRFC"
        );
    }

    #[test]
    fn short_range_fork() {
        fork_test!(
            short take
                "length-60-61",
            "3NLQEb5mXqXCL34rueHrMkUVyWSQ7aYjvi6K98ZdpEnTozef69uR",
            "3NKuw8mvieV9RLpdRmHb4kxg7NWR83TfwzNkVmJCeHUmVWFdUQCp"
        );
        fork_test!(
            short take
                "vrf-99-99",
                "3NL4kAA33FRs9K66GvVNupNT94L4shALtYLHJRfmxhdZV8iPg2pi",
                "3NKC9F6mgtvRiHgYxiPBt1P5QDYaPVpD3YWyJhjmJZkNnT7RYitm"
        );
        fork_test!(
            short keep
                "vrf-117-117",
                "3NLWvDBFYJ2NXZ1EKMZXHB52zcbVtosHPArn4cGj8pDKkYsTHNnC",
                "3NKLEnUBTAhC95XEdJpLvJPqAUuvkC176tFKyLDcXUcofXXgQUvY"
        );
    }
}
