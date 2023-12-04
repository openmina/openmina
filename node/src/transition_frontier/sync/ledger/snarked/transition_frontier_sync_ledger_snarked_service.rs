use mina_p2p_messages::v2::{LedgerHash, MinaBaseAccountBinableArgStableV2};

use crate::ledger::LedgerAddress;

pub trait TransitionFrontierSyncLedgerSnarkedService: redux::Service {
    fn hashes_set(
        &mut self,
        snarked_ledger_hash: LedgerHash,
        parent: &LedgerAddress,
        hashes: (LedgerHash, LedgerHash),
    ) -> Result<(), String>;

    fn accounts_set(
        &mut self,
        snarked_ledger_hash: LedgerHash,
        parent: &LedgerAddress,
        accounts: Vec<MinaBaseAccountBinableArgStableV2>,
    ) -> Result<(), ()>;
}
