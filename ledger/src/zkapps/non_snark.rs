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
    AccountUpdateInterface, AmountInterface, CallForestInterface, CallStackInterface,
    IndexInterface, Opt, SignedAmountInterface, StackFrameInterface, StackFrameMakeParams,
    StackInterface, WitnessGenerator,
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
    fn make(params: StackFrameMakeParams<'_, Self::Calls>, w: &mut Self::W) -> Self {
        todo!()
    }
    fn on_if(self, w: &mut Self::W) -> Self {
        self
    }
}

impl StackInterface for CallStack {
    type Elt = StackFrame;
    type W = ();
    fn empty() -> Self {
        todo!()
    }
    fn is_empty(&self, w: &mut Self::W) -> Boolean {
        todo!()
    }
    fn pop_exn(&self) -> (Self::Elt, Self) {
        todo!()
    }
    fn pop(&self, w: &mut Self::W) -> Opt<(Self::Elt, Self)> {
        todo!()
    }
    fn push(elt: Self::Elt, onto: Self, w: &mut Self::W) -> Self {
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
    type AccountUpdate = AccountUpdate;

    fn empty() -> Self {
        todo!()
    }
    fn is_empty(&self, w: &mut Self::W) -> Boolean {
        todo!()
    }
    fn pop_exn(&self, w: &mut Self::W) -> ((AccountUpdate, Self), Self) {
        todo!()
    }
}

impl AccountUpdateInterface for AccountUpdate {
    fn body(&self) -> &crate::scan_state::transaction_logic::zkapp_command::Body {
        let Self {
            body,
            authorization: _,
        } = self;
        body
    }
    fn set(&mut self, new: Self) {
        todo!()
    }
    fn verification_key_hash(&self) -> Fp {
        todo!()
    }
    fn is_proved(&self) -> Boolean {
        todo!()
    }
    fn is_signed(&self) -> Boolean {
        todo!()
    }
}
