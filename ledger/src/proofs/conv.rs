// REVIEW(dw): STATUS: Require checking mapping
// REVIEW(dw): I did not check if the conversions map correctly the Caml types.
// TODO.

// REVIEW(dw): ! Careful ! I would use from mina_curves instead. Seems to be an
// alias though, but ust cleaner to use directly from mina-curves
use mina_hasher::Fp;
use mina_p2p_messages::v2;

use crate::scan_state::transaction_logic::protocol_state::{EpochData, EpochLedger};

use super::{
    block::{consensus::ConsensusState, ProtocolState, ProtocolStateBody},
    numbers::{
        currency::{CheckedAmount, CheckedCurrency},
        nat::{CheckedLength, CheckedNat, CheckedSlot},
    },
};

// REVIEW(dw): Mainly converters. I did not check if it was mapping the correct
// values.
fn state_hash(value: Fp) -> v2::StateHash {
    v2::DataHashLibStateHashStableV1(value.into()).into()
}

// REVIEW(dw): Mainly converters. I did not check if it was mapping the correct
// values.
fn ledger_hash(value: Fp) -> v2::LedgerHash {
    v2::MinaBaseLedgerHash0StableV1(value.into()).into()
}

// REVIEW(dw): Mainly converters. I did not check if it was mapping the correct
// values.
fn epoch_ledger_seed(value: Fp) -> v2::EpochSeed {
    v2::MinaBaseEpochSeedStableV1(value.into()).into()
}

// REVIEW(dw): Mainly converters. I did not check if it was mapping the correct
// values.
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

impl From<&ProtocolStateBody> for v2::MinaStateProtocolStateBodyValueStableV2 {
    fn from(value: &ProtocolStateBody) -> Self {
        Self {
            genesis_state_hash: state_hash(value.genesis_state_hash),
            blockchain_state: value.blockchain_state.clone(),
            consensus_state: (&value.consensus_state).into(),
            constants: value.constants.clone(),
        }
    }
}

impl From<&ConsensusState> for v2::ConsensusProofOfStakeDataConsensusStateValueStableV2 {
    fn from(value: &ConsensusState) -> Self {
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
            block_stake_winner: (&value.block_stake_winner).into(),
            block_creator: (&value.block_creator).into(),
            coinbase_receiver: (&value.coinbase_receiver).into(),
            supercharge_coinbase: value.supercharge_coinbase.as_bool(),
        }
    }
}
