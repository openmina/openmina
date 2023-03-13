use mina_hasher::Fp;
use mina_p2p_messages::v2::{
    MinaStateProtocolStateBodyValueStableV2, MinaStateProtocolStateValueStableV2, SgnStableV1,
};

use crate::hash::{hash_with_kimchi, Inputs};

pub trait MinaHash {
    fn hash(&self) -> Fp;
}

impl MinaHash for MinaStateProtocolStateBodyValueStableV2 {
    fn hash(&self) -> Fp {
        let mut inputs = Inputs::new();

        // constants
        {
            inputs.append_u32(self.constants.k.0.as_u32());
            inputs.append_u32(self.constants.delta.0.as_u32());
            inputs.append_u32(self.constants.slots_per_epoch.0.as_u32());
            inputs.append_u32(self.constants.slots_per_sub_window.0.as_u32());
            inputs.append_u64(self.constants.genesis_state_timestamp.0 .0.as_u64());
        }

        // Genesis
        {
            inputs.append_field(self.genesis_state_hash.0.to_field());
        }

        // This is blockchain_state
        {
            // Self::blockchain_state.staged_ledger_hash
            {
                let staged = &self.blockchain_state.staged_ledger_hash;
                inputs.append_bytes(&staged.non_snark.sha256());
                inputs.append_field(staged.pending_coinbase_hash.0 .0.to_field());
            }
            // Self::blockchain_state.genesis_ledger_hash
            inputs.append_field(self.blockchain_state.genesis_ledger_hash.0.to_field());
            // Self::blockchain_state.registers
            // {
            //     let reg = &self.blockchain_state.registers;
            //     inputs.append_field(reg.ledger.to_field());
            //     inputs.append_field(reg.local_state.stack_frame.to_field());
            //     inputs.append_field(reg.local_state.call_stack.to_field());
            //     inputs.append_field(reg.local_state.transaction_commitment.to_field());
            //     inputs.append_field(reg.local_state.full_transaction_commitment.to_field());
            //     inputs.append_field(reg.local_state.token_id.to_field());
            //     inputs.append_u64(reg.local_state.excess.magnitude.as_u64());
            //     inputs.append_bool(matches!(reg.local_state.excess.sgn.0, SgnStableV1::Pos));
            //     inputs.append_u64(reg.local_state.supply_increase.magnitude.as_u64());
            //     inputs.append_bool(matches!(
            //         reg.local_state.supply_increase.sgn.0,
            //         SgnStableV1::Pos
            //     ));
            //     inputs.append_field(reg.local_state.ledger.to_field());
            //     inputs.append_u32(reg.local_state.account_update_index.as_u32());
            //     inputs.append_bool(reg.local_state.success);
            // }
            inputs.append_u64(self.blockchain_state.timestamp.0 .0.as_u64());
            inputs.append_bytes(self.blockchain_state.body_reference.0 .0.as_ref());
        }

        // CONSENSUS
        {
            let consensus = &self.consensus_state;
            inputs.append_u32(consensus.blockchain_length.0.as_u32());
            inputs.append_u32(consensus.epoch_count.0.as_u32());
            inputs.append_u32(consensus.min_window_density.0.as_u32());
            for window in &consensus.sub_window_densities {
                inputs.append_u32(window.0.as_u32());
            }
            {
                let vrf: &[u8] = consensus.last_vrf_output.0.as_ref();
                inputs.append_bytes(&vrf[..31]);
                // Ignore the last 3 bits
                let last_byte = vrf[31];
                for bit in [1, 2, 4, 8, 16] {
                    inputs.append_bool(last_byte & bit != 0);
                }
            }
            inputs.append_u64(consensus.total_currency.0 .0.as_u64());
            inputs.append_u32(consensus.curr_global_slot.slot_number.0.as_u32());
            inputs.append_u32(consensus.curr_global_slot.slots_per_epoch.0.as_u32());
            inputs.append_u32(consensus.global_slot_since_genesis.0.as_u32());
            inputs.append_bool(consensus.has_ancestor_in_same_checkpoint_window);
            inputs.append_bool(consensus.supercharge_coinbase);

            let staking_epoch_data = &consensus.staking_epoch_data;
            inputs.append_field(staking_epoch_data.seed.0.to_field());
            inputs.append_field(staking_epoch_data.start_checkpoint.0.to_field());
            inputs.append_u32(staking_epoch_data.epoch_length.0.as_u32());
            inputs.append_field(staking_epoch_data.ledger.hash.0.to_field());
            inputs.append_u64(staking_epoch_data.ledger.total_currency.0 .0.as_u64());
            inputs.append_field(staking_epoch_data.lock_checkpoint.0.to_field());

            let next_epoch_data = &consensus.next_epoch_data;
            inputs.append_field(next_epoch_data.seed.0.to_field());
            inputs.append_field(next_epoch_data.start_checkpoint.0.to_field());
            inputs.append_u32(next_epoch_data.epoch_length.0.as_u32());
            inputs.append_field(next_epoch_data.ledger.hash.0.to_field());
            inputs.append_u64(next_epoch_data.ledger.total_currency.0 .0.as_u64());
            inputs.append_field(next_epoch_data.lock_checkpoint.0.to_field());

            inputs.append_field(consensus.block_stake_winner.x.to_field());
            inputs.append_bool(consensus.block_stake_winner.is_odd);
            inputs.append_field(consensus.block_creator.x.to_field());
            inputs.append_bool(consensus.block_creator.is_odd);
            inputs.append_field(consensus.coinbase_receiver.x.to_field());
            inputs.append_bool(consensus.coinbase_receiver.is_odd);
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
        let previous_state_hash = self.previous_state_hash.0.to_field();
        let body_hash = self.body.hash();

        hashes_abstract(previous_state_hash, body_hash)
    }
}
