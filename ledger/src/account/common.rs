use ark_ff::{PrimeField, UniformRand, Zero};
use mina_hasher::Fp;
use mina_p2p_messages::{b58, b58version};
use o1_utils::{field_helpers::FieldHelpersError, FieldHelpers};
use serde::{Deserialize, Serialize};

use crate::{
    hash::hash_noinputs,
    proofs::{
        field::{Boolean, FieldWitness, ToBoolean},
        numbers::{
            currency::{CheckedAmount, CheckedBalance},
            nat::{CheckedSlot, CheckedSlotSpan},
        },
        to_field_elements::ToFieldElements,
    },
    scan_state::currency::{Amount, Balance, Magnitude, Slot, SlotSpan},
    ControlTag, ToInputs,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VotingFor(pub Fp);

impl VotingFor {
    pub fn dummy() -> Self {
        Self(Fp::zero())
    }

    pub fn from_base58(s: &str) -> Result<Self, FieldHelpersError> {
        // TODO(tizoc): done in an ugly way, figure out the proper way and update this code
        let decoded =
            b58::decode(s, b58version::STATE_HASH).map_err(|_| FieldHelpersError::DecodeHex)?;
        Ok(Self(Fp::from_be_bytes_mod_order(&decoded[1..])))
    }
}

#[test]
fn test_voting_for_b58decode() {
    let source = "3NK2tkzqqK5spR2sZ7tujjqPksL45M3UUrcA4WhCkeiPtnugyE2x";
    VotingFor::from_base58(source).unwrap();
}

impl ToFieldElements<Fp> for VotingFor {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self(f) = self;
        f.to_field_elements(fields)
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

    pub fn from_base58(s: &str) -> Result<Self, FieldHelpersError> {
        // TODO(tizoc): done in an ugly way, figure out the proper way and update this code
        let decoded = b58::decode(s, b58version::RECEIPT_CHAIN_HASH)
            .map_err(|_| FieldHelpersError::DecodeHex)?;
        Ok(Self(Fp::from_be_bytes_mod_order(&decoded[1..])))
    }

    pub fn gen() -> Self {
        Self(Fp::rand(&mut rand::thread_rng()))
    }
}

#[test]
fn test_receipt_chain_b58decode() {
    let source = "2mzbV7WevxLuchs2dAMY4vQBS6XttnCUF8Hvks4XNBQ5qiSGGBQe";
    ReceiptChainHash::from_base58(source).unwrap();

    let source = "2n2K1aziimdYu5QCf8mU4gducZCB5u5s78sGnp56zT2tig4ugVHD";
    ReceiptChainHash::from_base58(source).unwrap();
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

impl Timing {
    pub fn is_timed(&self) -> bool {
        match self {
            Timing::Untimed => false,
            Timing::Timed { .. } => true,
        }
    }

    pub fn to_record(&self) -> TimingAsRecord {
        match self.clone() {
            Timing::Untimed => TimingAsRecord {
                is_timed: false,
                initial_minimum_balance: Balance::zero(),
                cliff_time: Slot::zero(),
                cliff_amount: Amount::zero(),
                vesting_period: SlotSpan::from_u32(1),
                vesting_increment: Amount::zero(),
            },
            Timing::Timed {
                initial_minimum_balance,
                cliff_time,
                cliff_amount,
                vesting_period,
                vesting_increment,
            } => TimingAsRecord {
                is_timed: true,
                initial_minimum_balance,
                cliff_time,
                cliff_amount,
                vesting_period,
                vesting_increment,
            },
        }
    }

    pub fn to_record_checked<F: FieldWitness>(&self) -> TimingAsRecordChecked<F> {
        let TimingAsRecord {
            is_timed,
            initial_minimum_balance,
            cliff_time,
            cliff_amount,
            vesting_period,
            vesting_increment,
        } = self.to_record();

        TimingAsRecordChecked {
            is_timed: is_timed.to_boolean(),
            initial_minimum_balance: initial_minimum_balance.to_checked(),
            cliff_time: cliff_time.to_checked(),
            cliff_amount: cliff_amount.to_checked(),
            vesting_period: vesting_period.to_checked(),
            vesting_increment: vesting_increment.to_checked(),
        }
    }
}

pub struct TimingAsRecord {
    pub is_timed: bool,
    pub initial_minimum_balance: Balance,
    pub cliff_time: Slot,
    pub cliff_amount: Amount,
    pub vesting_period: SlotSpan,
    pub vesting_increment: Amount,
}

pub struct TimingAsRecordChecked<F: FieldWitness> {
    pub is_timed: Boolean,
    pub initial_minimum_balance: CheckedBalance<F>,
    pub cliff_time: CheckedSlot<F>,
    pub cliff_amount: CheckedAmount<F>,
    pub vesting_period: CheckedSlotSpan<F>,
    pub vesting_increment: CheckedAmount<F>,
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
pub struct AuthRequiredEncoded<Bool> {
    pub constant: Bool,
    pub signature_necessary: Bool,
    pub signature_sufficient: Bool,
}

impl AuthRequired {
    pub fn encode(self) -> AuthRequiredEncoded<bool> {
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

    pub fn verification_key_perm_fallback_to_signature_with_older_version(&self) -> Self {
        use AuthRequired::*;

        match self {
            Impossible | Proof => Signature,
            x => *x,
        }
    }
}

impl AuthRequiredEncoded<bool> {
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
