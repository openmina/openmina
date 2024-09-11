use mina_hasher::Fp;
use mina_p2p_messages::v2::{
    ConsensusProofOfStakeDataConsensusStateValueStableV2,
    ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
    ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
    MinaBaseEpochLedgerValueStableV1, MinaBaseFeeExcessStableV1,
    MinaBasePendingCoinbaseStackVersionedStableV1, MinaBasePendingCoinbaseStateStackStableV1,
    MinaBaseProtocolConstantsCheckedValueStableV1, MinaBaseStagedLedgerHashStableV1,
    MinaStateBlockchainStateValueStableV2,
    MinaStateBlockchainStateValueStableV2LedgerProofStatement,
    MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
    MinaStateProtocolStateBodyValueStableV2, MinaStateProtocolStateValueStableV2,
    MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1, SgnStableV1, TokenFeeExcess,
};

use crate::{
    hash::{hash_with_kimchi, Inputs},
    proofs::block::{consensus::ConsensusState, ProtocolStateBody},
    scan_state::transaction_logic::protocol_state::{EpochData, EpochLedger},
    ToInputs,
};

pub trait MinaHash {
    fn hash(&self) -> Fp;
}

impl ToInputs for MinaStateProtocolStateBodyValueStableV2 {
    fn to_inputs(&self, inputs: &mut Inputs) {
        let MinaStateProtocolStateBodyValueStableV2 {
            genesis_state_hash,
            blockchain_state,
            consensus_state,
            constants,
        } = &self;

        // constants
        constants.to_inputs(inputs);

        // Genesis
        inputs.append_field(genesis_state_hash.to_field());

        // This is blockchain_state
        blockchain_state.to_inputs(inputs);

        // CONSENSUS
        {
            let ConsensusProofOfStakeDataConsensusStateValueStableV2 {
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
            } = consensus_state;

            inputs.append_u32(blockchain_length.as_u32());
            inputs.append_u32(epoch_count.as_u32());
            inputs.append_u32(min_window_density.as_u32());
            for window in sub_window_densities {
                inputs.append_u32(window.as_u32());
            }
            {
                let vrf: &[u8] = last_vrf_output.as_ref();
                inputs.append_bytes(&vrf[..31]);
                // Ignore the last 3 bits
                let last_byte = vrf[31];
                for bit in [1, 2, 4, 8, 16] {
                    inputs.append_bool(last_byte & bit != 0);
                }
            }
            inputs.append_u64(total_currency.as_u64());
            inputs.append_u32(curr_global_slot_since_hard_fork.slot_number.as_u32());
            inputs.append_u32(curr_global_slot_since_hard_fork.slots_per_epoch.as_u32());
            inputs.append_u32(global_slot_since_genesis.as_u32());
            inputs.append_bool(*has_ancestor_in_same_checkpoint_window);
            inputs.append_bool(*supercharge_coinbase);

            {
                let ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1 {
                    ledger,
                    seed,
                    start_checkpoint,
                    lock_checkpoint,
                    epoch_length,
                } = staking_epoch_data;

                inputs.append_field(seed.to_field());
                inputs.append_field(start_checkpoint.to_field());
                inputs.append_u32(epoch_length.as_u32());

                let MinaBaseEpochLedgerValueStableV1 {
                    hash,
                    total_currency,
                } = ledger;

                inputs.append_field(hash.to_field());
                inputs.append_u64(total_currency.as_u64());
                inputs.append_field(lock_checkpoint.to_field());
            }

            {
                let ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1 {
                    ledger,
                    seed,
                    start_checkpoint,
                    lock_checkpoint,
                    epoch_length,
                } = next_epoch_data;

                inputs.append_field(seed.to_field());
                inputs.append_field(start_checkpoint.to_field());
                inputs.append_u32(epoch_length.as_u32());

                let MinaBaseEpochLedgerValueStableV1 {
                    hash,
                    total_currency,
                } = ledger;

                inputs.append_field(hash.to_field());
                inputs.append_u64(total_currency.as_u64());
                inputs.append_field(lock_checkpoint.to_field());
            }

            inputs.append_field(block_stake_winner.x.to_field());
            inputs.append_bool(block_stake_winner.is_odd);
            inputs.append_field(block_creator.x.to_field());
            inputs.append_bool(block_creator.is_odd);
            inputs.append_field(coinbase_receiver.x.to_field());
            inputs.append_bool(coinbase_receiver.is_odd);
        }
    }
}

