use mina_hasher::Fp;
use mina_p2p_messages::v2::{
    ConsensusProofOfStakeDataConsensusStateValueStableV1,
    ConsensusProofOfStakeDataEpochDataNextValueVersionedValueStableV1,
    ConsensusProofOfStakeDataEpochDataStakingValueVersionedValueStableV1,
    MinaBaseEpochLedgerValueStableV1, MinaBaseFeeExcessStableV1,
    MinaBasePendingCoinbaseStackVersionedStableV1, MinaBasePendingCoinbaseStateStackStableV1,
    MinaBaseProtocolConstantsCheckedValueStableV1, MinaBaseStagedLedgerHashStableV1,
    MinaStateBlockchainStateValueStableV2,
    MinaStateBlockchainStateValueStableV2LedgerProofStatement,
    MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
    MinaStateProtocolStateBodyValueStableV2, MinaStateProtocolStateValueStableV2,
    MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1, SgnStableV1,
};

use crate::hash::{hash_with_kimchi, Inputs};

pub trait MinaHash {
    fn hash(&self) -> Fp;
}

impl MinaHash for MinaStateProtocolStateBodyValueStableV2 {
    fn hash(&self) -> Fp {
        let mut inputs = Inputs::new();

        let MinaStateProtocolStateBodyValueStableV2 {
            genesis_state_hash,
            blockchain_state,
            consensus_state,
            constants,
        } = &self;

        // constants
        {
            let MinaBaseProtocolConstantsCheckedValueStableV1 {
                k,
                slots_per_epoch,
                slots_per_sub_window,
                delta,
                genesis_state_timestamp,
            } = constants;

            inputs.append_u32(k.as_u32());
            inputs.append_u32(delta.as_u32());
            inputs.append_u32(slots_per_epoch.as_u32());
            inputs.append_u32(slots_per_sub_window.as_u32());
            inputs.append_u64(genesis_state_timestamp.as_u64());
        }

        // Genesis
        {
            inputs.append_field(genesis_state_hash.to_field());
        }

        // This is blockchain_state
        {
            let MinaStateBlockchainStateValueStableV2 {
                staged_ledger_hash,
                genesis_ledger_hash,
                ledger_proof_statement,
                timestamp,
                body_reference,
            } = blockchain_state;

            // Self::blockchain_state.staged_ledger_hash
            {
                let MinaBaseStagedLedgerHashStableV1 {
                    non_snark,
                    pending_coinbase_hash,
                } = staged_ledger_hash;

                inputs.append_bytes(&non_snark.sha256());
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
                        pending_coinbase_stack,
                        local_state,
                    } = source;

                    inputs.append_field(first_pass_ledger.to_field());
                    inputs.append_field(second_pass_ledger.to_field());

                    let MinaBasePendingCoinbaseStackVersionedStableV1 { data, state } =
                        pending_coinbase_stack;

                    inputs.append_field(data.to_field());

                    let MinaBasePendingCoinbaseStateStackStableV1 { init, curr } = state;

                    inputs.append_field(init.to_field());
                    inputs.append_field(curr.to_field());

                    let MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1 {
                        stack_frame,
                        call_stack,
                        transaction_commitment,
                        full_transaction_commitment,
                        token_id,
                        excess,
                        supply_increase,
                        ledger,
                        success,
                        account_update_index,
                        failure_status_tbl: _,
                        will_succeed,
                    } = local_state;

                    inputs.append_field(stack_frame.to_field());
                    inputs.append_field(call_stack.to_field());
                    inputs.append_field(transaction_commitment.to_field());
                    inputs.append_field(full_transaction_commitment.to_field());
                    inputs.append_field(token_id.to_field());
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
                        pending_coinbase_stack,
                        local_state,
                    } = target;

                    inputs.append_field(first_pass_ledger.to_field());
                    inputs.append_field(second_pass_ledger.to_field());

                    let MinaBasePendingCoinbaseStackVersionedStableV1 { data, state } =
                        pending_coinbase_stack;

                    inputs.append_field(data.to_field());

                    let MinaBasePendingCoinbaseStateStackStableV1 { init, curr } = state;

                    inputs.append_field(init.to_field());
                    inputs.append_field(curr.to_field());

                    let MinaTransactionLogicZkappCommandLogicLocalStateValueStableV1 {
                        stack_frame,
                        call_stack,
                        transaction_commitment,
                        full_transaction_commitment,
                        token_id,
                        excess,
                        supply_increase,
                        ledger,
                        success,
                        account_update_index,
                        failure_status_tbl: _,
                        will_succeed,
                    } = local_state;

                    inputs.append_field(stack_frame.to_field());
                    inputs.append_field(call_stack.to_field());
                    inputs.append_field(transaction_commitment.to_field());
                    inputs.append_field(full_transaction_commitment.to_field());
                    inputs.append_field(token_id.to_field());
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

                let MinaBaseFeeExcessStableV1 {
                    fee_token_l,
                    fee_excess_l,
                    fee_token_r,
                    fee_excess_r,
                } = fee_excess;

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

        // CONSENSUS
        {
            let ConsensusProofOfStakeDataConsensusStateValueStableV1 {
                blockchain_length,
                epoch_count,
                min_window_density,
                sub_window_densities,
                last_vrf_output,
                total_currency,
                curr_global_slot,
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
            inputs.append_u32(curr_global_slot.slot_number.as_u32());
            inputs.append_u32(curr_global_slot.slots_per_epoch.as_u32());
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

        hash_with_kimchi("MinaProtoStateBody", &inputs.to_fields())
    }
}

pub fn hashes_abstract(previous_state_hash: Fp, body_hash: Fp) -> Fp {
    let mut inputs = Inputs::new();

    inputs.append_field(previous_state_hash);
    inputs.append_field(body_hash);

    hash_with_kimchi("MinaProtoState", &inputs.to_fields())
}

impl MinaHash for MinaStateProtocolStateValueStableV2 {
    fn hash(&self) -> Fp {
        let previous_state_hash = self.previous_state_hash.to_field();
        let body_hash = self.body.hash();

        hashes_abstract(previous_state_hash, body_hash)
    }
}
