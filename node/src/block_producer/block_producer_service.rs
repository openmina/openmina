use std::sync::Arc;

use ledger::proofs::gates::Provers;
use mina_p2p_messages::v2::{
    ConsensusBodyReferenceStableV1, LedgerProofProdStableV2, MinaBasePendingCoinbaseUpdateStableV1,
    MinaBasePendingCoinbaseWitnessStableV2, MinaBaseSparseLedgerBaseStableV2,
    MinaBaseStagedLedgerHashStableV1, ProverExtendBlockchainInputStableV2,
    StagedLedgerDiffDiffStableV2, StateHash,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StagedLedgerDiffCreateOutput {
    pub diff: StagedLedgerDiffDiffStableV2,
    /// `protocol_state.blockchain_state.body_reference`
    pub diff_hash: ConsensusBodyReferenceStableV1,
    pub staged_ledger_hash: MinaBaseStagedLedgerHashStableV1,
    pub emitted_ledger_proof: Option<Box<LedgerProofProdStableV2>>,
    pub pending_coinbase_update: MinaBasePendingCoinbaseUpdateStableV1,
    pub pending_coinbase_witness: MinaBasePendingCoinbaseWitnessStableV2,
    pub stake_proof_sparse_ledger: MinaBaseSparseLedgerBaseStableV2,
}

pub trait BlockProducerService {
    fn provers(&self) -> Arc<Provers>;
    fn prove(&mut self, block_hash: StateHash, input: Box<ProverExtendBlockchainInputStableV2>);
}
