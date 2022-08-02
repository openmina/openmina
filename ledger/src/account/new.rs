use std::borrow::Cow;

use ark_ff::Zero;
use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

use crate::hash::{hash_with_kimchi, Inputs};

use super::common::*;

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/account.ml#L93
pub type TokenSymbol = String;

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/permissions.mli#L49
#[derive(Clone, Debug)]
pub struct Permissions<Controller> {
    edit_state: Controller,
    send: Controller,
    receive: Controller,
    set_delegate: Controller,
    set_permissions: Controller,
    set_verification_key: Controller,
    set_zkapp_uri: Controller,
    edit_sequence_state: Controller,
    set_token_symbol: Controller,
    increment_nonce: Controller,
    set_voting_for: Controller,
}

impl Default for Permissions<AuthRequired> {
    fn default() -> Self {
        Self::user_default()
    }
}

impl Permissions<AuthRequired> {
    fn user_default() -> Self {
        use AuthRequired::*;
        Self {
            edit_state: Signature,
            send: Signature,
            receive: None,
            set_delegate: Signature,
            set_permissions: Signature,
            set_verification_key: Signature,
            set_zkapp_uri: Signature,
            edit_sequence_state: Signature,
            set_token_symbol: Signature,
            increment_nonce: Signature,
            set_voting_for: Signature,
        }
    }

    fn empty() -> Self {
        use AuthRequired::*;
        Self {
            edit_state: None,
            send: None,
            receive: None,
            set_delegate: None,
            set_permissions: None,
            set_verification_key: None,
            set_zkapp_uri: None,
            edit_sequence_state: None,
            set_token_symbol: None,
            increment_nonce: None,
            set_voting_for: None,
        }
    }
}

// TODO: Fill this struct
// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/zkapp_account.ml#L148-L170
#[derive(Clone, Debug)]
pub struct ZkAppAccount {
    app_state: Vec<u8>,
    verification: (),
    zkapp_version: u32,
    sequence_state: (),
    last_sequence_slot: Slot,
    proved_state: bool,
}

// https://github.com/MinaProtocol/mina/blob/1765ba6bdfd7c454e5ae836c49979fa076de1bea/src/lib/mina_base/account.ml#L368
#[derive(Clone, Debug)]
pub struct Account {
    pub public_key: CompressedPubKey,         // Public_key.Compressed.t
    pub token_id: TokenId,                    // Token_id.t
    pub token_permissions: TokenPermissions,  // Token_permissions.t
    pub token_symbol: TokenSymbol,            // Token_symbol.t
    pub balance: Balance,                     // Balance.t
    pub nonce: Nonce,                         // Nonce.t
    pub receipt_chain_hash: ReceiptChainHash, // Receipt.Chain_hash.t
    pub delegate: Option<CompressedPubKey>,   // Public_key.Compressed.t option
    pub voting_for: VotingFor,                // State_hash.t
    pub timing: Timing,                       // Timing.t
    pub permissions: Permissions<AuthRequired>, // Permissions.t
    pub zkapp: Option<ZkAppAccount>,          // Zkapp_account.t
    pub zkapp_uri: String,                    // string
}

impl Account {
    pub fn create() -> Self {
        let pubkey = CompressedPubKey::from_address(
            "B62qnzbXmRNo9q32n4SNu2mpB8e7FYYLH8NmaX6oFCBYjjQ8SbD7uzV",
            // "B62qiTKpEPjGTSHZrtM8uXiKgn8So916pLmNJKDhKeyBQL9TDb3nvBG", // Public_key.Compressed.empty
        )
        .unwrap();

        Self {
            public_key: pubkey.clone(),
            token_id: TokenId::default(),
            token_permissions: TokenPermissions::NotOwned {
                account_disabled: false,
            },
            token_symbol: "seb".to_string(),
            // token_symbol: String::new(),
            balance: 10101,
            nonce: 62772,
            receipt_chain_hash: ReceiptChainHash::default(),
            delegate: Some(pubkey),
            // delegate: None,
            voting_for: VotingFor::default(),
            timing: Timing::Untimed,
            permissions: Permissions::user_default(),
            zkapp: None,
            zkapp_uri: String::new(),
        }
    }

    pub fn empty() -> Self {
        Self {
            public_key: CompressedPubKey {
                x: Fp::zero().into(),
                is_odd: false,
            },
            token_id: TokenId::default(),
            token_permissions: TokenPermissions::default(),
            token_symbol: String::new(),
            balance: 0,
            nonce: 0,
            receipt_chain_hash: ReceiptChainHash::empty(),
            delegate: None,
            voting_for: VotingFor::dummy(),
            timing: Timing::Untimed,
            permissions: Permissions::user_default(),
            zkapp: None,
            zkapp_uri: String::new(),
        }
    }

