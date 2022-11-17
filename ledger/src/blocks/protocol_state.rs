use ark_ff::{BigInteger256, PrimeField, UniformRand};
use mina_hasher::Fp;
use mina_p2p_messages::v2::MinaBaseTransactionStatusFailureStableV2;
use mina_signer::CompressedPubKey;
use rand::{rngs::ThreadRng, Rng};
use sha2::{
    digest::{generic_array::GenericArray, typenum::U32},
    Digest, Sha256,
};

use crate::{hash_with_kimchi, Inputs};

#[derive(Clone, Debug)]
pub struct StagedLedgerHashNonSnark {
    pub ledger_hash: Fp,
    pub aux_hash: [u8; 32],             // TODO: In binprot it's a string ?
    pub pending_coinbase_aux: [u8; 32], // TODO: In binprot it's a string ?
}

impl StagedLedgerHashNonSnark {
    fn sha256(&self) -> GenericArray<u8, U32> {
        let mut ledger_hash_bytes: [u8; 32] = [0; 32];

        {
            let ledger_hash: BigInteger256 = self.ledger_hash.into_repr();
            let ledger_hash_iter = ledger_hash.0.iter().rev();

            for (bytes, limb) in ledger_hash_bytes.chunks_exact_mut(8).zip(ledger_hash_iter) {
                let mut limb = limb.to_ne_bytes();
                limb.reverse();
                bytes.copy_from_slice(&limb);
            }
        }

        let mut hasher = Sha256::new();
        hasher.update(ledger_hash_bytes);
        hasher.update(self.aux_hash);
        hasher.update(self.pending_coinbase_aux);

        hasher.finalize()
    }
}

#[derive(Clone, Debug)]
pub struct StagedLedgerHash {
    pub non_snark: StagedLedgerHashNonSnark,
    pub pending_coinbase_hash: Fp,
}

#[derive(Clone, Debug)]
pub enum Sgn {
    Pos,
    Neg,
}

#[derive(Clone, Debug)]
pub struct SignedAmount {
    pub magnitude: i64,
    pub sgn: Sgn,
}

#[derive(Clone, Debug)]
pub struct LocalState {
    pub stack_frame: Fp,
    pub call_stack: Fp,
    pub transaction_commitment: Fp,
    pub full_transaction_commitment: Fp,
    pub token_id: Fp,
    pub excess: SignedAmount,
    pub ledger: Fp,
    pub success: bool,
    // pub party_index: i32,
    pub supply_increase: SignedAmount,
    pub failure_status_tbl: Vec<Vec<MinaBaseTransactionStatusFailureStableV2>>,
    pub account_update_index: u32,
}

#[derive(Clone, Debug)]
pub struct BlockchainStateRegisters {
    pub ledger: Fp,
    pub pending_coinbase_stack: (),
    pub local_state: LocalState,
}

#[derive(Clone, Debug)]
pub struct ConsensusGlobalSlot {
    pub slot_number: u32,
    pub slots_per_epoch: u32,
}

#[derive(Clone, Debug)]
pub struct EpochLedger {
    pub hash: Fp,
    pub total_currency: i64,
}

#[derive(Clone, Debug)]
pub struct DataStaking {
    pub ledger: EpochLedger,
    pub seed: Fp,
    pub start_checkpoint: Fp,
    pub lock_checkpoint: Fp,
    pub epoch_length: u32,
}

#[derive(Clone, Debug)]
pub struct ConsensusState {
    pub blockchain_length: u32,
    pub epoch_count: u32,
    pub min_window_density: u32,
    pub sub_window_densities: Vec<u32>,
    pub last_vrf_output: [u8; 32], // TODO: In binprot it's a string ?
    pub total_currency: i64,
    pub curr_global_slot: ConsensusGlobalSlot,
    pub global_slot_since_genesis: u32,
    pub staking_epoch_data: DataStaking,
    pub next_epoch_data: DataStaking,
    pub has_ancestor_in_same_checkpoint_window: bool,
    pub block_stake_winner: CompressedPubKey,
    pub block_creator: CompressedPubKey,
    pub coinbase_receiver: CompressedPubKey,
    pub supercharge_coinbase: bool,
}

#[derive(Clone, Debug)]
pub struct BlockchainState {
    pub staged_ledger_hash: StagedLedgerHash,
    pub genesis_ledger_hash: Fp,
    pub registers: BlockchainStateRegisters,
    pub timestamp: u64,
    pub body_reference: [u8; 32], // TODO: In binprot it's a string ?
}

