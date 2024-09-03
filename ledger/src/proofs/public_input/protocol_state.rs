use mina_hasher::Fp;
use mina_p2p_messages::v2::MinaBaseProtocolConstantsCheckedValueStableV1;

use crate::{
    hash::Inputs,
    proofs::block::{
        consensus::{CheckedConsensusState, ConsensusState},
        ProtocolStateBody,
    },
    scan_state::transaction_logic::protocol_state::{EpochData, EpochLedger},
    ToInputs,
};

impl ToInputs for crate::proofs::block::BlockchainState {
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
            let crate::staged_ledger::hash::StagedLedgerHash {
                non_snark,
                pending_coinbase_hash,
            } = staged_ledger_hash;

            inputs.append_bytes(non_snark.digest().as_ref());
            inputs.append_field(*pending_coinbase_hash);
        }
        // Self::blockchain_state.genesis_ledger_hash
        inputs.append_field(*genesis_ledger_hash);

        // Self::blockchain_state.ledger_proof_statement
        {
            let crate::scan_state::scan_state::transaction_snark::Statement {
                source,
                target,
                connecting_ledger_left,
                connecting_ledger_right,
                supply_increase,
                fee_excess,
                sok_digest,
            } = ledger_proof_statement;

            let _sok_digest: &() = sok_digest;

            inputs.append(source);
            inputs.append(target);

            inputs.append_field(*connecting_ledger_left);
            inputs.append_field(*connecting_ledger_right);

            inputs.append_u64(supply_increase.magnitude.as_u64());
            inputs.append_bool(supply_increase.sgn.is_pos());

            let crate::scan_state::fee_excess::FeeExcess {
                fee_token_l,
                fee_excess_l,
                fee_token_r,
                fee_excess_r,
            } = fee_excess;

            inputs.append(fee_token_l);
            inputs.append_u64(fee_excess_l.magnitude.as_u64());
            inputs.append_bool(fee_excess_l.sgn.is_pos());

            inputs.append(fee_token_r);
            inputs.append_u64(fee_excess_r.magnitude.as_u64());
            inputs.append_bool(fee_excess_r.sgn.is_pos());
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

impl ToInputs for ConsensusState {
    fn to_inputs(&self, inputs: &mut Inputs) {
        let crate::proofs::block::consensus::ConsensusState {
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
        } = self;

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
        inputs.append_bool(*has_ancestor_in_same_checkpoint_window);
        inputs.append_bool(*supercharge_coinbase);

        inputs.append(staking_epoch_data);
        inputs.append(next_epoch_data);

        inputs.append(block_stake_winner);
        inputs.append(block_creator);
        inputs.append(coinbase_receiver);
    }
}

impl ToInputs for CheckedConsensusState {
    fn to_inputs(&self, inputs: &mut Inputs) {
        let crate::proofs::block::consensus::CheckedConsensusState {
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
        } = self;

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

        inputs.append(staking_epoch_data);
        inputs.append(next_epoch_data);

        inputs.append(block_stake_winner);
        inputs.append(block_creator);
        inputs.append(coinbase_receiver);
    }
}

impl ToInputs for EpochData<Fp> {
    fn to_inputs(&self, inputs: &mut Inputs) {
        let EpochData {
            ledger,
            seed,
            start_checkpoint,
            lock_checkpoint,
            epoch_length,
        } = self;

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
    }
}

impl<CS: ToInputs> ToInputs for ProtocolStateBody<CS> {
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
        consensus_state.to_inputs(inputs);
    }
}
