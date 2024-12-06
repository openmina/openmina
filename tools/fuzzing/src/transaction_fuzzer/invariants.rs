use ledger::{
    scan_state::{
        currency::TxnVersion,
        transaction_logic::zkapp_command::{AccountUpdate, Control},
    },
    Account, AccountId, AuthRequired, ControlTag, Permissions, SetVerificationKey, TokenId,
    ZkAppAccount, TXN_VERSION_CURRENT,
};
use once_cell::sync::Lazy;
use std::sync::{Mutex, RwLock};
use text_diff::{diff, Difference};

pub struct Checks {
    pub caller_id: TokenId,
    pub account_id: AccountId,
    pub account_before: Account,
    pub account_after: Option<Account>,
    pub check_account: Permissions<bool>,
    pub acc_update: AccountUpdate, // added for extra diagnostics
}

pub static BREAK: Lazy<RwLock<bool>> = Lazy::new(
    #[coverage(off)]
    || RwLock::new(false),
);
pub static STATE: Lazy<RwLock<Vec<Checks>>> = Lazy::new(
    #[coverage(off)]
    || RwLock::new(Vec::new()),
);
pub static LAST_RESULT: LastResult = LastResult::new();

pub struct LastResult {
    result: Mutex<Option<Result<(), String>>>,
}

impl LastResult {
    #[coverage(off)]
    pub const fn new() -> Self {
        Self {
            result: Mutex::new(None),
        }
    }

    fn set(&self, result: Result<(), String>) {
        *self.result.lock().unwrap() = Some(result);
    }

    pub fn get(&self) -> Option<Result<(), String>> {
        self.result.lock().unwrap().clone()
    }

    pub fn take(&self) -> Option<Result<(), String>> {
        self.result.lock().unwrap().take()
    }
}

impl Checks {
    // We are duplicating the implementation here because we don't want changes in the logic to affect invariant checks
    #[coverage(off)]
    fn check_permission(auth: AuthRequired, tag: ControlTag) -> bool {
        use AuthRequired as Auth;
        use ControlTag as Tag;

        match (auth, tag) {
            (Auth::Impossible, _) => false,
            (Auth::None, _) => true,
            (Auth::Proof, Tag::Proof) => true,
            (Auth::Signature, Tag::Signature) => true,
            (Auth::Either, Tag::Proof | Tag::Signature) => true,
            (Auth::Signature, Tag::Proof) => false,
            (Auth::Proof, Tag::Signature) => false,
            (Auth::Proof | Auth::Signature | Auth::Either, Tag::NoneGiven) => false,
            (Auth::Both, _) => unimplemented!(),
        }
    }

    #[coverage(off)]
    fn setvk_auth(auth: AuthRequired, txn_version: TxnVersion) -> AuthRequired {
        if txn_version <= TXN_VERSION_CURRENT {
            if txn_version == TXN_VERSION_CURRENT {
                auth
            } else {
                AuthRequired::Signature
            }
        } else {
            panic!("invalid txn_version: {}", txn_version.as_u32());
        }
    }

    #[coverage(off)]
    pub fn new(caller_id: &TokenId, account_update: &AccountUpdate, account: &Account) -> Self {
        let caller_id = caller_id.clone();
        let account_id = account_update.account_id();
        let account_before = account.clone();
        let tag = match account_update.authorization {
            Control::Signature(_) => ControlTag::Signature,
            Control::Proof(_) => ControlTag::Proof,
            Control::NoneGiven => ControlTag::NoneGiven,
        };
        let Permissions::<AuthRequired> {
            edit_state,
            access,
            send,
            receive,
            set_delegate,
            set_permissions,
            set_verification_key: SetVerificationKey { auth, txn_version },
            set_zkapp_uri,
            edit_action_state,
            set_token_symbol,
            increment_nonce,
            set_voting_for,
            set_timing,
        } = account_before.permissions;
        let check_account = Permissions::<bool> {
            edit_state: !Self::check_permission(edit_state, tag),
            access: !Self::check_permission(access, tag),
            send: !Self::check_permission(send, tag),
            receive: !Self::check_permission(receive, tag),
            set_delegate: !Self::check_permission(set_delegate, tag),
            set_permissions: !Self::check_permission(set_permissions, tag),
            set_verification_key: SetVerificationKey {
                auth: !Self::check_permission(Self::setvk_auth(auth, txn_version), tag),
                txn_version,
            },
            set_zkapp_uri: !Self::check_permission(set_zkapp_uri, tag),
            edit_action_state: !Self::check_permission(edit_action_state, tag),
            set_token_symbol: !Self::check_permission(set_token_symbol, tag),
            increment_nonce: !Self::check_permission(increment_nonce, tag),
            set_voting_for: !Self::check_permission(set_voting_for, tag),
            set_timing: !Self::check_permission(set_timing, tag),
        };

        Self {
            caller_id,
            account_id,
            account_before,
            account_after: None,
            check_account,
            acc_update: account_update.clone(),
        }
    }

    #[coverage(off)]
    fn check_authorization(caller_id: &TokenId) -> Result<(), String> {
        for check in STATE.read().unwrap().iter().rev() {
            // find parent's check
            if caller_id == &check.account_id.derive_token_id() {
                if check.check_account.access {
                    return Err("Invariant violation: caller access permission".to_string());
                }

                return Ok(());
            }
        }

        panic!()
    }

    #[coverage(off)]
    pub fn add_after_account(&mut self, account: Account) {
        self.account_after = Some(account);
    }

