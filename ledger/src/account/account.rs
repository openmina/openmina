use std::{borrow::Cow, str::FromStr};

use ark_ff::{One, Zero};
use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

use crate::hash::{hash_noinputs, hash_with_kimchi, Inputs};

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

// TODO: Not sure if the name is correct
// It seems that a similar type exist in proof-systems: TODO
#[derive(Copy, Clone, Debug)]
struct CurveAffine(Fp, Fp);

// https://github.com/MinaProtocol/mina/blob/a6e5f182855b3f4b4afb0ea8636760e618e2f7a0/src/lib/pickles_types/plonk_verification_key_evals.ml#L9-L18
#[derive(Clone, Debug)]
struct PlonkVerificationKeyEvals {
    sigma: [CurveAffine; 7],
    coefficients: [CurveAffine; 15],
    generic: CurveAffine,
    psm: CurveAffine,
    complete_add: CurveAffine,
    mul: CurveAffine,
    emul: CurveAffine,
    endomul_scalar: CurveAffine,
}

#[derive(Clone, Debug)]
enum ProofVerified {
    N0,
    N1,
    N2,
}

#[derive(Clone, Debug)]
struct VerificationKey {
    max_proofs_verified: ProofVerified,
    wrap_index: PlonkVerificationKeyEvals,
    // `wrap_vk` is not used for hash inputs
    wrap_vk: Option<()>,
}

impl VerificationKey {
    // https://github.com/MinaProtocol/mina/blob/35b1702fbc295713f9bb46bb17e2d007bc2bab84/src/lib/pickles/side_loaded_verification_key.ml#L295-L309
    fn dummy() -> Self {
        let g = CurveAffine(
            Fp::one(),
            Fp::from_str(
                "12418654782883325593414442427049395787963493412651469444558597405572177144507",
            )
            .unwrap(),
        );
        Self {
            max_proofs_verified: ProofVerified::N2,
            wrap_index: PlonkVerificationKeyEvals {
                sigma: [g; 7],
                coefficients: [g; 15],
                generic: g,
                psm: g,
                complete_add: g,
                mul: g,
                emul: g,
                endomul_scalar: g,
            },
            wrap_vk: None,
        }
    }

    fn hash(&self) -> Fp {
        let mut inputs = Inputs::new();

        // https://github.com/MinaProtocol/mina/blob/35b1702fbc295713f9bb46bb17e2d007bc2bab84/src/lib/pickles_base/proofs_verified.ml#L108-L118
        let bits = match self.max_proofs_verified {
            ProofVerified::N0 => [true, false, false],
            ProofVerified::N1 => [false, true, false],
            ProofVerified::N2 => [false, false, true],
        };

        for bit in bits {
            inputs.append_bool(bit);
        }

        let index = &self.wrap_index;

        for field in index.sigma {
            inputs.append_field(field.0);
            inputs.append_field(field.1);
        }

        for field in index.coefficients {
            inputs.append_field(field.0);
            inputs.append_field(field.1);
        }

        inputs.append_field(index.generic.0);
        inputs.append_field(index.generic.1);

        inputs.append_field(index.psm.0);
        inputs.append_field(index.psm.1);

        inputs.append_field(index.complete_add.0);
        inputs.append_field(index.complete_add.1);

        inputs.append_field(index.mul.0);
        inputs.append_field(index.mul.1);

        inputs.append_field(index.emul.0);
        inputs.append_field(index.emul.1);

        inputs.append_field(index.endomul_scalar.0);
        inputs.append_field(index.endomul_scalar.1);

        hash_with_kimchi("CodaSideLoadedVk", &inputs.to_fields())
    }
}

// TODO: Fill this struct
// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/zkapp_account.ml#L148-L170
#[derive(Clone, Debug)]
pub struct ZkAppAccount {
    app_state: Vec<Fp>,
    verification_key: Option<VerificationKey>,
    zkapp_version: u32,
    sequence_state: [Fp; 5],
    last_sequence_slot: Slot,
    proved_state: bool,
}

