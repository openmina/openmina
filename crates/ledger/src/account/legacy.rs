use super::common::*;
use crate::scan_state::currency::{Balance, Magnitude, Nonce};
use ark_ff::Zero;
use mina_hasher::{create_legacy, Fp, Hashable, Hasher, ROInput};
use mina_signer::CompressedPubKey;
use o1_utils::FieldHelpers;
use std::borrow::Cow;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TokenIdLegacy(pub u64);

impl Default for TokenIdLegacy {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Clone, Debug)]
pub struct PermissionsLegacy<Controller> {
    pub stake: bool,
    pub edit_state: Controller,
    pub send: Controller,
    pub receive: Controller,
    pub set_delegate: Controller,
    pub set_permissions: Controller,
    pub set_verification_key: Controller,
}

impl PermissionsLegacy<AuthRequired> {
    pub fn user_default() -> Self {
        use AuthRequired::*;

        Self {
            stake: true,
            edit_state: Signature,
            send: Signature,
            receive: None,
            set_delegate: Signature,
            set_permissions: Signature,
            set_verification_key: Signature,
        }
    }

    pub fn empty() -> Self {
        use AuthRequired::*;

        Self {
            stake: false,
            edit_state: None,
            send: None,
            receive: None,
            set_delegate: None,
            set_permissions: None,
            set_verification_key: None,
        }
    }
}

impl Default for PermissionsLegacy<AuthRequired> {
    fn default() -> Self {
        Self::user_default()
    }
}

#[derive(Clone, Debug)]
pub struct SnappAccount {
    pub app_state: Vec<Fp>,
    pub verification_key: Option<Fp>,
}

impl Default for SnappAccount {
    fn default() -> Self {
        Self {
            app_state: vec![Fp::zero(); 8],
            verification_key: None,
        }
    }
}

impl Hashable for SnappAccount {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        let mut roi = ROInput::new();

        if let Some(vk) = self.verification_key.as_ref() {
            roi = roi.append_field(*vk);
        } else {
            roi = roi.append_field(
                // Value of `dummy_vk_hash`:
                // <https://github.com/MinaProtocol/mina/blob/4f765c866b81fa6fed66be52707fd91fd915041d/src/lib/mina_base/snapp_account.ml#L116>
                Fp::from_hex("77a430a03efafd14d72e1a3c45a1fdca8267fcce9a729a1d25128bb5dec69d3f")
                    .unwrap(),
            );
        }

        for field in &self.app_state {
            roi = roi.append_field(*field);
        }

        roi
    }

    fn domain_string(_domain_param: Self::D) -> Option<String> {
        Some("CodaSnappAccount****".to_string())
    }
}

// <https://github.com/MinaProtocol/mina/blob/1765ba6bdfd7c454e5ae836c49979fa076de1bea/src/lib/mina_base/account.ml#L368>
#[derive(Clone, Debug)]
pub struct AccountLegacy {
    pub public_key: CompressedPubKey,         // Public_key.Compressed.t
    pub token_id: TokenIdLegacy,              // Token_id.t
    pub token_permissions: TokenPermissions,  // Token_permissions.t
    pub balance: Balance,                     // Balance.t
    pub nonce: Nonce,                         // Nonce.t
    pub receipt_chain_hash: ReceiptChainHash, // Receipt.Chain_hash.t
    pub delegate: Option<CompressedPubKey>,   // Public_key.Compressed.t option
    pub voting_for: VotingFor,                // State_hash.t
    pub timing: Timing,                       // Timing.t
    pub permissions: PermissionsLegacy<AuthRequired>, // Permissions.t
    pub snap: Option<SnappAccount>,
}

pub fn get_legacy_hash_of<T: Hashable>(init_value: T::D, item: &T) -> Fp {
    let mut hasher = create_legacy::<T>(init_value);
    hasher.update(item);
    hasher.digest()
}

impl Hashable for AccountLegacy {
    type D = ();