impl ToInputs for MinaStateBlockchainStateValueStableV2 {
    fn to_inputs(&self, inputs: &mut Inputs) {
        let Self {
            staged_ledger_hash,
            genesis_ledger_hash,
            ledger_proof_statement,
            timestamp,
            body_reference,
        } = self;

        // Self::blockchain_state.staged_ledger_hash
        {
            let MinaBaseStagedLedgerHashStableV1 {
                non_snark,
                pending_coinbase_hash,
            } = staged_ledger_hash;

            inputs.append_bytes(non_snark.sha256().as_ref());
            inputs.append_field(pending_coinbase_hash.to_field());
        }
        // Self::blockchain_state.genesis_ledger_hash
        inputs.append_field(genesis_ledger_hash.to_field());

        // Self::blockchain_state.ledger_proof_statement
        {
            let MinaStateBlockchainStateValueStableV2LedgerProofStatement {
                source,
                target,
                connecting_ledger_left,
                connecting_ledger_right,
                supply_increase,
                fee_excess,
                sok_digest,
            } = ledger_proof_statement;

            let _sok_digest: &() = sok_digest;

            {
                let MinaStateBlockchainStateValueStableV2LedgerProofStatementSource {
                    first_pass_ledger,
                    second_pass_ledger,
                    pending_coinbase_stack:
                        MinaBasePendingCoinbaseStackVersionedStableV1 {
                            data,
                            state: MinaBasePendingCoinbaseStateStackStableV1 { init, curr },
                        },
                    local_state:
                        MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1 {
                            stack_frame,
                            call_stack,
                            transaction_commitment,
                            full_transaction_commitment,
                            excess,
                            supply_increase,
                            ledger,
                            success,
                            account_update_index,
                            failure_status_tbl: _,
                            will_succeed,
                        },
                } = source;

                inputs.append_field(first_pass_ledger.to_field());
                inputs.append_field(second_pass_ledger.to_field());
                inputs.append_field(data.to_field());
                inputs.append_field(init.to_field());
                inputs.append_field(curr.to_field());

                inputs.append_field(stack_frame.to_field());
                inputs.append_field(call_stack.to_field());
                inputs.append_field(transaction_commitment.to_field());
                inputs.append_field(full_transaction_commitment.to_field());
                inputs.append_u64(excess.magnitude.as_u64());
                inputs.append_bool(matches!(excess.sgn, SgnStableV1::Pos));
                inputs.append_u64(supply_increase.magnitude.as_u64());
                inputs.append_bool(matches!(supply_increase.sgn, SgnStableV1::Pos));
                inputs.append_field(ledger.to_field());
                inputs.append_u32(account_update_index.as_u32());
                inputs.append_bool(*success);
                inputs.append_bool(*will_succeed);
            }

            {
                let MinaStateBlockchainStateValueStableV2LedgerProofStatementSource {
                    first_pass_ledger,
                    second_pass_ledger,
                    pending_coinbase_stack:
                        MinaBasePendingCoinbaseStackVersionedStableV1 {
                            data,
                            state: MinaBasePendingCoinbaseStateStackStableV1 { init, curr },
                        },
                    local_state:
                        MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1 {
                            stack_frame,
                            call_stack,
                            transaction_commitment,
                            full_transaction_commitment,
                            excess,
                            supply_increase,
                            ledger,
                            success,
                            account_update_index,
                            failure_status_tbl: _,
                            will_succeed,
                        },
                } = target;

                inputs.append_field(first_pass_ledger.to_field());
                inputs.append_field(second_pass_ledger.to_field());
                inputs.append_field(data.to_field());
                inputs.append_field(init.to_field());
                inputs.append_field(curr.to_field());

                inputs.append_field(stack_frame.to_field());
                inputs.append_field(call_stack.to_field());
                inputs.append_field(transaction_commitment.to_field());
                inputs.append_field(full_transaction_commitment.to_field());
                inputs.append_u64(excess.magnitude.as_u64());
                inputs.append_bool(matches!(excess.sgn, SgnStableV1::Pos));
                inputs.append_u64(supply_increase.magnitude.as_u64());
                inputs.append_bool(matches!(supply_increase.sgn, SgnStableV1::Pos));
                inputs.append_field(ledger.to_field());
                inputs.append_u32(account_update_index.as_u32());
                inputs.append_bool(*success);
                inputs.append_bool(*will_succeed);
            }

            inputs.append_field(connecting_ledger_left.to_field());
            inputs.append_field(connecting_ledger_right.to_field());

            inputs.append_u64(supply_increase.magnitude.as_u64());
            inputs.append_bool(matches!(supply_increase.sgn, SgnStableV1::Pos));

            let MinaBaseFeeExcessStableV1(
                TokenFeeExcess {
                    token: fee_token_l,
                    amount: fee_excess_l,
                },
                TokenFeeExcess {
                    token: fee_token_r,
                    amount: fee_excess_r,
                },
            ) = fee_excess;

            inputs.append_field(fee_token_l.to_field());
            inputs.append_u64(fee_excess_l.magnitude.as_u64());
            inputs.append_bool(matches!(fee_excess_l.sgn, SgnStableV1::Pos));

            inputs.append_field(fee_token_r.to_field());
            inputs.append_u64(fee_excess_r.magnitude.as_u64());
            inputs.append_bool(matches!(fee_excess_r.sgn, SgnStableV1::Pos));
        }

        inputs.append_u64(timestamp.as_u64());
        inputs.append_bytes(body_reference.as_ref());
    }
}

