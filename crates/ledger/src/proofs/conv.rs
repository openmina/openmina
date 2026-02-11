use mina_curves::pasta::Fp;
use mina_p2p_messages::v2;

use crate::scan_state::{
    currency::{self, Length, Slot},
    transaction_logic::protocol_state::{EpochData, EpochLedger},
};

use super::{
    block::{
        consensus::{CheckedConsensusState, ConsensusState},
        ProtocolState, ProtocolStateBody,
    },
    numbers::{
        currency::{CheckedAmount, CheckedCurrency},
        nat::{CheckedLength, CheckedNat, CheckedSlot},
    },
};

fn pubkey_to_nzcurvepoint(pk: &mina_signer::CompressedPubKey) -> v2::NonZeroCurvePoint {
    let v1: v2::NonZeroCurvePointUncompressedStableV1 = pk.clone().into();
    v1.into()
}

fn epoch_data_from_p2p(
    ledger: &v2::MinaBaseEpochLedgerValueStableV1,
    seed: &v2::EpochSeed,
    start_checkpoint: &v2::StateHash,
    lock_checkpoint: &v2::StateHash,
    epoch_length: &v2::UnsignedExtendedUInt32StableV1,
) -> EpochData<Fp> {
    EpochData {
        ledger: EpochLedger {
            hash: ledger.hash.to_field::<Fp>(),
            total_currency: currency::Amount::from_u64(ledger.total_currency.as_u64()),
        },
        seed: seed.to_field::<Fp>(),
        start_checkpoint: start_checkpoint.to_field::<Fp>(),
        lock_checkpoint: lock_checkpoint.to_field::<Fp>(),
        epoch_length: Length::from_u32(epoch_length.as_u32()),
    }
}

fn state_hash(value: Fp) -> v2::StateHash {
    v2::DataHashLibStateHashStableV1(value.into()).into()
}

fn ledger_hash(value: Fp) -> v2::LedgerHash {
    v2::MinaBaseLedgerHash0StableV1(value.into()).into()
}

fn epoch_ledger_seed(value: Fp) -> v2::EpochSeed {
    v2::MinaBaseEpochSeedStableV1(value.into()).into()
}

impl From<CheckedLength<Fp>> for v2::UnsignedExtendedUInt32StableV1 {
    fn from(value: CheckedLength<Fp>) -> Self {
        Self(value.to_inner().as_u32().into())
    }
}

impl From<CheckedSlot<Fp>> for v2::UnsignedExtendedUInt32StableV1 {
    fn from(value: CheckedSlot<Fp>) -> Self {
        Self(value.to_inner().as_u32().into())
    }
}

impl From<CheckedAmount<Fp>> for v2::UnsignedExtendedUInt64Int64ForVersionTagsStableV1 {
    fn from(value: CheckedAmount<Fp>) -> Self {
        Self(value.to_inner().as_u64().into())
    }
}

impl From<CheckedAmount<Fp>> for v2::CurrencyAmountStableV1 {
    fn from(value: CheckedAmount<Fp>) -> Self {
        Self(value.into())
    }
}

impl From<&EpochLedger<Fp>> for v2::MinaBaseEpochLedgerValueStableV1 {
    fn from(value: &EpochLedger<Fp>) -> Self {
        Self {
            hash: ledger_hash(value.hash),
            total_currency: value.total_currency.into(),
        }
    }
}

impl From<&EpochData<Fp>>
    for v2::ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1
{
    fn from(value: &EpochData<Fp>) -> Self {
        Self {
            ledger: (&value.ledger).into(),
            seed: epoch_ledger_seed(value.seed),
            start_checkpoint: state_hash(value.start_checkpoint),
            lock_checkpoint: state_hash(value.lock_checkpoint),
            epoch_length: (&value.epoch_length).into(),
        }
    }
}

impl From<&EpochData<Fp>>
    for v2::ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1
{
    fn from(value: &EpochData<Fp>) -> Self {
        Self {
            ledger: (&value.ledger).into(),
            seed: epoch_ledger_seed(value.seed),
            start_checkpoint: state_hash(value.start_checkpoint),
            lock_checkpoint: state_hash(value.lock_checkpoint),
            epoch_length: (&value.epoch_length).into(),
        }
    }
}

impl From<&ProtocolState> for v2::MinaStateProtocolStateValueStableV2 {
    fn from(value: &ProtocolState) -> Self {
        Self {
            previous_state_hash: state_hash(value.previous_state_hash),
            body: (&value.body).into(),
        }
    }
}

