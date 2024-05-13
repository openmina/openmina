use mina_p2p_messages::v2::{self, StateHash};

use crate::constants::{slots_per_window, CONSTRAINT_CONSTANTS};

pub fn genesis_and_negative_one_protocol_states(
    constants: v2::MinaBaseProtocolConstantsCheckedValueStableV1,
    genesis_ledger_hash: v2::LedgerHash,
    genesis_total_currency: v2::CurrencyAmountStableV1,
    genesis_winner: v2::NonZeroCurvePoint,
    empty_pending_coinbase_hash: v2::PendingCoinbaseHash,
    empty_local_state: v2::MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1,
    empty_body_hash: v2::ConsensusBodyReferenceStableV1,
    genesis_vrf_output: v2::ConsensusVrfOutputTruncatedStableV1,
    genesis_epoch_seed: v2::EpochSeed,
) -> (
    v2::MinaStateProtocolStateValueStableV2,
    v2::MinaStateProtocolStateValueStableV2,
) {
    let negative_one = protocol_state(
        constants.clone(),
        genesis_ledger_hash.clone(),
        genesis_total_currency.clone(),
        genesis_winner.clone(),
        empty_pending_coinbase_hash.clone(),
        empty_local_state.clone(),
        empty_body_hash.clone(),
        true,
    );
    let negative_one_hash = negative_one.hash();
    let mut genesis = protocol_state(
        constants,
        genesis_ledger_hash,
        genesis_total_currency,
        genesis_winner,
        empty_pending_coinbase_hash,
        empty_local_state,
        empty_body_hash,
        false,
    );
    if CONSTRAINT_CONSTANTS.fork.is_none() {
        genesis.previous_state_hash = negative_one_hash.clone();
        genesis.body.genesis_state_hash = negative_one_hash.clone();
    }
    genesis.body.consensus_state.last_vrf_output = genesis_vrf_output;
    genesis.body.consensus_state.next_epoch_data =
        v2::ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1 {
            seed: genesis_epoch_seed,
            lock_checkpoint: negative_one_hash,
            epoch_length: 2.into(),
            ..genesis.body.consensus_state.next_epoch_data
        };

    (negative_one, genesis)
}

fn protocol_state(
    constants: v2::MinaBaseProtocolConstantsCheckedValueStableV1,
    genesis_ledger_hash: v2::LedgerHash,
    genesis_total_currency: v2::CurrencyAmountStableV1,
    genesis_winner: v2::NonZeroCurvePoint,
    empty_pending_coinbase_hash: v2::PendingCoinbaseHash,
    empty_local_state: v2::MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1,
    empty_body_hash: v2::ConsensusBodyReferenceStableV1,
    negative_one: bool,
) -> v2::MinaStateProtocolStateValueStableV2 {
    v2::MinaStateProtocolStateValueStableV2 {
        previous_state_hash: match CONSTRAINT_CONSTANTS.fork.as_ref() {
            None => StateHash::zero(),
            Some(_) if negative_one => StateHash::zero(),
            Some(fork) => StateHash::from_fp(fork.previous_state_hash),
        },
        body: v2::MinaStateProtocolStateBodyValueStableV2 {
            genesis_state_hash: StateHash::zero(),
            blockchain_state: blockchain_state(
                genesis_ledger_hash.clone(),
                constants.genesis_state_timestamp.clone(),
                empty_pending_coinbase_hash,
                empty_local_state,
                empty_body_hash,
            ),
            consensus_state: consensus_state(
                &constants,
                genesis_ledger_hash,
                genesis_total_currency,
                genesis_winner,
                negative_one,
            ),
            constants,
        },
    }
}

