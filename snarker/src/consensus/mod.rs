mod consensus_state;
pub use consensus_state::*;

mod consensus_actions;
pub use consensus_actions::*;

mod consensus_reducer;
pub use consensus_reducer::*;

mod consensus_effects;
pub use consensus_effects::*;

use mina_p2p_messages::v2::{
    ConsensusProofOfStakeDataConsensusStateValueStableV2 as MinaConsensusState, StateHash,
};

// TODO get constants from elsewher
const GRACE_PERIOD_END: u32 = 1440;
const SUB_WINDOWS_PER_WINDOW: u32 = 11;
const SLOTS_PER_SUB_WINDOW: u32 = 7;

// TODO(binier): do we need to verify constants? Probably they are verified
// using block proof verification, but check just to be sure.
pub fn is_short_range_fork(a: &MinaConsensusState, b: &MinaConsensusState) -> bool {
    let check = |s1: &MinaConsensusState, s2: &MinaConsensusState| {
        let slots_per_epoch = s2.curr_global_slot.slots_per_epoch.as_u32();
        let s2_epoch_slot = s2.global_slot() % slots_per_epoch;
        if s1.epoch_count.as_u32() == s2.epoch_count.as_u32() + 1
            && s2_epoch_slot >= slots_per_epoch * 2 / 3
        {
            shared::log::debug!(shared::log::system_time(); kind = "is_short_range_fork", msg = format!("s2 is 1 epoch behind and not in seed update range: {} vs {}", s1.staking_epoch_data.lock_checkpoint, s2.next_epoch_data.lock_checkpoint));
            // S1 is one epoch ahead of S2 and S2 is not in the seed update range
            s1.staking_epoch_data.lock_checkpoint == s2.next_epoch_data.lock_checkpoint
        } else {
            shared::log::debug!(shared::log::system_time(); kind = "is_short_range_fork", msg = format!("chains are from different epochs"));
            false
        }
    };

    shared::log::debug!(shared::log::system_time(); kind = "is_short_range_fork", msg = format!("epoch count: {} vs {}", a.epoch_count.as_u32(), b.epoch_count.as_u32()));
    if a.epoch_count == b.epoch_count {
        let a_prev_lock_checkpoint = &a.staking_epoch_data.lock_checkpoint;
        let b_prev_lock_checkpoint = &b.staking_epoch_data.lock_checkpoint;
        // Simple case: blocks have same previous epoch, so compare previous epochs' lock_checkpoints
        shared::log::debug!(shared::log::system_time(); kind = "is_short_range_fork", msg = format!("checkpoints: {} vs {}", a_prev_lock_checkpoint, b_prev_lock_checkpoint));
        a_prev_lock_checkpoint == b_prev_lock_checkpoint
    } else {
        // Check for previous epoch case using both orientations
        check(&a, &b) || check(&b, &a)
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
        let shift_count = min(
            max(max_slot - global_slot(b1) - 1, 0),
            SUB_WINDOWS_PER_WINDOW,
        );

        // Initialize projected window
        let mut projected_window = b1
            .sub_window_densities
            .iter()
            .map(|d| d.as_u32())
            .collect::<Vec<_>>();

        // Ring-shift
        let mut i = relative_sub_window(global_slot(b1));
        for _ in [0..=shift_count] {
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

fn relative_sub_window(global_slot: u32) -> u32 {
    (global_slot / SLOTS_PER_SUB_WINDOW) % SUB_WINDOWS_PER_WINDOW
}

fn global_slot(b: &MinaConsensusState) -> u32 {
    b.curr_global_slot.slot_number.as_u32()
}

fn long_range_fork_take(
    tip_cs: &MinaConsensusState,
    candidate_cs: &MinaConsensusState,
    tip_hash: &StateHash,
    candidate_hash: &StateHash,
) -> (bool, ConsensusLongRangeForkResolutionKind) {
    let tip_density = relative_min_window_density(tip_cs, candidate_cs);
    let candidate_density = relative_min_window_density(candidate_cs, tip_cs);
    use std::cmp::Ordering::*;
    use ConsensusLongRangeForkResolutionKind::*;
    let (take, why) = match candidate_density.cmp(&tip_density) {
        Greater => (true, SubWindowDensity),
        Less => (false, SubWindowDensity),
        Equal => {
            let tip_height = &tip_cs.blockchain_length;
            let candidate_height = &candidate_cs.blockchain_length;
            match candidate_height.cmp(&tip_height) {
                Greater => (true, ChainLength),
                Less => (false, ChainLength),
                Equal => {
                    let tip_vrf = tip_cs.last_vrf_output.blake2b();
                    let candidate_vrf = candidate_cs.last_vrf_output.blake2b();

                    match candidate_vrf.cmp(&tip_vrf) {
                        Greater => (true, Vrf),
                        Less => (false, Vrf),
                        Equal => {
                            if candidate_hash > tip_hash {
                                (true, StateHash)
                            } else {
                                (false, StateHash)
                            }
                        }
                    }
                }
            }
        }
    };
    (take, why)
}
