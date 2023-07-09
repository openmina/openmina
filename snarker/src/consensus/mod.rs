mod consensus_state;
pub use consensus_state::*;

mod consensus_actions;
pub use consensus_actions::*;

mod consensus_reducer;
pub use consensus_reducer::*;

mod consensus_effects;
pub use consensus_effects::*;

use mina_p2p_messages::v2::{
    ConsensusProofOfStakeDataConsensusStateValueStableV2, MinaStateProtocolStateBodyValueStableV2,
};

// TODO(binier): do we need to verify constants? Probably they are verified
// using block proof verification, but check just to be sure.
pub fn is_short_range_fork(
    a: &MinaStateProtocolStateBodyValueStableV2,
    b: &MinaStateProtocolStateBodyValueStableV2,
) -> bool {
    let check = |s1: &ConsensusProofOfStakeDataConsensusStateValueStableV2,
                 s2: &ConsensusProofOfStakeDataConsensusStateValueStableV2| {
        let slots_per_epoch = s2.curr_global_slot.slots_per_epoch.as_u32();
        let s2_epoch_slot = s2.curr_global_slot.slot_number.as_u32() % slots_per_epoch;
        if s1.epoch_count.as_u32() == s2.epoch_count.as_u32() + 1
            && s2_epoch_slot >= slots_per_epoch * 2 / 3
        {
            // S1 is one epoch ahead of S2 and S2 is not in the seed update range
            s1.staking_epoch_data.lock_checkpoint == s2.next_epoch_data.lock_checkpoint
        } else {
            false
        }
    };

    if a.consensus_state.epoch_count == b.consensus_state.epoch_count {
        let a_prev_lock_checkpoint = &a.consensus_state.staking_epoch_data.lock_checkpoint;
        let b_prev_lock_checkpoint = &b.consensus_state.staking_epoch_data.lock_checkpoint;
        // Simple case: blocks have same previous epoch, so compare previous epochs' lock_checkpoints
        a_prev_lock_checkpoint == b_prev_lock_checkpoint
    } else {
        // Check for previous epoch case using both orientations
        check(&a.consensus_state, &b.consensus_state)
            || check(&b.consensus_state, &a.consensus_state)
    }
}