    fn to_roinput(&self) -> ROInput {
        let mut roi = ROInput::new();

        // Self::snapp
        let snapp_accout = match self.snap.as_ref() {
            Some(snapp) => Cow::Borrowed(snapp),
            None => Cow::Owned(SnappAccount::default()),
        };
        let snapp_digest = get_legacy_hash_of((), snapp_accout.as_ref());

        roi = roi.append_field(snapp_digest);

        // Self::permissions
        for auth in [
            self.permissions.set_verification_key,
            self.permissions.set_permissions,
            self.permissions.set_delegate,
            self.permissions.receive,
            self.permissions.send,
            self.permissions.edit_state,
        ] {
            for bit in auth.encode().to_bits() {
                roi = roi.append_bool(bit);
            }
        }
        roi = roi.append_bool(self.permissions.stake);

        // Self::timing
        match &self.timing {
            Timing::Untimed => {
                roi = roi.append_bool(false);
                roi = roi.append_u64(0); // initial_minimum_balance
                roi = roi.append_u32(0); // cliff_time
                roi = roi.append_u64(0); // cliff_amount
                roi = roi.append_u32(1); // vesting_period
                roi = roi.append_u64(0); // vesting_increment
            }
            Timing::Timed {
                initial_minimum_balance,
                cliff_time,
                cliff_amount,
                vesting_period,
                vesting_increment,
            } => {
                roi = roi.append_bool(true);
                roi = roi.append_u64(initial_minimum_balance.as_u64());
                roi = roi.append_u32(cliff_time.as_u32());
                roi = roi.append_u64(cliff_amount.as_u64());
                roi = roi.append_u32(vesting_period.as_u32());
                roi = roi.append_u64(vesting_increment.as_u64());
            }
        }

        // Self::voting_for
        roi = roi.append_field(self.voting_for.0);

        // Self::delegate
        match self.delegate.as_ref() {
            Some(delegate) => {
                roi = roi.append_field(delegate.x);
                roi = roi.append_bool(delegate.is_odd);
            }
            None => {
                // Public_key.Compressed.empty
                roi = roi.append_field(Fp::zero());
                roi = roi.append_bool(false);
            }
        }

        // Self::receipt_chain_hash
        roi = roi.append_field(self.receipt_chain_hash.0);

        // Self::nonce
        roi = roi.append_u32(self.nonce.as_u32());

        // Self::balance
        roi = roi.append_u64(self.balance.as_u64());

        // Self::token_permissions
        match self.token_permissions {
            TokenPermissions::TokenOwned {
                disable_new_accounts,
            } => {
                roi = roi.append_bool(true);
                roi = roi.append_bool(disable_new_accounts);
            }
            TokenPermissions::NotOwned { account_disabled } => {
                roi = roi.append_bool(false);
                roi = roi.append_bool(account_disabled);
            }
        }

        // Self::token_id
        roi = roi.append_u64(self.token_id.0);

        // Self::public_key
        roi = roi.append_field(self.public_key.x);
        roi = roi.append_bool(self.public_key.is_odd);

        roi
    }

    fn domain_string(_: ()) -> Option<String> {
        Some("CodaAccount*********".to_string())
    }
}

impl AccountLegacy {
    pub fn create() -> Self {
        let pubkey = CompressedPubKey::from_address(
            "B62qnzbXmRNo9q32n4SNu2mpB8e7FYYLH8NmaX6oFCBYjjQ8SbD7uzV",
        )
        .unwrap();

        Self {
            public_key: pubkey.clone(),
            token_id: TokenIdLegacy::default(),
            token_permissions: TokenPermissions::NotOwned {
                account_disabled: false,
            },
            balance: Balance::from_u64(10101),
            nonce: Nonce::from_u32(62772),
            receipt_chain_hash: ReceiptChainHash::default(),
            delegate: Some(pubkey),
            voting_for: VotingFor::default(),
            timing: Timing::Untimed,
            permissions: PermissionsLegacy::user_default(),
            snap: None,
        }
    }

    pub fn empty() -> Self {
        Self {
            public_key: CompressedPubKey {
                x: Fp::zero(),
                is_odd: false,
            },
            token_id: TokenIdLegacy::default(),
            token_permissions: TokenPermissions::default(),
            balance: Balance::zero(),
            nonce: Nonce::zero(),
            receipt_chain_hash: ReceiptChainHash::empty_legacy(),
            delegate: None,
            voting_for: VotingFor::dummy(),
            timing: Timing::Untimed,
            permissions: PermissionsLegacy::user_default(),
            snap: None,
        }
    }
}
