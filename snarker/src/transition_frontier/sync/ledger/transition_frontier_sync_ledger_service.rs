use std::sync::Arc;

use mina_p2p_messages::v2::{LedgerHash, MinaBaseAccountBinableArgStableV2};

use crate::ledger::LedgerAddress;
use crate::p2p::channels::rpc::StagedLedgerAuxAndPendingCoinbases;

pub trait TransitionFrontierSyncLedgerService: redux::Service {
    fn root_set(&mut self, hash: LedgerHash);

    fn hashes_set(
        &mut self,
        parent: &LedgerAddress,
        hashes: (LedgerHash, LedgerHash),
    ) -> Result<(), ()>;

    fn accounts_set(
        &mut self,
        parent: &LedgerAddress,
        accounts: Vec<MinaBaseAccountBinableArgStableV2>,
    ) -> Result<(), ()>;

    fn staged_ledger_reconstruct(
        &mut self,
        parts: Arc<StagedLedgerAuxAndPendingCoinbases>,
    ) -> Result<(), String>;
}
