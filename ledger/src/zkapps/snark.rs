use ark_ff::Zero;
use mina_hasher::Fp;

use crate::{
    proofs::{
        numbers::{
            currency::{CheckedAmount, CheckedSigned},
            nat::{CheckedIndex, CheckedSlot},
        },
        to_field_elements::ToFieldElements,
        witness::{field, Boolean, Check, FieldWitness, ToBoolean, Witness},
        zkapp::{GlobalStateForProof, LedgerWithHash, WithStackHash},
    },
    scan_state::transaction_logic::{
        local_state::{StackFrame, StackFrameChecked, StackFrameCheckedFrame},
        zkapp_command::{AccountUpdate, CallForest, WithHash},
    },
    TokenId,
};

use super::intefaces::{
    AmountInterface, CallForestInterface, CallStackInterface, GlobalSlotSinceGenesisInterface,
    GlobalStateInterface, IndexInterface, SignedAmountInterface, StackFrameInterface,
    StackInterface, WitnessGenerator,
};

use super::intefaces::WitnessGenerator as W;

impl<F: FieldWitness> WitnessGenerator<F> for Witness<F> {
    fn exists<T>(&mut self, data: T) -> T
    where
        T: ToFieldElements<F> + Check<F>,
    {
        self.exists(data)
    }

    fn exists_no_check<T>(&mut self, data: T) -> T
    where
        T: ToFieldElements<F>,
    {
        self.exists_no_check(data)
    }
}

impl SignedAmountInterface for CheckedSigned<Fp, CheckedAmount<Fp>> {
    fn zero() -> Self {
        todo!()
    }
    fn is_neg(&self) -> Boolean {
        todo!()
    }
    fn equal(&self, other: &Self) -> Boolean {
        todo!()
    }
    fn is_non_neg(&self) -> Boolean {
        todo!()
    }
    fn negate(&self) -> Self {
        todo!()
    }
    fn add_flagged(&self, other: &Self) -> (Self, Boolean) {
        todo!()
    }
    fn of_unsigned(fee: impl AmountInterface) -> Self {
        todo!()
    }
}

impl AmountInterface for CheckedAmount<Fp> {
    fn zero() -> Self {
        todo!()
    }
    fn equal(&self, other: &Self) -> Boolean {
        todo!()
    }
    fn add_flagged(&self, other: &Self) -> (Self, Boolean) {
        todo!()
    }
    fn add_signed_flagged(&self, signed: &impl SignedAmountInterface) -> (Self, Boolean) {
        todo!()
    }
    fn of_constant_fee(fee: crate::scan_state::currency::Fee) -> Self {
        todo!()
    }
}

impl CallForestInterface for WithHash<CallForest<AccountUpdate>> {
    type W = Witness<Fp>;

    fn empty() -> Self {
        todo!()
    }
    fn is_empty(&self, w: &mut Self::W) -> Boolean {
        let Self { hash, data: _ } = self;
        let empty = Fp::zero();
        field::equal(empty, *hash, w)
    }
    fn pop_exn(&self) -> ((AccountUpdate, Self), Self) {
        todo!()
    }
}

impl StackFrameInterface for StackFrameChecked {
    type Calls = WithHash<CallForest<AccountUpdate>>;
    type W = Witness<Fp>;

    fn caller(&self) -> crate::TokenId {
        todo!()
    }
    fn caller_caller(&self) -> crate::TokenId {
        todo!()
    }
    fn calls(&self) -> &Self::Calls {
        &self.calls
    }
    fn make(caller: TokenId, caller_caller: TokenId, calls: &Self::Calls, w: &mut Self::W) -> Self {
        let frame = StackFrameCheckedFrame {
            caller,
            caller_caller,
            calls: calls.clone(),
        };
        Self::of_frame(frame, w)
    }
}

impl StackInterface for WithHash<Vec<WithStackHash<WithHash<StackFrame>>>> {
    type Elt = StackFrameChecked;

    fn empty() -> Self {
        WithHash {
            data: Vec::new(),
            hash: Fp::zero(),
        }
    }
    fn is_empty(&self) -> Boolean {
        todo!()
    }
    fn pop_exn(&self) -> (Self::Elt, Self) {
        todo!()
    }
    fn pop(&self) -> Option<(Self::Elt, Self)> {
        todo!()
    }
    fn push(&self, elt: Self::Elt) -> Self {
        todo!()
    }
}
impl CallStackInterface for WithHash<Vec<WithStackHash<WithHash<StackFrame>>>> {
    type StackFrame = StackFrameChecked;
}

impl GlobalStateInterface for GlobalStateForProof {
    type Ledger = LedgerWithHash;

    type SignedAmount = CheckedSigned<Fp, CheckedAmount<Fp>>;

    fn first_pass_ledger(&self) -> Self::Ledger {
        self.first_pass_ledger.clone()
    }
    fn set_first_pass_ledger(&self) -> Self::Ledger {
        todo!()
    }
    fn second_pass_ledger(&self) -> Self::Ledger {
        todo!()
    }
    fn set_second_pass_ledger(&self) -> Self::Ledger {
        todo!()
    }
    fn fee_excess(&self) -> Self::SignedAmount {
        todo!()
    }
    fn supply_increase(&self) -> Self::SignedAmount {
        todo!()
    }
}

impl IndexInterface for CheckedIndex<Fp> {
    fn zero() -> Self {
        todo!()
    }
    fn succ(&self) -> Self {
        todo!()
    }
}

impl GlobalSlotSinceGenesisInterface for CheckedSlot<Fp> {
    fn zero() -> Self {
        todo!()
    }
    fn greater_than(&self, other: &Self) -> Boolean {
        todo!()
    }
    fn equal(&self, other: &Self) -> Boolean {
        todo!()
    }
}
