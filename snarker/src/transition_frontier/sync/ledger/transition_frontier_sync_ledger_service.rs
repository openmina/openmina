use std::sync::Arc;

use mina_p2p_messages::v2::{LedgerHash, MinaBaseAccountBinableArgStableV2};

use crate::ledger::LedgerAddress;
use crate::p2p::channels::rpc::StagedLedgerAuxAndPendingCoinbases;

pub trait TransitionFrontierSyncLedgerService: redux::Service {
    fn hashes_set(
        &mut self,
        snarked_ledger_hash: LedgerHash,
        parent: &LedgerAddress,
        hashes: (LedgerHash, LedgerHash),
    ) -> Result<(), ()>;

    fn accounts_set(
        &mut self,
        snarked_ledger_hash: LedgerHash,
        parent: &LedgerAddress,
        accounts: Vec<MinaBaseAccountBinableArgStableV2>,
    ) -> Result<(), ()>;

    fn staged_ledger_reconstruct(
        &mut self,
        snarked_ledger_hash: LedgerHash,
        parts: Arc<StagedLedgerAuxAndPendingCoinbases>,
    ) -> Result<(), String>;
}