#[derive(Clone, Debug)]
pub struct ProtocolConstants {
    pub k: u32,
    pub slots_per_epoch: u32,
    pub slots_per_sub_window: u32,
    pub delta: u32,
    pub genesis_state_timestamp: u64,
}

#[derive(Clone, Debug)]
pub struct ProtocolStateBody {
    pub genesis_state_hash: Fp,
    pub blockchain_state: BlockchainState,
    pub consensus_state: ConsensusState,
    pub constants: ProtocolConstants,
}

impl ProtocolStateBody {
    pub fn hash(&self) -> Fp {
        let mut inputs = Inputs::new();

        // constants
        {
            inputs.append_u32(self.constants.k);
            inputs.append_u32(self.constants.delta);
            inputs.append_u32(self.constants.slots_per_epoch);
            inputs.append_u32(self.constants.slots_per_sub_window);
            inputs.append_u64(self.constants.genesis_state_timestamp);
        }

        // Genesis
        {
            inputs.append_field(self.genesis_state_hash);
        }

        // This is blockchain_state
        {
            // Self::blockchain_state.staged_ledger_hash
            {
                let staged = &self.blockchain_state.staged_ledger_hash;
                inputs.append_bytes(&staged.non_snark.sha256());
                inputs.append_field(staged.pending_coinbase_hash);
            }
            // Self::blockchain_state.genesis_ledger_hash
            inputs.append_field(self.blockchain_state.genesis_ledger_hash);
            // Self::blockchain_state.registers
            {
                let reg = &self.blockchain_state.registers;
                inputs.append_field(reg.ledger);
                inputs.append_field(reg.local_state.stack_frame);
                inputs.append_field(reg.local_state.call_stack);
                inputs.append_field(reg.local_state.transaction_commitment);
                inputs.append_field(reg.local_state.full_transaction_commitment);
                inputs.append_field(reg.local_state.token_id);
                inputs.append_u64(reg.local_state.excess.magnitude as u64);
                inputs.append_bool(matches!(reg.local_state.excess.sgn, Sgn::Pos));
                inputs.append_u64(reg.local_state.supply_increase.magnitude as u64);
                inputs.append_bool(matches!(reg.local_state.supply_increase.sgn, Sgn::Pos));
                inputs.append_field(reg.local_state.ledger);
                inputs.append_u32(reg.local_state.account_update_index);
                inputs.append_bool(reg.local_state.success);
            }
            inputs.append_u64(self.blockchain_state.timestamp);
            inputs.append_bytes(&self.blockchain_state.body_reference);
        }

        // CONSENSUS
        {
            let consensus = &self.consensus_state;
            inputs.append_u32(consensus.blockchain_length);
            inputs.append_u32(consensus.epoch_count);
            inputs.append_u32(consensus.min_window_density);
            for window in &consensus.sub_window_densities {
                inputs.append_u32(*window);
            }
            {
                let vrf = &consensus.last_vrf_output;
                inputs.append_bytes(&vrf[..31]);
                // Ignore the last 3 bits
                let last_byte = vrf[31];
                for bit in [1, 2, 4, 8, 16] {
                    inputs.append_bool(last_byte & bit != 0);
                }
            }
            inputs.append_u64(consensus.total_currency as u64);
            inputs.append_u32(consensus.curr_global_slot.slot_number);
            inputs.append_u32(consensus.curr_global_slot.slots_per_epoch);
            inputs.append_u32(consensus.global_slot_since_genesis);
            inputs.append_bool(consensus.has_ancestor_in_same_checkpoint_window);
            inputs.append_bool(consensus.supercharge_coinbase);
            for data in &[&consensus.staking_epoch_data, &consensus.next_epoch_data] {
                inputs.append_field(data.seed);
                inputs.append_field(data.start_checkpoint);
                inputs.append_u32(data.epoch_length);
                inputs.append_field(data.ledger.hash);
                inputs.append_u64(data.ledger.total_currency as u64);
                inputs.append_field(data.lock_checkpoint);
            }
            inputs.append_field(consensus.block_stake_winner.x);
            inputs.append_bool(consensus.block_stake_winner.is_odd);
            inputs.append_field(consensus.block_creator.x);
            inputs.append_bool(consensus.block_creator.is_odd);
            inputs.append_field(consensus.coinbase_receiver.x);
            inputs.append_bool(consensus.coinbase_receiver.is_odd);
        }

        hash_with_kimchi("MinaProtoStateBody", &inputs.to_fields())
    }
}