impl From<&v2::MinaStateProtocolStateValueStableV2> for ProtocolState {
    fn from(value: &v2::MinaStateProtocolStateValueStableV2) -> Self {
        let v2::MinaStateProtocolStateValueStableV2 {
            previous_state_hash,
            body,
        } = value;

        Self {
            previous_state_hash: previous_state_hash.to_field::<Fp>(),
            body: body.into(),
        }
    }
}

impl From<&super::block::BlockchainState> for v2::MinaStateBlockchainStateValueStableV2 {
    fn from(value: &super::block::BlockchainState) -> Self {
        let super::block::BlockchainState {
            staged_ledger_hash,
            genesis_ledger_hash,
            ledger_proof_statement,
            timestamp,
            body_reference,
        } = value;

        Self {
            staged_ledger_hash: staged_ledger_hash.into(),
            genesis_ledger_hash: (*genesis_ledger_hash).into(),
            ledger_proof_statement: ledger_proof_statement.into(),
            timestamp: timestamp.into(),
            body_reference: body_reference.clone(),
        }
    }
}

impl From<&ProtocolStateBody<ConsensusState>> for v2::MinaStateProtocolStateBodyValueStableV2 {
    fn from(value: &ProtocolStateBody) -> Self {
        Self {
            genesis_state_hash: state_hash(value.genesis_state_hash),
            blockchain_state: (&value.blockchain_state).into(),
            consensus_state: (&value.consensus_state).into(),
            constants: value.constants,
        }
    }
}

impl From<&v2::MinaStateProtocolStateBodyValueStableV2> for ProtocolStateBody<ConsensusState> {
    fn from(value: &v2::MinaStateProtocolStateBodyValueStableV2) -> Self {
        let v2::MinaStateProtocolStateBodyValueStableV2 {
            genesis_state_hash,
            blockchain_state,
            consensus_state,
            constants,
        } = value;

        Self {
            genesis_state_hash: genesis_state_hash.to_field::<Fp>(),
            blockchain_state: blockchain_state.into(),
            consensus_state: consensus_state.into(),
            constants: *constants,
        }
    }
}

impl From<&v2::ConsensusProofOfStakeDataConsensusStateValueStableV2> for ConsensusState {
    fn from(value: &v2::ConsensusProofOfStakeDataConsensusStateValueStableV2) -> Self {
        let v2::ConsensusProofOfStakeDataConsensusStateValueStableV2 {
            blockchain_length,
            epoch_count,
            min_window_density,
            sub_window_densities,
            last_vrf_output,
            total_currency,
            curr_global_slot_since_hard_fork,
            global_slot_since_genesis,
            staking_epoch_data,
            next_epoch_data,
            has_ancestor_in_same_checkpoint_window,
            block_stake_winner,
            block_creator,
            coinbase_receiver,
            supercharge_coinbase,
        } = value;

        Self {
            blockchain_length: Length::from_u32(blockchain_length.as_u32()),
            epoch_count: Length::from_u32(epoch_count.as_u32()),
            min_window_density: Length::from_u32(min_window_density.as_u32()),
            sub_window_densities: sub_window_densities
                .iter()
                .map(|sub| Length::from_u32(sub.as_u32()))
                .collect(),
            last_vrf_output: {
                let mut output = Vec::with_capacity(253);
                output.extend(last_vrf_output.iter().take(31).flat_map(|b| {
                    [1u8, 2, 4, 8, 16, 32, 64, 128]
                        .iter()
                        .map(|bit| *bit & *b != 0)
                }));
                // Ignore last 3 bits
                let last_byte = last_vrf_output[31];
                output.extend([1, 2, 4, 8, 16].iter().map(|bit| *bit & last_byte != 0));
                output.try_into().unwrap() // Vec of exactly 253 elements
            },
            curr_global_slot_since_hard_fork: curr_global_slot_since_hard_fork.into(),
            global_slot_since_genesis: Slot::from_u32(global_slot_since_genesis.as_u32()),
            total_currency: currency::Amount::from_u64(total_currency.as_u64()),
            staking_epoch_data: epoch_data_from_p2p(
                &staking_epoch_data.ledger,
                &staking_epoch_data.seed,
                &staking_epoch_data.start_checkpoint,
                &staking_epoch_data.lock_checkpoint,
                &staking_epoch_data.epoch_length,
            ),
            next_epoch_data: epoch_data_from_p2p(
                &next_epoch_data.ledger,
                &next_epoch_data.seed,
                &next_epoch_data.start_checkpoint,
                &next_epoch_data.lock_checkpoint,
                &next_epoch_data.epoch_length,
            ),
            has_ancestor_in_same_checkpoint_window: *has_ancestor_in_same_checkpoint_window,
            block_stake_winner: block_stake_winner.into(),
            block_creator: block_creator.into(),
            coinbase_receiver: coinbase_receiver.into(),
            supercharge_coinbase: *supercharge_coinbase,
        }
    }
}

