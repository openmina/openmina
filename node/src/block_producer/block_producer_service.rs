use std::collections::BTreeMap;

use mina_p2p_messages::v2::{
    ConsensusBodyReferenceStableV1, LedgerProofProdStableV2, MinaBaseStagedLedgerHashStableV1,
    NonZeroCurvePoint, StagedLedgerDiffDiffStableV2,
};
use openmina_core::{
    block::ArcBlockWithHash,
    snark::{Snark, SnarkJobId},
};

use super::BlockProducerWonSlot;

pub struct StagedLedgerDiffCreateOutput {
    pub diff: StagedLedgerDiffDiffStableV2,
    /// `protocol_state.blockchain_state.body_reference`
    pub diff_hash: ConsensusBodyReferenceStableV1,
    pub staged_ledger_hash: MinaBaseStagedLedgerHashStableV1,
    pub emitted_ledger_proof: Option<LedgerProofProdStableV2>,
}

pub trait BlockProducerService: redux::Service {
    fn staged_ledger_diff_create(
        &mut self,
        pred_block: &ArcBlockWithHash,
        won_slot: &BlockProducerWonSlot,
        coinbase_receiver: &NonZeroCurvePoint,
        completed_snarks: BTreeMap<SnarkJobId, Snark>,
        supercharge_coinbase: bool,
    ) -> Result<StagedLedgerDiffCreateOutput, String>;
}