    #[coverage(off)]
    fn diagnostic(&self, err: &str) -> String {
        let orig = format!("{:#?}", self.account_before);
        let edit = format!("{:#?}", self.account_after.as_ref().unwrap());
        let split = " ";
        let (_, changeset) = diff(orig.as_str(), edit.as_str(), split);

        let mut ret = String::new();

        for seq in changeset {
            match seq {
                Difference::Same(ref x) => {
                    ret.push_str(x);
                    ret.push_str(split);
                }
                Difference::Add(ref x) => {
                    ret.push_str("\x1B[92m");
                    ret.push_str(x);
                    ret.push_str("\x1B[0m");
                    ret.push_str(split);
                }
                Difference::Rem(ref x) => {
                    ret.push_str("\x1B[91m");
                    ret.push_str(x);
                    ret.push_str("\x1B[0m");
                    ret.push_str(split);
                }
            }
        }

        format!("{}\n{}\nUpdate:\n{:#?}\n", err, ret, self.acc_update)
    }

    #[coverage(off)]
    pub fn check_exit(&self) -> Result<(), String> {
        let res = self._check_exit();
        LAST_RESULT.set(res.clone());
        res
    }

    #[coverage(off)]
    fn _check_exit(&self) -> Result<(), String> {
        let Permissions::<bool> {
            edit_state,
            access,
            send,
            receive,
            set_delegate,
            set_permissions,
            set_verification_key:
                SetVerificationKey {
                    auth: set_vk_auth, ..
                },
            set_zkapp_uri,
            edit_action_state,
            set_token_symbol,
            increment_nonce,
            set_voting_for,
            set_timing,
        } = self.check_account;

        // Token owner approval of children account updates
        if self.caller_id != TokenId::default() {
            Self::check_authorization(&self.caller_id)?;
        }

        let account = self.account_after.as_ref().unwrap();

        if access && self.account_before != *account {
            return Err(self.diagnostic("Invariant violation: access permission"));
        }

        if send && self.account_before.balance > account.balance {
            return Err(self.diagnostic("Invariant violation: send permission"));
        }

        if receive && self.account_before.balance < account.balance {
            return Err(self.diagnostic("Invariant violation: receive permission"));
        }

        let is_change_from_none_to_default = self.account_before.delegate.is_none()
            && account.delegate.as_ref() == Some(&account.public_key);
        if set_delegate
            && self.account_before.delegate != account.delegate
            && !is_change_from_none_to_default
        {
            return Err(self.diagnostic("Invariant violation: set_delegate permission"));
        }

        if set_permissions && self.account_before.permissions != account.permissions {
            return Err(self.diagnostic("Invariant violation: set_permissions permission"));
        }

        if set_token_symbol && self.account_before.token_symbol != account.token_symbol {
            return Err(self.diagnostic("Invariant violation: set_token_symbol permission"));
        }

        if increment_nonce && self.account_before.nonce != account.nonce {
            return Err(self.diagnostic("Invariant violation: increment_nonce permission"));
        }

        if set_voting_for && self.account_before.voting_for != account.voting_for {
            return Err(self.diagnostic("Invariant violation: set_voting_for permission"));
        }

        if set_timing && self.account_before.timing != account.timing {
            return Err(self.diagnostic("Invariant violation: set_timing permission"));
        }

        let default_values = ZkAppAccount::default();

        if edit_state {
            let invariant_violation = match &self.account_before.zkapp {
                Some(zkapp) => account.zkapp.as_ref().map_or(
                    true,
                    #[coverage(off)]
                    |x| x.app_state != zkapp.app_state,
                ),
                None => account.zkapp.as_ref().map_or(
                    false,
                    #[coverage(off)]
                    |x| x.app_state != default_values.app_state,
                ),
            };

            if invariant_violation {
                return Err(self.diagnostic("Invariant violation: edit_state permission"));
            }
        }

        if set_vk_auth {
            let invariant_violation = match &self.account_before.zkapp {
                Some(zkapp) => account.zkapp.as_ref().map_or(
                    true,
                    #[coverage(off)]
                    |x| x.verification_key != zkapp.verification_key,
                ),
                None => account.zkapp.as_ref().map_or(
                    false,
                    #[coverage(off)]
                    |x| x.verification_key != default_values.verification_key,
                ),
            };

            if invariant_violation {
                return Err(self.diagnostic("Invariant violation: set_verification_key permission"));
            }
        }

        if set_zkapp_uri {
            let invariant_violation = match &self.account_before.zkapp {
                Some(zkapp) => account.zkapp.as_ref().map_or(
                    true,
                    #[coverage(off)]
                    |x| x.zkapp_uri != zkapp.zkapp_uri,
                ),
                None => account.zkapp.as_ref().map_or(
                    false,
                    #[coverage(off)]
                    |x| x.zkapp_uri != default_values.zkapp_uri,
                ),
            };

            if invariant_violation {
                return Err(self.diagnostic("Invariant violation: set_zkapp_uri permission"));
            }
        }

        if edit_action_state {
            let invariant_violation = match &self.account_before.zkapp {
                Some(zkapp) => account.zkapp.as_ref().map_or(
                    true,
                    #[coverage(off)]
                    |x| x.action_state != zkapp.action_state,
                ),
                None => account.zkapp.as_ref().map_or(
                    false,
                    #[coverage(off)]
                    |x| x.action_state != default_values.action_state,
                ),
            };

            if invariant_violation {
                return Err(self.diagnostic("Invariant violation: edit_action_state permission"));
            }
        }

        Ok(())
    }
}
