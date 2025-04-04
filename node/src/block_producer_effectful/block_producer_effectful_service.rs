//! Defines service interfaces for block producer effectful operations.
//! These interfaces allow the block producer to interact with external services
//! like the prover and account management.

use std::sync::Arc;

use ledger::proofs::provers::BlockProver;
use mina_p2p_messages::v2::{
    ConsensusBodyReferenceStableV1, LedgerProofProdStableV2, MinaBasePendingCoinbaseUpdateStableV1,
    MinaBasePendingCoinbaseWitnessStableV2, MinaBaseSparseLedgerBaseStableV2,
    MinaBaseStagedLedgerHashStableV1, ProverExtendBlockchainInputStableV2,
    StagedLedgerDiffDiffStableV2, StateHash,
};
use openmina_node_account::AccountSecretKey;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Output from the staged ledger diff creation process.
/// Contains all the necessary components for building a block,
/// including the diff itself and related cryptographic material.
pub struct StagedLedgerDiffCreateOutput {
    pub diff: StagedLedgerDiffDiffStableV2,
    /// `protocol_state.blockchain_state.body_reference`
    pub diff_hash: ConsensusBodyReferenceStableV1,
    pub staged_ledger_hash: MinaBaseStagedLedgerHashStableV1,
    pub emitted_ledger_proof: Option<Arc<LedgerProofProdStableV2>>,
    pub pending_coinbase_update: MinaBasePendingCoinbaseUpdateStableV1,
    pub pending_coinbase_witness: MinaBasePendingCoinbaseWitnessStableV2,
    pub stake_proof_sparse_ledger: MinaBaseSparseLedgerBaseStableV2,
}

/// Service interface for block production operations.
/// Provides methods for proving blocks and accessing producer keypairs.
// FACT-CHECKER-WARNING: The service interface is missing methods for block injection to the transition frontier and P2P network, which are critical parts of the block production process.
pub trait BlockProducerService {
    fn provers(&self) -> BlockProver;
    fn prove(&mut self, block_hash: StateHash, input: Box<ProverExtendBlockchainInputStableV2>);
    fn with_producer_keypair<T>(&self, f: impl FnOnce(&AccountSecretKey) -> T) -> Option<T>;
}
