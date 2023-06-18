#[allow(clippy::module_inception)]
mod sparse_ledger;
mod sparse_ledger_impl;

use mina_hasher::Fp;
pub use sparse_ledger::*;

use crate::{scan_state::transaction_logic::AccountState, Account, AccountId};

/// Trait used in transaction logic, on the ledger witness (`SparseLedger`), or on mask
///
/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/ledger_intf.ml
/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/sparse_ledger_base.ml
pub trait LedgerIntf {
    type Location: Clone + std::fmt::Debug;

    fn get(&self, addr: &Self::Location) -> Option<Account>;
    fn location_of_account(&self, account_id: &AccountId) -> Option<Self::Location>;
    fn set(&mut self, addr: &Self::Location, account: Account);
    fn get_or_create(
        &mut self,
        account_id: &AccountId,
    ) -> Result<(AccountState, Account, Self::Location), String>;
    fn create_new_account(&mut self, account_id: AccountId, account: Account) -> Result<(), ()>;
    fn remove_accounts_exn(&mut self, account_ids: &[AccountId]);
    fn merkle_root(&mut self) -> Fp;
    fn empty(depth: usize) -> Self;
    fn create_masked(&self) -> Self;
    fn apply_mask(&mut self, mask: Self);

    /// Returns all account locations in this ledger (and its parents if any)
    ///
    /// The result is sorted
    fn account_locations(&self) -> Vec<Self::Location>;
}
