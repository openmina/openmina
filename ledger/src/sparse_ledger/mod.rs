#[allow(clippy::module_inception)]
mod sparse_ledger;
mod sparse_ledger_impl;

use mina_hasher::Fp;
pub use sparse_ledger::*;

use crate::{
    proofs::zkapp::LedgerWithHash, scan_state::transaction_logic::AccountState, Account, AccountId,
};

/// Trait used in transaction logic, on the ledger witness (`SparseLedger`), or on mask
///
/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/ledger_intf.ml
/// https://github.com/MinaProtocol/mina/blob/05c2f73d0f6e4f1341286843814ce02dcb3919e0/src/lib/mina_base/sparse_ledger_base.ml
pub trait LedgerIntf {
    type Location: Clone + std::fmt::Debug;

    fn get(&self, addr: &Self::Location) -> Option<Box<Account>>;
    fn location_of_account(&self, account_id: &AccountId) -> Option<Self::Location>;
    fn set(&mut self, addr: &Self::Location, account: Box<Account>);
    fn get_or_create(
        &mut self,
        account_id: &AccountId,
    ) -> Result<(AccountState, Box<Account>, Self::Location), String>;
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

#[allow(unused)]
impl LedgerIntf for LedgerWithHash {
    type Location = <SparseLedger as LedgerIntf>::Location;

    fn get(&self, addr: &Self::Location) -> Option<Box<Account>> {
        todo!()
    }
    fn location_of_account(&self, account_id: &AccountId) -> Option<Self::Location> {
        todo!()
    }
    fn set(&mut self, addr: &Self::Location, account: Box<Account>) {
        todo!()
    }
    fn get_or_create(
        &mut self,
        account_id: &AccountId,
    ) -> Result<(AccountState, Box<Account>, Self::Location), String> {
        todo!()
    }
    fn create_new_account(&mut self, account_id: AccountId, account: Account) -> Result<(), ()> {
        todo!()
    }
    fn remove_accounts_exn(&mut self, account_ids: &[AccountId]) {
        todo!()
    }
    fn merkle_root(&mut self) -> Fp {
        todo!()
    }
    fn empty(depth: usize) -> Self {
        todo!()
    }
    fn create_masked(&self) -> Self {
        todo!()
    }
    fn apply_mask(&mut self, mask: Self) {
        todo!()
    }
    fn account_locations(&self) -> Vec<Self::Location> {
        todo!()
    }
}