fn blockchain_state(
    genesis_ledger_hash: v2::LedgerHash,
    genesis_state_timestamp: v2::BlockTimeTimeStableV1,
    empty_pending_coinbase_hash: v2::PendingCoinbaseHash,
    empty_local_state: v2::MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1,
    empty_body_hash: v2::ConsensusBodyReferenceStableV1,
) -> v2::MinaStateBlockchainStateValueStableV2 {
    let stmt_registers = v2::MinaStateBlockchainStateValueStableV2LedgerProofStatementSource {
        first_pass_ledger: genesis_ledger_hash.clone(),
        second_pass_ledger: genesis_ledger_hash.clone(),
        pending_coinbase_stack: v2::MinaBasePendingCoinbaseStackVersionedStableV1::empty(),
        local_state: empty_local_state,
    };
    let empty_fee_excess = v2::TokenFeeExcess {
        token: v2::TokenIdKeyHash::default(),
        amount: v2::SignedAmount {
            magnitude: v2::CurrencyAmountStableV1(0u64.into()),
            sgn: v2::SgnStableV1::Pos,
        },
    };
    let ledger_proof_statement = v2::MinaStateBlockchainStateValueStableV2LedgerProofStatement {
        source: stmt_registers.clone(),
        target: stmt_registers.clone(),
        connecting_ledger_left: genesis_ledger_hash.clone(),
        connecting_ledger_right: genesis_ledger_hash.clone(),
        supply_increase: v2::MinaStateBlockchainStateValueStableV2SignedAmount {
            magnitude: v2::CurrencyAmountStableV1(0u64.into()),
            sgn: v2::SgnStableV1::Pos,
        },
        fee_excess: v2::MinaBaseFeeExcessStableV1(empty_fee_excess.clone(), empty_fee_excess),
        sok_digest: (),
    };

    v2::MinaStateBlockchainStateValueStableV2 {
        staged_ledger_hash: v2::MinaBaseStagedLedgerHashStableV1::zero(
            genesis_ledger_hash.clone(),
            empty_pending_coinbase_hash,
        ),
        genesis_ledger_hash,
        ledger_proof_statement,
        timestamp: genesis_state_timestamp,
        body_reference: empty_body_hash,
    }
}

fn consensus_state(
    constants: &v2::MinaBaseProtocolConstantsCheckedValueStableV1,
    genesis_ledger_hash: v2::LedgerHash,
    genesis_total_currency: v2::CurrencyAmountStableV1,
    genesis_winner: v2::NonZeroCurvePoint,
    negative_one: bool,
) -> v2::ConsensusProofOfStakeDataConsensusStateValueStableV2 {
    let is_genesis = if negative_one { 0 } else { 1 };
    let (blockchain_length, global_slot_since_genesis) = match CONSTRAINT_CONSTANTS.fork.as_ref() {
        None => (is_genesis, 0),
        Some(fork) => (fork.previous_length + is_genesis, fork.genesis_slot),
    };

    v2::ConsensusProofOfStakeDataConsensusStateValueStableV2 {
        blockchain_length: v2::UnsignedExtendedUInt32StableV1(blockchain_length.into()),
        epoch_count: v2::UnsignedExtendedUInt32StableV1::default(),
        min_window_density: slots_per_window(constants).into(),
        sub_window_densities: std::iter::once(is_genesis.into())
            .chain(
                (1..CONSTRAINT_CONSTANTS.sub_windows_per_window)
                    .map(|_| constants.slots_per_sub_window.clone()),
            )
            .collect(),
        last_vrf_output: v2::ConsensusVrfOutputTruncatedStableV1::zero(),
        total_currency: genesis_total_currency.clone(),
        curr_global_slot_since_hard_fork: v2::ConsensusGlobalSlotStableV1 {
            slot_number: v2::MinaNumbersGlobalSlotSinceHardForkMStableV1::SinceHardFork(
                v2::UnsignedExtendedUInt32StableV1::default(),
            ),
            slots_per_epoch: constants.slots_per_epoch.clone(),
        },
        global_slot_since_genesis: v2::MinaNumbersGlobalSlotSinceGenesisMStableV1::SinceGenesis(
            global_slot_since_genesis.into(),
        ),
        staking_epoch_data:
            v2::ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1::zero(
                genesis_ledger_hash.clone(),
                genesis_total_currency.clone(),
            ),
        next_epoch_data:
            v2::ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1::zero(
                genesis_ledger_hash,
                genesis_total_currency,
            ),
        has_ancestor_in_same_checkpoint_window: !negative_one,
        block_stake_winner: genesis_winner.clone(),
        block_creator: genesis_winner.clone(),
        coinbase_receiver: genesis_winner,
        supercharge_coinbase: true,
    }
}
