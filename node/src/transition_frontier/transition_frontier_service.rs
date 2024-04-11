use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use ledger::scan_state::scan_state::{transaction_snark::OneOrTwo, AvailableJobMessage};
use mina_p2p_messages::v2::{
    LedgerHash, MinaLedgerSyncLedgerAnswerStableV2, MinaLedgerSyncLedgerQueryStableV1,
    MinaStateProtocolStateValueStableV2, StateHash,
};
use openmina_core::block::ArcBlockWithHash;

use crate::p2p::channels::rpc::StagedLedgerAuxAndPendingCoinbases;

use super::sync::TransitionFrontierRootSnarkedLedgerUpdates;

#[derive(Debug, Default)]
pub struct CommitResult {
    pub available_jobs: Vec<OneOrTwo<AvailableJobMessage>>,
    pub needed_protocol_states: BTreeSet<StateHash>,
}

pub trait TransitionFrontierService: redux::Service {
    fn block_apply(
        &self,
        block: ArcBlockWithHash,
        pred_block: ArcBlockWithHash,
    ) -> Result<(), String>;
    fn commit(
        &self,
        ledgers_to_keep: BTreeSet<LedgerHash>,
        root_snarked_ledger_updates: TransitionFrontierRootSnarkedLedgerUpdates,
        needed_protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
        new_root: &ArcBlockWithHash,
        new_best_tip: &ArcBlockWithHash,
    ) -> CommitResult;
    fn answer_ledger_query(
        &self,
        ledger_hash: LedgerHash,
        query: MinaLedgerSyncLedgerQueryStableV1,
    ) -> Option<MinaLedgerSyncLedgerAnswerStableV2>;
    fn staged_ledger_aux_and_pending_coinbase(
        &self,
        ledger_hash: LedgerHash,
        protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
    ) -> Option<Arc<StagedLedgerAuxAndPendingCoinbases>>;
}