impl Default for ZkAppAccount {
    fn default() -> Self {
        Self {
            app_state: vec![Fp::zero(); 8],
            verification_key: None,
            zkapp_version: 0,
            sequence_state: {
                let empty = hash_noinputs("MinaSnappSequenceEmpty");
                [empty, empty, empty, empty, empty]
            },
            last_sequence_slot: 0,
            proved_state: false,
        }
    }
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
        )
        .unwrap();

        Self {
            public_key: pubkey.clone(),
            token_id: TokenId::default(),
            token_permissions: TokenPermissions::default(),
            token_symbol: String::new(),
            balance: 10101,
            nonce: 0,
            receipt_chain_hash: ReceiptChainHash::empty(),
            delegate: Some(pubkey),
            voting_for: VotingFor::dummy(),
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

    pub fn hash(&self) -> Fp {
        let mut inputs = Inputs::new();

        // Self::zkapp_uri
        // Note: This doesn't cover when zkapp_uri is None, which
        // is never the case for accounts
        let field_zkapp_uri = {
            let mut bits = vec![true; self.zkapp_uri.len() * 8 + 1];
            for (i, c) in self.zkapp_uri.as_bytes().iter().enumerate() {
                for j in 0..8 {
                    bits[(i * 8) + j] = (c & (1 << j)) != 0;
                }
            }

            let mut inputs = Inputs::new();
            for bit in bits {
                inputs.append_bool(bit);
            }

            hash_with_kimchi("MinaZkappUri", &inputs.to_fields())
        };

        inputs.append_field(field_zkapp_uri);

        // Self::zkapp
        let field_zkapp = {
            let zkapp = match self.zkapp.as_ref() {
                Some(zkapp) => Cow::Borrowed(zkapp),
                None => Cow::Owned(ZkAppAccount::default()),
            };
            let zkapp = zkapp.as_ref();

            let mut inputs = Inputs::new();

            inputs.append_bool(zkapp.proved_state);
            inputs.append_u32(zkapp.last_sequence_slot);
            for fp in &zkapp.sequence_state {
                inputs.append_field(*fp);
            }
            inputs.append_u32(zkapp.zkapp_version);
            let vk_hash = match zkapp.verification_key.as_ref() {
                Some(vk) => vk.hash(),
                None => VerificationKey::dummy().hash(),
            };
            inputs.append_field(vk_hash);
            for fp in &zkapp.app_state {
                inputs.append_field(*fp);
            }

            hash_with_kimchi("CodaZkappAccount", &inputs.to_fields())
        };

        inputs.append_field(field_zkapp);

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

        // Self::timing
        match self.timing {
            Timing::Untimed => {
                inputs.append_bool(false);
                inputs.append_u64(0); // initial_minimum_balance
                inputs.append_u32(0); // cliff_time
                inputs.append_u64(0); // cliff_amount
                inputs.append_u32(1); // vesting_period
                inputs.append_u64(0); // vesting_increment
            }
            Timing::Timed {
                initial_minimum_balance,
                cliff_time,
                cliff_amount,
                vesting_period,
                vesting_increment,
            } => {
                inputs.append_bool(true);
                inputs.append_u64(initial_minimum_balance);
                inputs.append_u32(cliff_time);
                inputs.append_u64(cliff_amount);
                inputs.append_u32(vesting_period);
                inputs.append_u64(vesting_increment);
            }
        }

        // Self::voting_for
        inputs.append_field(self.voting_for.0);

        // Self::delegate
        match self.delegate.as_ref() {
            Some(delegate) => {
                inputs.append_field(delegate.x);
                inputs.append_bool(delegate.is_odd);
            }
            None => {
                // Public_key.Compressed.empty
                inputs.append_field(Fp::zero());
                inputs.append_bool(false);
            }
        }

        // Self::receipt_chain_hash
        inputs.append_field(self.receipt_chain_hash.0);

        // Self::nonce
        inputs.append_u32(self.nonce);

        // Self::balance
        inputs.append_u64(self.balance);

        // Self::token_symbol

        // https://github.com/MinaProtocol/mina/blob/2fac5d806a06af215dbab02f7b154b4f032538b7/src/lib/mina_base/account.ml#L97
        assert!(self.token_symbol.len() <= 6);

        let mut s = <[u8; 6]>::default();
        if !self.token_symbol.is_empty() {
            let len = self.token_symbol.len();
            s[..len].copy_from_slice(&self.token_symbol.as_bytes());
        }
        inputs.append_u48(s);

        // Self::token_permissions
        match self.token_permissions {
            TokenPermissions::TokenOwned {
                disable_new_accounts,
            } => {
                let bit = if disable_new_accounts { 1 } else { 0 };
                inputs.append_u2(0b10 & bit);
            }
            TokenPermissions::NotOwned { account_disabled } => {
                let bit = if account_disabled { 1 } else { 0 };
                inputs.append_u2(0b00 & bit);
            }
        }

        // Self::token_id
        inputs.append_field(self.token_id.0.into());

        // Self::public_key
        inputs.append_field(self.public_key.x);
        inputs.append_bool(self.public_key.is_odd);

        hash_with_kimchi("CodaAccount", &inputs.to_fields())
    }
}

#[cfg(test)]
mod tests {
    use o1_utils::FieldHelpers;

    use super::*;

    #[test]
    fn test_hash_account() {
        let acc = Account::create();
        let hash = acc.hash();

        println!("account_hash={}", hash.to_string());
        println!("account_hash={}", hash.to_hex());

        assert_eq!(
            hash.to_hex(),
            "29ed0b3d0e00d8e24a86752291e90834bcccfee0953441e29f83c89a8e51ef37"
        );

        let acc = Account {
            public_key: CompressedPubKey::from_address(
                "B62qnzbXmRNo9q32n4SNu2mpB8e7FYYLH8NmaX6oFCBYjjQ8SbD7uzV",
            )
            .unwrap(),
            token_id: TokenId::default(),
            token_permissions: TokenPermissions::default(),
            token_symbol: "seb".to_string(),
            balance: 10101,
            nonce: 62772,
            receipt_chain_hash: ReceiptChainHash::empty(),
            delegate: None,
            voting_for: VotingFor::dummy(),
            timing: Timing::Untimed,
            permissions: Permissions::user_default(),
            zkapp: None,
            zkapp_uri: "https://target/release/deps/mina_tree-6ee5ea26e91aacf6".to_string(),
        };

        assert_eq!(
            acc.hash().to_hex(),
            "080ed90fa2552976f8ec3ada5a5d613ef0f6741b7ae1c60573105c6a146c942f"
        );
    }

    #[test]
    fn test_dummy_sideloaded_verification_key() {
        assert_eq!(
            VerificationKey::dummy().hash().to_hex(),
            "bda165a90435d2ecd2577002c32ee361e08fb3bbcb0445c9316d36992a470323"
        );
    }
}
