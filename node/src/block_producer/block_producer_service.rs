use std::collections::BTreeMap;

use mina_p2p_messages::v2::{
    ConsensusBodyReferenceStableV1, LedgerHash, LedgerProofProdStableV2,
    MinaBasePendingCoinbaseUpdateStableV1, MinaBasePendingCoinbaseWitnessStableV2,
    MinaBaseSparseLedgerBaseStableV2, MinaBaseStagedLedgerHashStableV1, NonZeroCurvePoint,
    ProverExtendBlockchainInputStableV2, StagedLedgerDiffDiffStableV2, StateHash,
};
use openmina_core::{
    block::ArcBlockWithHash,
    snark::{Snark, SnarkJobId},
};
use serde::{Deserialize, Serialize};

use openmina_node_account::AccountSecretKey;

use super::BlockProducerWonSlot;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StagedLedgerDiffCreateOutput {
    pub diff: StagedLedgerDiffDiffStableV2,
    /// `protocol_state.blockchain_state.body_reference`
    pub diff_hash: ConsensusBodyReferenceStableV1,
    pub staged_ledger_hash: MinaBaseStagedLedgerHashStableV1,
    pub emitted_ledger_proof: Option<Box<LedgerProofProdStableV2>>,
    pub pending_coinbase_update: MinaBasePendingCoinbaseUpdateStableV1,
    pub pending_coinbase_witness: MinaBasePendingCoinbaseWitnessStableV2,
}

pub trait BlockProducerLedgerService: redux::Service {
    fn staged_ledger_diff_create(
        &self,
        pred_block: &ArcBlockWithHash,
        won_slot: &BlockProducerWonSlot,
        coinbase_receiver: &NonZeroCurvePoint,
        completed_snarks: BTreeMap<SnarkJobId, Snark>,
        supercharge_coinbase: bool,
    ) -> Result<StagedLedgerDiffCreateOutput, String>;

    fn stake_proof_sparse_ledger(
        &self,
        staking_ledger: LedgerHash,
        producer: NonZeroCurvePoint,
        delegator: NonZeroCurvePoint,
    ) -> Option<MinaBaseSparseLedgerBaseStableV2>;
}

pub trait BlockProducerService: BlockProducerLedgerService {
    fn keypair(&mut self) -> Option<AccountSecretKey>;

    fn prove(&mut self, block_hash: StateHash, input: Box<ProverExtendBlockchainInputStableV2>);
}