impl From<&CheckedConsensusState> for v2::ConsensusProofOfStakeDataConsensusStateValueStableV2 {
    fn from(value: &CheckedConsensusState) -> Self {
        Self {
            blockchain_length: value.blockchain_length.into(),
            epoch_count: value.epoch_count.into(),
            min_window_density: value.min_window_density.into(),
            sub_window_densities: value
                .sub_window_densities
                .iter()
                .map(|v| (*v).into())
                .collect(),
            // can't restore it.
            last_vrf_output: v2::ConsensusVrfOutputTruncatedStableV1(Vec::new().into()),
            total_currency: value.total_currency.into(),
            curr_global_slot_since_hard_fork: v2::ConsensusGlobalSlotStableV1 {
                slot_number: v2::MinaNumbersGlobalSlotSinceHardForkMStableV1::SinceHardFork(
                    value.curr_global_slot_since_hard_fork.slot_number.into(),
                ),
                slots_per_epoch: value
                    .curr_global_slot_since_hard_fork
                    .slots_per_epoch
                    .into(),
            },
            global_slot_since_genesis: v2::MinaNumbersGlobalSlotSinceGenesisMStableV1::SinceGenesis(
                v2::UnsignedExtendedUInt32StableV1(
                    value.global_slot_since_genesis.to_inner().as_u32().into(),
                ),
            ),
            staking_epoch_data: (&value.staking_epoch_data).into(),
            next_epoch_data: (&value.next_epoch_data).into(),
            has_ancestor_in_same_checkpoint_window: value
                .has_ancestor_in_same_checkpoint_window
                .as_bool(),
            block_stake_winner: pubkey_to_nzcurvepoint(&value.block_stake_winner),
            block_creator: pubkey_to_nzcurvepoint(&value.block_creator),
            coinbase_receiver: pubkey_to_nzcurvepoint(&value.coinbase_receiver),
            supercharge_coinbase: value.supercharge_coinbase.as_bool(),
        }
    }
}

impl From<&ConsensusState> for v2::ConsensusProofOfStakeDataConsensusStateValueStableV2 {
    fn from(value: &ConsensusState) -> Self {
        Self {
            blockchain_length: value.blockchain_length.as_u32().into(),
            epoch_count: value.epoch_count.as_u32().into(),
            min_window_density: value.min_window_density.as_u32().into(),
            sub_window_densities: value
                .sub_window_densities
                .iter()
                .map(|v| v.as_u32().into())
                .collect(),
            // can't restore it.
            last_vrf_output: v2::ConsensusVrfOutputTruncatedStableV1(Vec::new().into()),
            total_currency: value.total_currency.into(),
            curr_global_slot_since_hard_fork: v2::ConsensusGlobalSlotStableV1 {
                slot_number: v2::MinaNumbersGlobalSlotSinceHardForkMStableV1::SinceHardFork(
                    value.curr_global_slot_since_hard_fork.slot_number.into(),
                ),
                slots_per_epoch: value
                    .curr_global_slot_since_hard_fork
                    .slots_per_epoch
                    .into(),
            },
            global_slot_since_genesis: v2::MinaNumbersGlobalSlotSinceGenesisMStableV1::SinceGenesis(
                v2::UnsignedExtendedUInt32StableV1(value.global_slot_since_genesis.as_u32().into()),
            ),
            staking_epoch_data: (&value.staking_epoch_data).into(),
            next_epoch_data: (&value.next_epoch_data).into(),
            has_ancestor_in_same_checkpoint_window: value.has_ancestor_in_same_checkpoint_window,
            block_stake_winner: pubkey_to_nzcurvepoint(&value.block_stake_winner),
            block_creator: pubkey_to_nzcurvepoint(&value.block_creator),
            coinbase_receiver: pubkey_to_nzcurvepoint(&value.coinbase_receiver),
            supercharge_coinbase: value.supercharge_coinbase,
        }
    }
}
