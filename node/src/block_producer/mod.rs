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
pub use block_producer_reducer::*;

mod block_producer_effects;
pub use block_producer_effects::*;

mod block_producer_service;
pub use block_producer_service::*;

use ledger::AccountIndex;
use mina_p2p_messages::{
    bigint::BigInt,
    v2::{
        BlockTimeTimeStableV1, ConsensusGlobalSlotStableV1, ConsensusVrfOutputTruncatedStableV1,
        LedgerHash, MinaNumbersGlobalSlotSinceGenesisMStableV1, NonZeroCurvePoint,
        UnsignedExtendedUInt64Int64ForVersionTagsStableV1, MinaNumbersGlobalSlotSinceHardForkMStableV1,
    },
};
use mina_signer::CompressedPubKey;
use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};
use vrf::VrfWonSlot;

use crate::account::AccountPublicKey;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct BlockProducerWonSlot {
    pub slot_time: redux::Timestamp,
    pub delegator: (NonZeroCurvePoint, AccountIndex),
    pub global_slot: ConsensusGlobalSlotStableV1,
    pub global_slot_since_genesis: MinaNumbersGlobalSlotSinceGenesisMStableV1,
    pub vrf_output: ConsensusVrfOutputTruncatedStableV1,
    // TODO(adonagy): maybe instead of passing it here, it can be
    // calculated on spot from `vrf_output`? Maybe with `vrf_output.blake2b()`?
    pub vrf_hash: BigInt,
    // Staking ledger which was used during vrf evaluation.
    // pub staking_ledger_hash: LedgerHash,
}

impl BlockProducerWonSlot {
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
            self.global_slot_since_genesis
                .as_u32()
                .cmp(&other.global_slot_since_genesis.as_u32())
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
        Some(
            self.global_slot_since_genesis
                .as_u32()
                .cmp(&other.global_slot())
                .then_with(|| {
                    self.vrf_output.blake2b().cmp(
                        &other
                            .header()
                            .protocol_state
                            .body
                            .consensus_state
                            .last_vrf_output
                            .blake2b(),
                    )
                }),
        )
    }
}

impl From<VrfWonSlot> for BlockProducerWonSlot {
    fn from(value: VrfWonSlot) -> Self {
        // TODO
        let slot_time = redux::Timestamp::global_now();

        let winner_pub_key = AccountPublicKey::from(CompressedPubKey::from_address(&value.winner_account).unwrap());
        let delegator = (winner_pub_key.into(), value.account_index.clone());
        let global_slot = ConsensusGlobalSlotStableV1 {
            slot_number: MinaNumbersGlobalSlotSinceHardForkMStableV1::SinceHardFork(value.global_slot.into()),
            slots_per_epoch: 7140.into(), // TODO
        };
        let global_slot_since_genesis = MinaNumbersGlobalSlotSinceGenesisMStableV1::SinceGenesis(value.global_slot.into());

        let vrf_output = ConsensusVrfOutputTruncatedStableV1(value.vrf_output_bytes.into());
        let vrf_hash = value.vrf_hash;
        Self {
            slot_time,
            delegator,
            global_slot,
            global_slot_since_genesis,
            vrf_output,
            vrf_hash,
        }
    }
}