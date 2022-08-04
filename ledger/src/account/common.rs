use ark_ff::Zero;
use mina_hasher::Fp;
use o1_utils::FieldHelpers;

use crate::{hash::hash_noinputs, tree::TreeVersion};

pub type Balance = u64;

pub type Amount = u64;

// TODO: Not sure if it's the correct type
//       It seems that the token id is a simple number, but on ocaml when they
//       convert it to/from string (base58), they add/remove the byte 0x1C:
//       https://github.com/MinaProtocol/mina/blob/3a35532cb19d17583b63036bc50d8dde5460b791/src/lib/mina_base/account_id.ml#L30
//       need more research
#[derive(Clone, Debug)]
pub struct TokenId(pub u64);

impl Default for TokenId {
    fn default() -> Self {
        Self(1)
    }
}

pub type Slot = u32;

// TODO: Those types are `Field.t` in ocaml
//       not sure how to represent them in Rust, they seem to be 256 bits
// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/receipt.mli#L67
// type VotingFor = [u8; 32];

#[derive(Clone, Debug, Default)]
pub struct VotingFor(pub Fp);

impl VotingFor {
    pub fn dummy() -> Self {
        Self(Fp::zero())
    }
}

#[derive(Clone, Debug)]
pub struct ReceiptChainHash(pub Fp);

impl ReceiptChainHash {
    pub fn empty_legacy() -> Self {
        Self(empty_receipt_hash_legacy())
    }

    pub fn empty() -> Self {
        Self(hash_noinputs("CodaReceiptEmpty"))
    }
}

fn empty_receipt_hash_legacy() -> Fp {
    // Value of `Receipt.Chain_hash.empty` in Ocaml (`compatible` branch)
    Fp::from_hex("0b143c0645497a5987a7b88f66340e03db943f0a0df48b69a3a82921ce97b10a").unwrap()
}

impl Default for ReceiptChainHash {
    fn default() -> Self {
        Self::empty_legacy()
    }
}

// CodaReceiptEmpty

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/account_timing.ml#L31-L34
#[derive(Clone, Debug)]
pub enum Timing {
    Untimed,
    Timed {
        initial_minimum_balance: Balance,
        cliff_time: Slot,
        cliff_amount: Amount,
        vesting_period: Slot,
        vesting_increment: Amount,
    },
}

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_numbers/intf.ml#L155
pub type Nonce = u32;

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/token_permissions.ml#L9
#[derive(Clone, Debug)]
pub enum TokenPermissions {
    TokenOwned { disable_new_accounts: bool },
    NotOwned { account_disabled: bool },
}

impl Default for TokenPermissions {
    fn default() -> Self {
        Self::NotOwned {
            account_disabled: false,
        }
    }
}

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/permissions.mli#L10
#[derive(Copy, Clone, Debug)]
pub enum AuthRequired {
    None,
    Either,
    Proof,
    Signature,
    Impossible,
    Both, // Legacy only
}

impl Default for AuthRequired {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AuthRequiredEncoded {
    constant: bool,
    signature_necessary: bool,
    signature_sufficient: bool,
}

impl AuthRequired {
    pub fn encode(self) -> AuthRequiredEncoded {
        let (constant, signature_necessary, signature_sufficient) = match self {
            AuthRequired::None => (true, false, true),
            AuthRequired::Either => (false, false, true),
            AuthRequired::Proof => (false, false, false),
            AuthRequired::Signature => (false, true, true),
            AuthRequired::Impossible => (true, true, false),
            AuthRequired::Both => (false, true, false),
        };

        AuthRequiredEncoded {
            constant,
            signature_necessary,
            signature_sufficient,
        }
    }
}

impl AuthRequiredEncoded {
    pub fn decode(self) -> AuthRequired {
        match (
            self.constant,
            self.signature_necessary,
            self.signature_sufficient,
        ) {
            (true, _, false) => AuthRequired::Impossible,
            (true, _, true) => AuthRequired::None,
            (false, false, false) => AuthRequired::Proof,
            (false, true, true) => AuthRequired::Signature,
            (false, false, true) => AuthRequired::Either,
            (false, true, false) => AuthRequired::Both,
        }
    }

    pub fn to_bits(self) -> [bool; 3] {
        [
            self.constant,
            self.signature_necessary,
            self.signature_sufficient,
        ]
    }
}
