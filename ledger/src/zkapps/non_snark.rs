use mina_hasher::Fp;

use crate::{
    proofs::{
        field::{Boolean, FieldWitness},
        to_field_elements::ToFieldElements,
        witness::Check,
    },
    scan_state::{
        currency::{Amount, Index, Signed},
        transaction_logic::{
            local_state::{CallStack, StackFrame},
            zkapp_command::{AccountUpdate, CallForest, CheckAuthorizationResult},
        },
    },
};

use super::intefaces::{
    AccountUpdateInterface, AmountInterface, BoolInterface, BranchEvaluation, BranchInterface,
    BranchParam, CallForestInterface, CallStackInterface, IndexInterface, Opt,
    SignedAmountBranchParam, SignedAmountInterface, StackFrameInterface, StackFrameMakeParams,
    StackInterface, WitnessGenerator,
};

impl<F: FieldWitness> WitnessGenerator<F> for () {
    type Bool = Boolean;

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
    fn exists_no_check_on_bool<T>(&mut self, _b: Self::Bool, data: T) -> T
    where
        T: ToFieldElements<F>,
    {
        data
    }
}

impl AmountInterface for Amount {
    type W = ();
    type Bool = Boolean;
    fn zero() -> Self {
        todo!()
    }
    fn equal(&self, other: &Self) -> Self::Bool {
        todo!()
    }
    fn add_flagged(&self, other: &Self, w: &mut Self::W) -> (Self, Self::Bool) {
        todo!()
    }
    fn add_signed_flagged(&self, signed: &impl SignedAmountInterface) -> (Self, Self::Bool) {
        todo!()
    }
    fn of_constant_fee(fee: crate::scan_state::currency::Fee) -> Self {
        todo!()
    }
}

impl SignedAmountInterface for Signed<Amount> {
    type W = ();
    type Bool = Boolean;
    type Amount = Amount;

    fn zero() -> Self {
        todo!()
    }
    fn is_neg(&self) -> Self::Bool {
        todo!()
    }
    fn equal(&self, other: &Self, w: &mut Self::W) -> Self::Bool {
        todo!()
    }
    fn is_non_neg(&self) -> Self::Bool {
        todo!()
    }
    fn negate(&self) -> Self {
        todo!()
    }
    fn add_flagged(&self, other: &Self, w: &mut Self::W) -> (Self, Self::Bool) {
        todo!()
    }
    fn of_unsigned(unsigned: Self::Amount) -> Self {
        todo!()
    }
    fn on_if<'a>(
        b: Self::Bool,
        param: SignedAmountBranchParam<&'a Self>,
        w: &mut Self::W,
    ) -> &'a Self {
        todo!()
    }
}

impl StackFrameInterface for StackFrame {
    type Calls = CallForest<AccountUpdate>;
    type W = ();
    type Bool = Boolean;

    fn caller(&self) -> crate::TokenId {
        todo!()
    }
    fn caller_caller(&self) -> crate::TokenId {
        todo!()
    }
    fn calls(&self) -> &CallForest<AccountUpdate> {
        todo!()
    }
    fn make(params: StackFrameMakeParams<'_, Self::Calls>) -> Self {
        todo!()
    }
    fn make_default(params: StackFrameMakeParams<'_, Self::Calls>) -> Self {
        todo!()
    }
    fn on_if<F: FnOnce(&mut Self::W) -> Self, F2: FnOnce(&mut Self::W) -> Self>(
        b: Self::Bool,
        param: BranchParam<Self, Self::W, F, F2>,
        w: &mut Self::W,
    ) -> Self {
        let BranchParam { on_true, on_false } = param;

        match b {
            Boolean::True => on_true.eval(w),
            Boolean::False => on_false.eval(w),
        }
    }
}

impl StackInterface for CallStack {
    type Elt = StackFrame;
    type W = ();
    type Bool = Boolean;

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
    type Bool = Boolean;

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

impl BoolInterface for Boolean {
    type W = ();
    type FailureStatusTable = ();

    fn as_boolean(&self) -> Boolean {
        *self
    }
    fn of_boolean(b: Boolean) -> Self {
        b
    }
    fn true_() -> Self {
        Boolean::True
    }
    fn false_() -> Self {
        Boolean::False
    }
    fn neg(&self) -> Self {
        self.neg()
    }
    fn or(a: Self, b: Self, w: &mut Self::W) -> Self {
        todo!()
    }
    fn and(a: Self, b: Self, w: &mut Self::W) -> Self {
        todo!()
    }
    fn equal(a: Self, b: Self, w: &mut Self::W) -> Self {
        todo!()
    }
    fn all(bs: &[Self], w: &mut Self::W) -> Self {
        todo!()
    }
    fn assert_any(bs: &[Self], w: &mut Self::W) {
        todo!()
    }
    fn assert_with_failure_status_tbl(
        b: Self,
        table: &Self::FailureStatusTable,
    ) -> Result<(), String> {
        todo!()
    }
}

impl AccountUpdateInterface for AccountUpdate {
    type W = ();
    type SingleData = ();
    type CallForest = CallForest<AccountUpdate>;
    type Bool = Boolean;
    type SignedAmount = Signed<Amount>;

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
    fn check_authorization(
        &self,
        will_succeed: Boolean,
        commitment: Fp,
        calls: &Self::CallForest,
        single_data: &Self::SingleData,
        w: &mut Self::W,
    ) -> CheckAuthorizationResult<Boolean> {
        todo!()
    }
    fn increment_nonce(&self) -> Self::Bool {
        todo!()
    }
    fn use_full_commitment(&self) -> Self::Bool {
        todo!()
    }
    fn account_precondition_nonce_is_constant(&self, w: &mut Self::W) -> Self::Bool {
        todo!()
    }
    fn implicit_account_creation_fee(&self) -> Self::Bool {
        todo!()
    }
    fn balance_change(&self) -> Self::SignedAmount {
        todo!()
    }
}

struct NonSnarkBranch;

impl BranchInterface for NonSnarkBranch {
    type W = ();

    fn make<T, F>(_w: &mut Self::W, run: F) -> BranchEvaluation<T, Self::W, F>
    where
        F: FnOnce(&mut Self::W) -> T,
    {
        // We don't run the closure now.
        // The closure will be run when `BranchEvaluation::eval` is called.
        BranchEvaluation::Pending(run)
    }
}