impl ToInputs for MinaBaseProtocolConstantsCheckedValueStableV1 {
    fn to_inputs(&self, inputs: &mut Inputs) {
        let Self {
            k,
            slots_per_epoch,
            slots_per_sub_window,
            delta,
            grace_period_slots,
            genesis_state_timestamp,
        } = self;

        inputs.append_u32(k.as_u32());
        inputs.append_u32(delta.as_u32());
        inputs.append_u32(slots_per_epoch.as_u32());
        inputs.append_u32(slots_per_sub_window.as_u32());
        inputs.append_u32(grace_period_slots.as_u32());
        inputs.append_u64(genesis_state_timestamp.as_u64());
    }
}

impl ToInputs for ProtocolStateBody {
    fn to_inputs(&self, inputs: &mut Inputs) {
        let Self {
            genesis_state_hash,
            blockchain_state,
            consensus_state,
            constants,
        } = &self;

        // constants
        constants.to_inputs(inputs);

        // Genesis
        inputs.append_field(*genesis_state_hash);

        // This is blockchain_state
        blockchain_state.to_inputs(inputs);

        // CONSENSUS
        {
            let ConsensusState {
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
            } = consensus_state;

            inputs.append(blockchain_length);
            inputs.append(epoch_count);
            inputs.append(min_window_density);

            for window in sub_window_densities {
                inputs.append(window);
            }
            for b in last_vrf_output.iter() {
                inputs.append_bool(*b);
            }

            inputs.append(total_currency);
            inputs.append(&curr_global_slot_since_hard_fork.slot_number);
            inputs.append(&curr_global_slot_since_hard_fork.slots_per_epoch);
            inputs.append(global_slot_since_genesis);
            inputs.append_bool(has_ancestor_in_same_checkpoint_window.as_bool());
            inputs.append_bool(supercharge_coinbase.as_bool());

            let mut add_epoch_data = |epoch_data: &EpochData<Fp>| {
                let EpochData {
                    ledger,
                    seed,
                    start_checkpoint,
                    lock_checkpoint,
                    epoch_length,
                } = epoch_data;

                inputs.append(seed);
                inputs.append(start_checkpoint);
                inputs.append(epoch_length);

                let EpochLedger {
                    hash,
                    total_currency,
                } = ledger;

                inputs.append(hash);
                inputs.append(total_currency);
                inputs.append(lock_checkpoint);
            };

            add_epoch_data(staking_epoch_data);
            add_epoch_data(next_epoch_data);

            inputs.append(block_stake_winner);
            inputs.append(block_creator);
            inputs.append(coinbase_receiver);
        }
    }
}

// REVIEW(dw): should be top-level constant str?
impl MinaHash for MinaStateProtocolStateBodyValueStableV2 {
    fn hash(&self) -> Fp {
        self.hash_with_param("MinaProtoStateBody")
    }
}

// REVIEW(ok): test vectors/regression tests?
pub fn hashes_abstract(previous_state_hash: Fp, body_hash: Fp) -> Fp {
    let mut inputs = Inputs::new();

    inputs.append_field(previous_state_hash);
    inputs.append_field(body_hash);

    // REVIEW(dw): should be top-level constant str?
    hash_with_kimchi("MinaProtoState", &inputs.to_fields())
}

impl MinaHash for MinaStateProtocolStateValueStableV2 {
    fn hash(&self) -> Fp {
        let previous_state_hash = self.previous_state_hash.to_field();
        let body_hash = MinaHash::hash(&self.body);

        hashes_abstract(previous_state_hash, body_hash)
    }
}
