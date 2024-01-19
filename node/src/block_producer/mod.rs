pub mod vrf_evaluator;

mod block_producer_config;
pub use block_producer_config::*;

mod block_producer_state;
pub use block_producer_state::*;

mod block_producer_event;
pub use block_producer_event::*;

mod block_producer_actions;
pub use block_producer_actions::*;

mod block_producer_reducer;

mod block_producer_effects;
pub use block_producer_effects::*;

mod block_producer_service;
pub use block_producer_service::*;

use ledger::AccountIndex;
use mina_p2p_messages::{
    bigint::BigInt,
    v2::{
        BlockTimeTimeStableV1, ConsensusGlobalSlotStableV1, ConsensusVrfOutputTruncatedStableV1,
        LedgerHash, MinaNumbersGlobalSlotSinceGenesisMStableV1,
        MinaNumbersGlobalSlotSinceHardForkMStableV1, NonZeroCurvePoint,
        UnsignedExtendedUInt64Int64ForVersionTagsStableV1,
    },
};
use mina_signer::CompressedPubKey;
use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};

use crate::account::AccountPublicKey;

use self::vrf_evaluator::VrfWonSlotWithHash;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct BlockProducerWonSlot {
    pub slot_time: redux::Timestamp,
    pub delegator: (NonZeroCurvePoint, AccountIndex),
    pub global_slot: ConsensusGlobalSlotStableV1,
    pub vrf_output: ConsensusVrfOutputTruncatedStableV1,
    // TODO(adonagy): maybe instead of passing it here, it can be
    // calculated on spot from `vrf_output`? Maybe with `vrf_output.blake2b()`?
    pub vrf_hash: BigInt,
    // Staking ledger which was used during vrf evaluation.
    pub staking_ledger_hash: LedgerHash,
}

impl BlockProducerWonSlot {
    pub fn from_vrf_won_slot(
        won_slot_with_hash: &VrfWonSlotWithHash,
        genesis_timestamp: redux::Timestamp,
    ) -> Self {
        let VrfWonSlotWithHash {
            won_slot,
            staking_ledger_hash,
        } = won_slot_with_hash;

        let slot_time = Self::calculate_slot_time(genesis_timestamp, won_slot.global_slot);

        let winner_pub_key = AccountPublicKey::from(
            CompressedPubKey::from_address(&won_slot.winner_account).unwrap(),
        );
        let delegator = (winner_pub_key.into(), won_slot.account_index.clone());
        let global_slot = ConsensusGlobalSlotStableV1 {
            slot_number: MinaNumbersGlobalSlotSinceHardForkMStableV1::SinceHardFork(
                won_slot.global_slot.into(),
            ),
            slots_per_epoch: 7140.into(), // TODO
        };

        Self {
            slot_time,
            delegator,
            global_slot,
            vrf_output: ConsensusVrfOutputTruncatedStableV1(
                (&won_slot.vrf_output_bytes[..]).into(),
            ),
            vrf_hash: won_slot.vrf_hash.clone(),
            staking_ledger_hash: staking_ledger_hash.clone(),
        }
    }

    fn calculate_slot_time(genesis_timestamp: redux::Timestamp, slot: u32) -> redux::Timestamp {
        // FIXME: this calculation must use values from the protocol constants,
        // now it assumes 3 minutes blocks.
        genesis_timestamp + (slot as u64) * 3 * 60 * 1_000_000_000_u64
    }

    pub fn global_slot(&self) -> u32 {
        self.global_slot.slot_number.as_u32()
    }

    pub fn global_slot_since_genesis(
        &self,
        slot_diff: u32,
    ) -> MinaNumbersGlobalSlotSinceGenesisMStableV1 {
        let slot = self.global_slot() + slot_diff;
        MinaNumbersGlobalSlotSinceGenesisMStableV1::SinceGenesis(slot.into())
    }

    pub fn timestamp(&self) -> BlockTimeTimeStableV1 {
        let ms = u64::from(self.slot_time) / 1_000_000;
        BlockTimeTimeStableV1(UnsignedExtendedUInt64Int64ForVersionTagsStableV1(ms.into()))
    }

    pub fn next_slot_time(&self) -> redux::Timestamp {
        self.slot_time + 3 * 60 * 1_000_000_000_u64
    }
}

impl PartialOrd for BlockProducerWonSlot {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            self.global_slot()
                .cmp(&other.global_slot())
                .then_with(|| self.vrf_output.blake2b().cmp(&other.vrf_output.blake2b())),
        )
    }
}

impl PartialEq<ArcBlockWithHash> for BlockProducerWonSlot {
    fn eq(&self, other: &ArcBlockWithHash) -> bool {
        self.partial_cmp(other).map_or(false, |ord| ord.is_eq())
    }
}

impl PartialOrd<ArcBlockWithHash> for BlockProducerWonSlot {
    fn partial_cmp(&self, other: &ArcBlockWithHash) -> Option<std::cmp::Ordering> {
        // TODO(binier): this assumes short range fork
        Some(self.global_slot().cmp(&other.global_slot()).then_with(|| {
            self.vrf_output.blake2b().cmp(
                &other
                    .header()
                    .protocol_state
                    .body
                    .consensus_state
                    .last_vrf_output
                    .blake2b(),
            )
        }))
    }
}

pub fn to_epoch_and_slot(global_slot: &ConsensusGlobalSlotStableV1) -> (u32, u32) {
    let epoch = global_slot.slot_number.as_u32() / global_slot.slots_per_epoch.as_u32();
    let slot = global_slot.slot_number.as_u32() % global_slot.slots_per_epoch.as_u32();
    (epoch, slot)
}

pub fn next_epoch_first_slot(global_slot: &ConsensusGlobalSlotStableV1) -> u32 {
    let (epoch, slot) = to_epoch_and_slot(global_slot);
    (epoch + 1) * global_slot.slots_per_epoch.as_u32()
}

// Returns the epoch number and whether it is the last slot of the epoch
// pub fn epoch_with_bounds(global_slot: u32) -> (u32, bool) {
//     // let epoch_bound = |global_slot| -> (u32, bool) {
//     //     (global_slot / SLOTS_PER_EPOCH, (global_slot + 1) % SLOTS_PER_EPOCH == 0)
//     // };

// }
