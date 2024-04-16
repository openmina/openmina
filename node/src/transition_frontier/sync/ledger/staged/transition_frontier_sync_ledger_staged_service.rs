use std::sync::Arc;

use mina_p2p_messages::v2::LedgerHash;

use super::StagedLedgerAuxAndPendingCoinbasesValid;

pub trait TransitionFrontierSyncLedgerStagedService: redux::Service {
    fn staged_ledger_reconstruct(
        &self,
        snarked_ledger_hash: LedgerHash,
        parts: Option<Arc<StagedLedgerAuxAndPendingCoinbasesValid>>,
    );
}
