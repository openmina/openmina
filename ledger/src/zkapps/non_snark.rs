use mina_hasher::Fp;

use crate::{
    proofs::{
        to_field_elements::ToFieldElements,
        witness::{Boolean, Check, FieldWitness},
    },
    scan_state::{
        currency::{Amount, Index, Signed},
        transaction_logic::{
            local_state::{CallStack, StackFrame},
            zkapp_command::{AccountUpdate, CallForest},
        },
    },
    TokenId,
};

use super::intefaces::{
    AmountInterface, CallForestInterface, CallStackInterface, IndexInterface,
    SignedAmountInterface, StackFrameInterface, StackInterface, WitnessGenerator,
};

impl<F: FieldWitness> WitnessGenerator<F> for () {
    fn exists<T>(&mut self, data: T) -> T
    where
        T: ToFieldElements<F> + Check<F>,
    {
        data
    }

    fn exists_no_check<T>(&mut self, data: T) -> T
    where
        T: ToFieldElements<F>,
    {
        data
    }
}

impl SignedAmountInterface for Signed<Amount> {
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

impl StackFrameInterface for StackFrame {
    type Calls = CallForest<AccountUpdate>;
    type W = ();

    fn caller(&self) -> crate::TokenId {
        todo!()
    }
    fn caller_caller(&self) -> crate::TokenId {
        todo!()
    }
    fn calls(&self) -> &CallForest<AccountUpdate> {
        todo!()
    }
    fn make(caller: TokenId, caller_caller: TokenId, calls: &Self::Calls, w: &mut Self::W) -> Self {
        todo!()
    }
}

impl StackInterface for CallStack {
    type Elt = StackFrame;
    fn empty() -> Self {
        todo!()
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

impl CallStackInterface for CallStack {
    type StackFrame = StackFrame;
}

impl IndexInterface for Index {
    fn zero() -> Self {
        todo!()
    }
    fn succ(&self) -> Self {
        todo!()
    }
}

impl CallForestInterface for CallForest<AccountUpdate> {
    type W = ();

    fn empty() -> Self {
        todo!()
    }
    fn is_empty(&self, w: &mut Self::W) -> Boolean {
        todo!()
    }
    fn pop_exn(&self) -> ((AccountUpdate, Self), Self) {
        todo!()
    }
}
