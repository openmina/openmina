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
use mina_p2p_messages::{list::List, v2};
use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};
use vrf::output::VrfOutput;

use self::vrf_evaluator::VrfWonSlotWithHash;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct BlockProducerWonSlot {
    pub slot_time: redux::Timestamp,
    pub delegator: (v2::NonZeroCurvePoint, AccountIndex),
    pub global_slot: v2::ConsensusGlobalSlotStableV1,
    pub vrf_output: Box<VrfOutput>,
    pub value_with_threshold: Option<(f64, f64)>,
    // Staking ledger which was used during vrf evaluation.
    pub staking_ledger_hash: v2::LedgerHash,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockWithoutProof {
    pub protocol_state: v2::MinaStateProtocolStateValueStableV2,
    pub delta_block_chain_proof: (v2::StateHash, List<v2::StateBodyHash>),
    pub current_protocol_version: v2::ProtocolVersionStableV2,
    pub proposed_protocol_version_opt: Option<v2::ProtocolVersionStableV2>,
    pub body: v2::StagedLedgerDiffBodyStableV1,
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

        let delegator = (
            won_slot.winner_account.clone().into(),
            won_slot.account_index,
        );
        let global_slot = v2::ConsensusGlobalSlotStableV1 {
            slot_number: v2::MinaNumbersGlobalSlotSinceHardForkMStableV1::SinceHardFork(
                won_slot.global_slot.into(),
            ),
            slots_per_epoch: 7140.into(), // TODO
        };

        Self {
            slot_time,
            delegator,
            global_slot,
            vrf_output: won_slot.vrf_output.clone(),
            value_with_threshold: won_slot.value_with_threshold,
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

    pub fn epoch(&self) -> u32 {
        self.global_slot() / self.global_slot.slots_per_epoch.as_u32()
    }

    pub fn global_slot_since_genesis(
        &self,
        slot_diff: u32,
    ) -> v2::MinaNumbersGlobalSlotSinceGenesisMStableV1 {
        let slot = self.global_slot() + slot_diff;
        v2::MinaNumbersGlobalSlotSinceGenesisMStableV1::SinceGenesis(slot.into())
    }

    pub fn timestamp(&self) -> v2::BlockTimeTimeStableV1 {
        let ms = u64::from(self.slot_time) / 1_000_000;
        v2::BlockTimeTimeStableV1(v2::UnsignedExtendedUInt64Int64ForVersionTagsStableV1(
            ms.into(),
        ))
    }

    pub fn next_slot_time(&self) -> redux::Timestamp {
        self.slot_time + 3 * 60 * 1_000_000_000_u64
    }
}

impl PartialOrd for BlockProducerWonSlot {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.global_slot().cmp(&other.global_slot()).then_with(|| {
            v2::ConsensusVrfOutputTruncatedStableV1::from(&*self.vrf_output)
                .blake2b()
                .cmp(&v2::ConsensusVrfOutputTruncatedStableV1::from(&*other.vrf_output).blake2b())
        }))
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
            v2::ConsensusVrfOutputTruncatedStableV1::from(&*self.vrf_output)
                .blake2b()
                .cmp(
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

pub fn to_epoch_and_slot(global_slot: &v2::ConsensusGlobalSlotStableV1) -> (u32, u32) {
    let epoch = global_slot.slot_number.as_u32() / global_slot.slots_per_epoch.as_u32();
    let slot = global_slot.slot_number.as_u32() % global_slot.slots_per_epoch.as_u32();
    (epoch, slot)
}

pub fn next_epoch_first_slot(global_slot: &v2::ConsensusGlobalSlotStableV1) -> u32 {
    let (epoch, _) = to_epoch_and_slot(global_slot);
    (epoch + 1) * global_slot.slots_per_epoch.as_u32()
}

// Returns the epoch number and whether it is the last slot of the epoch
// pub fn epoch_with_bounds(global_slot: u32) -> (u32, bool) {
//     // let epoch_bound = |global_slot| -> (u32, bool) {
//     //     (global_slot / SLOTS_PER_EPOCH, (global_slot + 1) % SLOTS_PER_EPOCH == 0)
//     // };

// }

impl BlockWithoutProof {
    pub fn with_hash_and_proof(
        self,
        hash: v2::StateHash,
        proof: v2::MinaBaseProofStableV2,
    ) -> ArcBlockWithHash {
        let block = v2::MinaBlockBlockStableV2 {
            header: v2::MinaBlockHeaderStableV2 {
                protocol_state: self.protocol_state,
                protocol_state_proof: proof,
                delta_block_chain_proof: self.delta_block_chain_proof,
                current_protocol_version: self.current_protocol_version,
                proposed_protocol_version_opt: self.proposed_protocol_version_opt,
            },
            body: self.body,
        };

        ArcBlockWithHash {
            block: block.into(),
            hash,
        }
    }
}

pub fn calc_epoch_seed(
    prev_epoch_seed: &v2::EpochSeed,
    vrf_hash: mina_hasher::Fp,
) -> v2::EpochSeed {
    // TODO(adonagy): fix this unwrap
    let old_seed = prev_epoch_seed.to_field().unwrap();
    let new_seed = ledger::hash_with_kimchi("MinaEpochSeed", &[old_seed, vrf_hash]);
    v2::MinaBaseEpochSeedStableV1(new_seed.into()).into()
}
