use mina_p2p_messages::v2::{LedgerHash, MinaBaseAccountBinableArgStableV2};

use crate::ledger::LedgerAddress;

pub trait TransitionFrontierSyncLedgerSnarkedService: redux::Service {
    /// For the given ledger, compute the merkle root hash, forcing
    /// all pending hashes to be computed too.
    fn compute_snarked_ledger_hashes(&self, snarked_ledger_hash: &LedgerHash)
        -> Result<(), String>;

    /// Creates a new copy of the ledger stored under the `origin` hash
    /// and stores it under the `target` hash. If `overwrite` is false,
    /// only copy the ledger if the target doesn't exist already.
    fn copy_snarked_ledger_contents_for_sync(
        &self,
        origin: LedgerHash,
        target: LedgerHash,
        overwrite: bool,
    ) -> Result<bool, String>;

    /// For the given ledger, get the two children hashes at the `parent`
    /// address.
    fn child_hashes_get(
        &self,
        snarked_ledger_hash: LedgerHash,
        parent: &LedgerAddress,
    ) -> Result<(LedgerHash, LedgerHash), String>;

    /// For the given ledger, sets all accounts in `accounts` under
    /// the subtree starting at the `parent` address. The result
    /// is the hash computed for that subtree.
    fn accounts_set(
        &self,
        snarked_ledger_hash: LedgerHash,
        parent: &LedgerAddress,
        accounts: Vec<MinaBaseAccountBinableArgStableV2>,
    ) -> Result<LedgerHash, String>;
}