    fn hash(&self) -> Fp {
        let mut inputs = Inputs::new();

        // // Self::token_symbol

        // // https://github.com/MinaProtocol/mina/blob/2fac5d806a06af215dbab02f7b154b4f032538b7/src/lib/mina_base/account.ml#L97
        // assert!(self.token_symbol.len() <= 6);

        // let mut s = <[u8; 6]>::default();
        // if !self.token_symbol.is_empty() {
        //     let len = self.token_symbol.len();
        //     s[..len].copy_from_slice(&self.token_symbol.as_bytes());
        // }
        // inputs.append_u48(s);

        // // Self::snapp
        // let snapp_accout = match self.snap.as_ref() {
        //     Some(snapp) => Cow::Borrowed(snapp),
        //     None => Cow::Owned(SnappAccount::default()),
        // };
        // let snapp_digest = get_legacy_hash_of((), snapp_accout.as_ref());

        // inputs.append_field(snapp_digest);

        // // println!("ROINPUT={:?}", inputs);

        // Self::permissions

        for auth in [
            self.permissions.set_voting_for,
            self.permissions.increment_nonce,
            self.permissions.set_token_symbol,
            self.permissions.edit_sequence_state,
            self.permissions.set_zkapp_uri,
            self.permissions.set_verification_key,
            self.permissions.set_permissions,
            self.permissions.set_delegate,
            self.permissions.receive,
            self.permissions.send,
            self.permissions.edit_state,
        ] {
            for bit in auth.encode().to_bits() {
                inputs.append_bool(bit);
            }
        }

        // // Self::timing
        // match self.timing {
        //     Timing::Untimed => {
        //         inputs.append_bool(false);
        //         inputs.append_u64(0); // initial_minimum_balance
        //         inputs.append_u32(0); // cliff_time
        //         inputs.append_u64(0); // cliff_amount
        //         inputs.append_u32(1); // vesting_period
        //         inputs.append_u64(0); // vesting_increment
        //     }
        //     Timing::Timed {
        //         initial_minimum_balance,
        //         cliff_time,
        //         cliff_amount,
        //         vesting_period,
        //         vesting_increment,
        //     } => {
        //         inputs.append_bool(true);
        //         inputs.append_u64(initial_minimum_balance);
        //         inputs.append_u32(cliff_time);
        //         inputs.append_u64(cliff_amount);
        //         inputs.append_u32(vesting_period);
        //         inputs.append_u64(vesting_increment);
        //     }
        // }

        // // Self::voting_for
        // inputs.append_field(self.voting_for.0);

        // // Self::delegate
        // match self.delegate.as_ref() {
        //     Some(delegate) => {
        //         inputs.append_field(delegate.x);
        //         inputs.append_bool(delegate.is_odd);
        //     }
        //     None => {
        //         // Public_key.Compressed.empty
        //         inputs.append_field(Fp::zero());
        //         inputs.append_bool(false);
        //     }
        // }

        // // Self::receipt_chain_hash
        // inputs.append_field(self.receipt_chain_hash.0);

        // // Self::nonce
        // inputs.append_u32(self.nonce);

        // // Self::balance
        // inputs.append_u64(self.balance);

        // // Self::token_permissions
        // match self.token_permissions {
        //     TokenPermissions::TokenOwned {
        //         disable_new_accounts,
        //     } => {
        //         inputs.append_bool(true);
        //         inputs.append_bool(disable_new_accounts);
        //     }
        //     TokenPermissions::NotOwned { account_disabled } => {
        //         inputs.append_bool(false);
        //         inputs.append_bool(account_disabled);
        //     }
        // }

        // // Self::token_id
        // inputs.append_u64(self.token_id.0);

        // // Self::public_key
        // inputs.append_field(self.public_key.x);
        // inputs.append_bool(self.public_key.is_odd);

        println!("INPUTS={:#?}", inputs);

        hash_with_kimchi(Cow::Borrowed("CodaAccount"), &inputs.to_fields())
    }
}

#[cfg(test)]
mod tests {
    use mina_hasher::{create_kimchi, create_legacy, Hasher};

    use super::*;

    #[test]
    fn test_hash_account() {
        let acc = Account::create();
        let hash = acc.hash();

        println!("account_hash={}", hash.to_string());
    }
}
