use ark_ff::{UniformRand, Zero};
use mina_hasher::Fp;
use o1_utils::{field_helpers::FieldHelpersError, FieldHelpers};

use crate::{
    hash::hash_noinputs,
    scan_state::currency::{Amount, Balance, Slot, SlotSpan},
    ControlTag, ToInputs,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VotingFor(pub Fp);

impl VotingFor {
    pub fn dummy() -> Self {
        Self(Fp::zero())
    }
}

impl ToInputs for VotingFor {
    fn to_inputs(&self, inputs: &mut crate::Inputs) {
        inputs.append_field(self.0);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReceiptChainHash(pub Fp);

impl ToInputs for ReceiptChainHash {
    fn to_inputs(&self, inputs: &mut crate::Inputs) {
        inputs.append_field(self.0);
    }
}

impl ReceiptChainHash {
    pub fn empty_legacy() -> Self {
        // Value of `Receipt.Chain_hash.empty` in Ocaml (`compatible` branch)
        Self::from_hex("0b143c0645497a5987a7b88f66340e03db943f0a0df48b69a3a82921ce97b10a").unwrap()
    }

    pub fn empty() -> Self {
        Self(hash_noinputs("CodaReceiptEmpty"))
    }

    pub fn from_hex(s: &str) -> Result<Self, FieldHelpersError> {
        Fp::from_hex(s).map(Self)
    }

    pub fn gen() -> Self {
        Self(Fp::rand(&mut rand::thread_rng()))
    }
}

impl Default for ReceiptChainHash {
    fn default() -> Self {
        Self::empty_legacy()
    }
}

// CodaReceiptEmpty

/// A timed account is an account, which releases its balance to be spent
/// gradually. The process of releasing frozen funds is defined as follows.
/// Until the cliff_time global slot is reached, the initial_minimum_balance
/// of mina is frozen and cannot be spent. At the cliff slot, cliff_amount
/// is released and initial_minimum_balance is effectively lowered by that
/// amount. Next, every vesting_period number of slots, vesting_increment
/// is released, further decreasing the current minimum balance. At some
/// point minimum balance drops to 0, and after that the account behaves
/// like an untimed one. *)
///
/// https://github.com/MinaProtocol/mina/blob/2ff0292b637684ce0372e7b8e23ec85404dc5091/src/lib/mina_base/account_timing.ml#L22
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Timing {
    Untimed,
    Timed {
        initial_minimum_balance: Balance,
        cliff_time: Slot,
        cliff_amount: Amount,
        vesting_period: SlotSpan,
        vesting_increment: Amount,
    },
}

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_numbers/intf.ml#L155
// pub type Nonce = u32;

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/token_permissions.ml#L9
#[derive(Clone, Debug, PartialEq, Eq)]
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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

impl From<ControlTag> for AuthRequired {
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/permissions.ml#L68
    fn from(value: ControlTag) -> Self {
        match value {
            ControlTag::Proof => Self::Proof,
            ControlTag::Signature => Self::Signature,
            ControlTag::NoneGiven => Self::None,
        }
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

    /// permissions such that [check permission (Proof _)] is true
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/permissions.ml#L78
    pub fn gen_for_proof_authorization(rng: &mut rand::rngs::ThreadRng) -> Self {
        use rand::seq::SliceRandom;

        [Self::None, Self::Either, Self::Proof]
            .choose(rng)
            .cloned()
            .unwrap()
    }

    /// permissions such that [check permission (Signature _)] is true
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/permissions.ml#L82
    pub fn gen_for_signature_authorization(rng: &mut rand::rngs::ThreadRng) -> Self {
        use rand::seq::SliceRandom;

        [Self::None, Self::Either, Self::Signature]
            .choose(rng)
            .cloned()
            .unwrap()
    }

    /// permissions such that [check permission None_given] is true
    ///
    /// https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/mina_base/permissions.ml#L86
    pub fn gen_for_none_given_authorization(_rng: &mut rand::rngs::ThreadRng) -> Self {
        Self::None
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
