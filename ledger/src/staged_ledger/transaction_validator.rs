use std::collections::HashMap;

use ark_ff::Zero;
use mina_hasher::Fp;

use crate::{
    scan_state::{
        currency::Slot,
        scan_state::ConstraintConstants,
        transaction_logic::{
            self, protocol_state::ProtocolStateView, signed_command::SignedCommand, AccountState,
            Transaction, TransactionStatus,
        },
    },
    Account, AccountId, Address, Mask,
};

use super::sparse_ledger::LedgerIntf;

#[derive(Clone, Debug)]
pub enum Location {
    Ours(AccountId),
    Theirs(Address),
}

pub struct HashlessLedger {
    base: Mask,
    overlay: HashMap<AccountId, Account>,
}

fn err(s: &str) -> ! {
    panic!(
        "{}: somehow we got a location that isn't present in the underlying ledger",
        s
    )
}

impl HashlessLedger {
    pub fn create(ledger: Mask) -> Self {
        Self {
            base: ledger,
            overlay: HashMap::with_capacity(128),
        }
    }

    fn get_or_create_account(
        &mut self,
        account_id: AccountId,
        account: Account,
    ) -> (AccountState, Location) {
        match self.location_of_account(&account_id) {
            None => {
                self.set(&Location::Ours(account_id.clone()), account);
                (AccountState::Added, Location::Ours(account_id))
            }
            Some(loc) => (AccountState::Existed, loc),
        }
    }

    pub fn apply_transaction(
        &mut self,
        constraint_constants: &ConstraintConstants,
        txn_state_view: &ProtocolStateView,
        transaction: &Transaction,
    ) -> Result<TransactionStatus, String> {
        transaction_logic::apply_transaction(
            constraint_constants,
            txn_state_view,
            self,
            transaction,
        )
        .map(|res| res.transaction_status().clone())
    }

    pub fn apply_user_command(
        &mut self,
        constraint_constants: &ConstraintConstants,
        txn_state_view: &ProtocolStateView,
        txn_global_slot: &Slot,
        user_command: SignedCommand,
    ) -> Result<transaction_logic::transaction_applied::SignedCommandApplied, String> {
        transaction_logic::apply_user_command(
            constraint_constants,
            txn_state_view,
            txn_global_slot,
            self,
            &user_command,
        )
    }
}

impl LedgerIntf for HashlessLedger {
    type Location = Location;

    fn get(&self, addr: &Location) -> Option<crate::Account> {
        match addr {
            Location::Ours(account_id) => self.overlay.get(account_id).cloned(),
            Location::Theirs(addr) => match self.base.get(addr) {
                Some(account) => match self.overlay.get(&account.id()) {
                    None => Some(account),
                    s => s.cloned(),
                },
                None => err("get"),
            },
        }
    }

    fn location_of_account(&self, account_id: &crate::AccountId) -> Option<Location> {
        match self.overlay.get(account_id) {
            Some(_) => Some(Location::Ours(account_id.clone())),
            None => self
                .base
                .location_of_account(account_id)
                .map(Location::Theirs),
        }
    }

    fn set(&mut self, addr: &Location, account: crate::Account) {
        match addr {
            Location::Ours(key) => {
                self.overlay.insert(key.clone(), account);
            }
            Location::Theirs(addr) => match self.base.get(addr) {
                Some(a) => {
                    self.overlay.insert(a.id(), account);
                }
                None => err("set"),
            },
        }
    }

    fn get_or_create(
        &mut self,
        account_id: &crate::AccountId,
    ) -> Result<(AccountState, Account, Location), String> {
        let (action, loc) =
            self.get_or_create_account(account_id.clone(), Account::initialize(account_id));

        let account = self.get(&loc).ok_or_else(|| "get failed".to_string())?;

        Ok((action, account, loc))
    }

    fn create_new_account(&mut self, account_id: AccountId, account: Account) -> Result<(), ()> {
        let (action, _) = self.get_or_create_account(account_id.clone(), account);
        if matches!(action, AccountState::Existed) {
            eprintln!(
                "Could not create a new account with pk {:?}: Account already exists",
                account_id.public_key
            );
            Err(())
        } else {
            Ok(())
        }
    }

    fn remove_accounts_exn(&mut self, _account_ids: &[crate::AccountId]) {
        panic!("hashless_ledger: bug in transaction_logic")
    }

    /// Without any validation that the hashes match, Mina_transaction_logic doesn't really care what this is.
    fn merkle_root(&mut self) -> Fp {
        Fp::zero()
    }

    fn empty(depth: usize) -> Self {
        Self::create(Mask::new_unattached(depth))
    }

    fn create_masked(&self) -> Self {
        Self {
            base: self.base.clone(),
            overlay: self.overlay.clone(),
        }
    }

    fn apply_mask(&self, _mask: Self) {
        todo!()
    }
}