#[derive(Clone, Debug)]
pub struct ProtocolState {
    pub previous_state_hash: Fp,
    pub body: ProtocolStateBody,
}

impl ProtocolState {
    pub fn hash(&self) -> Fp {
        let mut inputs = Inputs::new();

        inputs.append_field(self.previous_state_hash);

        let body_hash = self.body.hash();
        inputs.append_field(body_hash);

        hash_with_kimchi("MinaProtoState", &inputs.to_fields())
    }

    pub fn rand(rng: &mut ThreadRng) -> Self {
        Self {
            previous_state_hash: Fp::rand(rng),
            body: ProtocolStateBody {
                genesis_state_hash: Fp::rand(rng),
                blockchain_state: BlockchainState {
                    staged_ledger_hash: StagedLedgerHash {
                        non_snark: StagedLedgerHashNonSnark {
                            ledger_hash: Fp::rand(rng),
                            aux_hash: rng.gen(),
                            pending_coinbase_aux: rng.gen(),
                        },
                        pending_coinbase_hash: Fp::rand(rng),
                    },
                    genesis_ledger_hash: Fp::rand(rng),
                    registers: BlockchainStateRegisters {
                        ledger: Fp::rand(rng),
                        pending_coinbase_stack: (),
                        local_state: LocalState {
                            stack_frame: Fp::rand(rng),
                            call_stack: Fp::rand(rng),
                            transaction_commitment: Fp::rand(rng),
                            full_transaction_commitment: Fp::rand(rng),
                            token_id: Fp::rand(rng),
                            excess: SignedAmount {
                                magnitude: rng.gen(),
                                sgn: if rng.gen() { Sgn::Pos } else { Sgn::Neg },
                            },
                            ledger: Fp::rand(rng),
                            success: rng.gen(),
                            failure_status_tbl: Vec::new(), // Not used for hashing
                            supply_increase: SignedAmount {
                                magnitude: rng.gen(),
                                sgn: if rng.gen() { Sgn::Pos } else { Sgn::Neg },
                            },
                            account_update_index: rng.gen(),
                        },
                    },
                    timestamp: rng.gen(),
                    body_reference: rng.gen(),
                },
                consensus_state: ConsensusState {
                    blockchain_length: rng.gen(),
                    epoch_count: rng.gen(),
                    min_window_density: rng.gen(),
                    sub_window_densities: {
                        let n = rng.gen::<u8>() % 20;
                        (0..n.max(1)).map(|_| rng.gen()).collect()
                    },
                    last_vrf_output: rng.gen(),
                    total_currency: rng.gen(),
                    curr_global_slot: ConsensusGlobalSlot {
                        slot_number: rng.gen(),
                        slots_per_epoch: rng.gen(),
                    },
                    global_slot_since_genesis: rng.gen(),
                    staking_epoch_data: DataStaking {
                        ledger: EpochLedger {
                            hash: Fp::rand(rng),
                            total_currency: rng.gen(),
                        },
                        seed: Fp::rand(rng),
                        start_checkpoint: Fp::rand(rng),
                        lock_checkpoint: Fp::rand(rng),
                        epoch_length: rng.gen(),
                    },
                    next_epoch_data: DataStaking {
                        ledger: EpochLedger {
                            hash: Fp::rand(rng),
                            total_currency: rng.gen(),
                        },
                        seed: Fp::rand(rng),
                        start_checkpoint: Fp::rand(rng),
                        lock_checkpoint: Fp::rand(rng),
                        epoch_length: rng.gen(),
                    },
                    has_ancestor_in_same_checkpoint_window: rng.gen(),
                    block_stake_winner: CompressedPubKey {
                        x: Fp::rand(rng),
                        is_odd: rng.gen(),
                    },
                    block_creator: CompressedPubKey {
                        x: Fp::rand(rng),
                        is_odd: rng.gen(),
                    },
                    coinbase_receiver: CompressedPubKey {
                        x: Fp::rand(rng),
                        is_odd: rng.gen(),
                    },
                    supercharge_coinbase: rng.gen(),
                },
                constants: ProtocolConstants {
                    k: rng.gen(),
                    slots_per_epoch: rng.gen(),
                    slots_per_sub_window: rng.gen(),
                    delta: rng.gen(),
                    genesis_state_timestamp: rng.gen(),
                },
            },
        }
    }
}
