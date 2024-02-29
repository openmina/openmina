use std::{fmt::Write, marker::PhantomData};

use ark_ff::Zero;
use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

use crate::{
    checked_equal_compressed_key, checked_equal_compressed_key_const_and,
    proofs::{
        field::{field, Boolean, CircuitVar, FieldWitness, ToBoolean},
        numbers::{
            common::ForZkappCheck,
            currency::{CheckedAmount, CheckedBalance, CheckedCurrency, CheckedSigned},
            nat::{CheckedIndex, CheckedNat, CheckedSlot},
        },
        to_field_elements::ToFieldElements,
        transaction::{
            create_shifted_inner_curve, decompress_var,
            transaction_snark::{check_timing, checked_chunked_signature_verify, checked_hash},
            Check, InnerCurve,
        },
        witness::Witness,
        zkapp::{GlobalStateForProof, LedgerWithHash, WithStackHash, ZkappSingleData},
    },
    scan_state::{
        currency::{Amount, SlotSpan, TxnVersion},
        transaction_logic::{
            local_state::{StackFrame, StackFrameChecked, StackFrameCheckedFrame, WithLazyHash},
            zkapp_command::{
                self, AccountUpdate, AccountUpdateSkeleton, CallForest, CheckAuthorizationResult,
                ClosedInterval, OrIgnore, SetOrKeep, Tree, WithHash,
                ACCOUNT_UPDATE_CONS_HASH_PARAM,
            },
            zkapp_statement::ZkappStatement,
            TransactionFailure,
        },
    },
    sparse_ledger::SparseLedger,
    zkapps::zkapp_logic,
    Account, AccountId, AuthRequired, AuthRequiredEncoded, Inputs, MyCow, ReceiptChainHash,
    ToInputs, TokenId, VerificationKey, ZkAppAccount, TXN_VERSION_CURRENT,
};

use super::intefaces::{
    AccountIdInterface, AccountInterface, AccountUpdateInterface, ActionsInterface,
    AmountInterface, BalanceInterface, BoolInterface, BranchEvaluation, BranchInterface,
    BranchParam, CallForestInterface, CallStackInterface, ControllerInterface,
    GlobalSlotSinceGenesisInterface, GlobalSlotSpanInterface, GlobalStateInterface, IndexInterface,
    LedgerInterface, LocalStateInterface, Opt, ReceiptChainHashInterface, SetOrKeepInterface,
    SignedAmountBranchParam, SignedAmountInterface, StackFrameInterface, StackFrameMakeParams,
    StackInterface, TokenIdInterface, TransactionCommitmentInterface, TxnVersionInterface,
    VerificationKeyHashInterface, WitnessGenerator, ZkappApplication, ZkappHandler,
};

pub struct ZkappSnark;

impl ZkappApplication for ZkappSnark {
    type Ledger = LedgerWithHash;
    type SignedAmount = CheckedSigned<Fp, CheckedAmount<Fp>>;
    type Amount = SnarkAmount;
    type Balance = SnarkBalance;
    type Index = CheckedIndex<Fp>;
    type GlobalSlotSinceGenesis = CheckedSlot<Fp>;
    type StackFrame = StackFrameChecked;
    type CallForest = WithHash<CallForest<AccountUpdate>>;
    type CallStack = WithHash<Vec<WithStackHash<WithHash<StackFrame>>>>;
    type GlobalState = GlobalStateForProof;
    type AccountUpdate =
        AccountUpdateSkeleton<WithHash<crate::scan_state::transaction_logic::zkapp_command::Body>>;
    type AccountId = SnarkAccountId;
    type TokenId = SnarkTokenId;
    type Bool = CircuitVar<Boolean>;
    type TransactionCommitment = SnarkTransactionCommitment;
    type FailureStatusTable = ();
    type LocalState = zkapp_logic::LocalState<Self>;
    type Account = SnarkAccount;
    type VerificationKeyHash = SnarkVerificationKeyHash;
    type SingleData = ZkappSingleData;
    type Controller = SnarkController;
    type TxnVersion = SnarkTxnVersion;
    type SetOrKeep = SnarkSetOrKeep;
    type GlobalSlotSpan = SnarkGlobalSlotSpan;
    type Actions = SnarkActions;
    type ReceiptChainHash = SnarkReceiptChainHash;
    type Handler = super::snark::SnarkHandler;
    type Branch = SnarkBranch;
    type WitnessGenerator = Witness<Fp>;
}

pub mod zkapp_check {
    use super::*;

    pub trait InSnarkCheck {
        type T;

        fn checked_zcheck(&self, x: &Self::T, w: &mut Witness<Fp>) -> Boolean;
    }

    impl<T> OrIgnore<T> {
        fn make_zcheck<F, F2>(&self, default_fn: F, compare_fun: F2, w: &mut Witness<Fp>) -> Boolean
        where
            F: Fn() -> T,
            F2: Fn(&T, &mut Witness<Fp>) -> Boolean,
        {
            let (is_some, value) = match self {
                OrIgnore::Check(v) => (Boolean::True, MyCow::Borrow(v)),
                OrIgnore::Ignore => (Boolean::False, MyCow::Own(default_fn())),
            };
            let is_good = compare_fun(&*value, w);
            Boolean::any(&[is_some.neg(), is_good], w)
        }
    }

    impl<Fun> InSnarkCheck for (&OrIgnore<Boolean>, Fun)
    where
        Fun: Fn() -> Boolean,
    {
        type T = Boolean;

        fn checked_zcheck(&self, x: &Self::T, w: &mut Witness<Fp>) -> Boolean {
            let (this, default_fn) = self;
            let compare = |value: &Self::T, w: &mut Witness<Fp>| Boolean::equal(x, value, w);
            this.make_zcheck(default_fn, compare, w)
        }
    }

    impl<Fun> InSnarkCheck for (&OrIgnore<Fp>, Fun)
    where
        Fun: Fn() -> Fp,
    {
        type T = Fp;

        fn checked_zcheck(&self, x: &Self::T, w: &mut Witness<Fp>) -> Boolean {
            let (this, default_fn) = self;
            let compare = |value: &Self::T, w: &mut Witness<Fp>| field::equal(*x, *value, w);
            this.make_zcheck(default_fn, compare, w)
        }
    }

    impl<Fun> InSnarkCheck for (&OrIgnore<CompressedPubKey>, Fun)
    where
        Fun: Fn() -> CompressedPubKey,
    {
        type T = CompressedPubKey;

        fn checked_zcheck(&self, x: &Self::T, w: &mut Witness<Fp>) -> Boolean {
            let (this, default_fn) = self;
            let compare = |value: &Self::T, w: &mut Witness<Fp>| {
                checked_equal_compressed_key(x, value, w)
                // checked_equal_compressed_key_const_and(x, value, w)
            };
            this.make_zcheck(default_fn, compare, w)
        }
    }

    impl<T, Fun> InSnarkCheck for (&OrIgnore<ClosedInterval<T>>, Fun)
    where
        Fun: Fn() -> ClosedInterval<T>,
        T: ForZkappCheck<Fp>,
    {
        type T = T;

        fn checked_zcheck(&self, x: &Self::T, w: &mut Witness<Fp>) -> Boolean {
            let (this, default_fn) = self;
            let compare = |value: &ClosedInterval<T>, w: &mut Witness<Fp>| {
                let ClosedInterval { lower, upper } = value;
                let lower = lower.to_checked();
                let upper = upper.to_checked();
                let x = x.to_checked();
                // We decompose this way because of OCaml evaluation order
                let lower_than_upper = <T as ForZkappCheck<Fp>>::lte(&x, &upper, w);
                let greater_than_lower = <T as ForZkappCheck<Fp>>::lte(&lower, &x, w);
                Boolean::all(&[greater_than_lower, lower_than_upper], w)
            };
            this.make_zcheck(default_fn, compare, w)
        }
    }
}

impl<F: FieldWitness> WitnessGenerator<F> for Witness<F> {
    type Bool = SnarkBool;

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
    fn exists_no_check_on_bool<T>(&mut self, b: Self::Bool, data: T) -> T
    where
        T: ToFieldElements<F>,
    {
        match b {
            CircuitVar::Var(_) => self.exists_no_check(data),
            CircuitVar::Constant(_) => data,
        }
    }
}

pub struct SnarkHandler;

impl ZkappHandler for SnarkHandler {
    type Z = ZkappSnark;
    type AccountUpdate = SnarkAccountUpdate;
    type Account = SnarkAccount;
    type Bool = SnarkBool;
    type W = Witness<Fp>;
    type GlobalState = GlobalStateForProof;

    fn check_account_precondition(
        account_update: &Self::AccountUpdate,
        account: &Self::Account,
        new_account: Self::Bool,
        local_state: &mut zkapp_logic::LocalState<ZkappSnark>,
        w: &mut Self::W,
    ) {
        let check = |failure: TransactionFailure, b: Boolean, w: &mut Witness<Fp>| {
            zkapp_logic::LocalState::<ZkappSnark>::add_check(local_state, failure, b.var(), w);
        };
        account_update.body.preconditions.account.checked_zcheck(
            new_account.as_boolean(),
            &*account.data,
            check,
            w,
        );
    }

    fn check_protocol_state_precondition(
        protocol_state_predicate: &zkapp_command::ZkAppPreconditions,
        global_state: &mut Self::GlobalState,
        w: &mut Self::W,
    ) -> Self::Bool {
        protocol_state_predicate
            .checked_zcheck(&global_state.protocol_state, w)
            .var()
    }

    fn check_valid_while_precondition(
        valid_while: &zkapp_command::Numeric<crate::scan_state::currency::Slot>,
        global_state: &mut Self::GlobalState,
        w: &mut Self::W,
    ) -> Self::Bool {
        use zkapp_check::InSnarkCheck;

        (valid_while, ClosedInterval::min_max)
            .checked_zcheck(&global_state.block_global_slot.to_inner(), w)
            .var()
    }

    fn init_account(
        account_update: &Self::AccountUpdate,
        account: &Self::Account,
    ) -> Self::Account {
        let AccountUpdateSkeleton {
            body: account_update,
            authorization: _,
        } = account_update;
        let account = Box::new(crate::Account {
            public_key: account_update.data.public_key.clone(),
            token_id: account_update.data.token_id.clone(),
            ..(*account.data).clone()
        });
        let account2 = account.clone();
        let account = WithLazyHash::new(account, move |w: &mut Witness<Fp>| {
            let zkapp = MyCow::borrow_or_default(&account2.zkapp);
            zkapp.checked_hash_with_param(ZkAppAccount::HASH_PARAM, w);
            account2.checked_hash(w)
        });
        account
    }
}

impl SignedAmountInterface for CheckedSigned<Fp, CheckedAmount<Fp>> {
    type W = Witness<Fp>;
    type Bool = SnarkBool;
    type Amount = SnarkAmount;

    fn zero() -> Self {
        CheckedSigned::zero()
    }
    fn is_neg(&self) -> Self::Bool {
        CheckedSigned::is_neg(self).var()
    }
    fn equal(&self, other: &Self, w: &mut Self::W) -> Self::Bool {
        CheckedSigned::const_equal(self, other, w).var()
    }
    fn is_non_neg(&self) -> Self::Bool {
        CheckedSigned::is_pos(self).var()
    }
    fn negate(&self) -> Self {
        CheckedSigned::negate(self.clone())
    }
    fn add_flagged(&self, other: &Self, w: &mut Self::W) -> (Self, Self::Bool) {
        let (value, is_overflow) = CheckedSigned::add_flagged(self, other, w);
        (value, is_overflow.var())
    }
    fn of_unsigned(unsigned: Self::Amount) -> Self {
        Self::of_unsigned(unsigned)
    }
    fn on_if<'a>(
        b: Self::Bool,
        param: SignedAmountBranchParam<&'a Self>,
        w: &mut Self::W,
    ) -> &'a Self {
        let SignedAmountBranchParam { on_true, on_false } = param;

        let amount = w.exists_no_check(match b.as_boolean() {
            Boolean::True => on_true,
            Boolean::False => on_false,
        });
        if on_true.try_get_value().is_some() && on_false.try_get_value().is_some() {
            w.exists_no_check(amount.force_value());
        }
        amount
    }
}

impl AmountInterface for SnarkAmount {
    type W = Witness<Fp>;
    type Bool = SnarkBool;

    fn zero() -> Self {
        <CheckedAmount<_> as CheckedCurrency<Fp>>::zero()
    }
    fn of_constant_fee(fee: crate::scan_state::currency::Fee) -> Self {
        Amount::of_fee(&fee).to_checked()
    }
}

type SnarkAccountUpdate =
    AccountUpdateSkeleton<WithHash<crate::scan_state::transaction_logic::zkapp_command::Body>>;
type SnarkCallForest = WithHash<CallForest<AccountUpdate>>;

impl CallForestInterface for SnarkCallForest {
    type W = Witness<Fp>;
    type AccountUpdate = SnarkAccountUpdate;
    type Bool = SnarkBool;

    fn empty() -> Self {
        WithHash {
            data: CallForest::empty(),
            hash: Fp::zero(),
        }
    }
    fn is_empty(&self, w: &mut Self::W) -> Self::Bool {
        let Self { hash, data: _ } = self;
        let empty = Fp::zero();
        field::equal(empty, *hash, w).var()
    }
    fn pop_exn(&self, w: &mut Self::W) -> ((Self::AccountUpdate, Self), Self) {
        let Self { data, hash: _ } = self;
        let hd_r = &data.first().unwrap().elt;
        let account_update = &hd_r.account_update;
        let auth = &account_update.authorization;
        let account_update = w.exists(&account_update.body);
        let account_update = {
            let hash = account_update.checked_hash_with_param(AccountUpdate::HASH_PARAM, w);
            WithHash {
                data: account_update.clone(),
                hash,
            }
        };
        let subforest = {
            let subforest = &hd_r.calls;
            let subforest_hash = w.exists(subforest.hash());
            WithHash {
                data: subforest.clone(),
                hash: subforest_hash,
            }
        };
        let tl_hash = w.exists(match data.tail().unwrap() {
            [] => Fp::zero(),
            [x, ..] => x.stack_hash,
        });
        let tree_hash = [account_update.hash, subforest.hash]
            .checked_hash_with_param(Tree::<AccountUpdate>::HASH_PARAM, w);
        let _hash_cons =
            [tree_hash, tl_hash].checked_hash_with_param(ACCOUNT_UPDATE_CONS_HASH_PARAM, w);
        let account = Self::AccountUpdate {
            body: account_update,
            authorization: auth.clone(),
        };

        let popped: (Self::AccountUpdate, Self) = (account, subforest);
        let tail: Self = WithHash {
            data: CallForest(data.tail().unwrap().to_vec()),
            hash: tl_hash,
        };

        (popped, tail)
    }
}

impl StackFrameInterface for StackFrameChecked {
    type Calls = SnarkCallForest;
    type W = Witness<Fp>;
    type Bool = SnarkBool;

    fn caller(&self) -> crate::TokenId {
        let Self {
            data:
                StackFrameCheckedFrame {
                    caller,
                    caller_caller: _,
                    calls: _,
                    is_default: _,
                },
            ..
        } = self;
        caller.clone()
    }
    fn caller_caller(&self) -> crate::TokenId {
        let Self {
            data:
                StackFrameCheckedFrame {
                    caller: _,
                    caller_caller,
                    calls: _,
                    is_default: _,
                },
            ..
        } = self;
        caller_caller.clone()
    }
    fn calls(&self) -> &Self::Calls {
        &self.calls
    }
    fn make(params: StackFrameMakeParams<'_, Self::Calls>) -> Self {
        let StackFrameMakeParams {
            caller,
            caller_caller,
            calls,
        } = params;

        let frame = StackFrameCheckedFrame {
            caller,
            caller_caller,
            calls: calls.clone(),
            is_default: false,
        };
        Self::of_frame(frame)
    }
    fn make_default(params: StackFrameMakeParams<'_, Self::Calls>) -> Self {
        let StackFrameMakeParams {
            caller,
            caller_caller,
            calls,
        } = params;

        let frame = StackFrameCheckedFrame {
            caller,
            caller_caller,
            calls: calls.clone(),
            is_default: true,
        };
        Self::of_frame(frame)
    }
    fn on_if<F: FnOnce(&mut Self::W) -> Self, F2: FnOnce(&mut Self::W) -> Self>(
        b: Self::Bool,
        param: BranchParam<Self, Self::W, F, F2>,
        w: &mut Self::W,
    ) -> Self {
        let BranchParam { on_true, on_false } = param;
        let on_true = on_true.eval(w);
        let on_false = on_false.eval(w);

        let data = match b.as_boolean() {
            Boolean::True => on_true.data.clone(),
            Boolean::False => on_false.data.clone(),
        };
        {
            let frame: &StackFrameCheckedFrame = &data;
            w.exists_no_check(frame);
        }
        WithLazyHash::new(data, move |w: &mut Witness<Fp>| {
            let on_false = on_false.hash(w);
            let on_true = on_true.hash(w);
            w.exists_no_check(match b.as_boolean() {
                Boolean::True => on_true,
                Boolean::False => on_false,
            })
        })
    }
}

/// Call_stack_digest.Checked.cons
fn call_stack_digest_checked_cons(h: Fp, t: Fp, w: &mut Witness<Fp>) -> Fp {
    checked_hash("MinaActUpStckFrmCons", &[h, t], w)
}

impl StackInterface for WithHash<Vec<WithStackHash<WithHash<StackFrame>>>> {
    type Elt = StackFrameChecked;
    type W = Witness<Fp>;
    type Bool = SnarkBool;

    fn empty() -> Self {
        WithHash {
            data: Vec::new(),
            hash: Fp::zero(),
        }
    }
    fn is_empty(&self, w: &mut Self::W) -> Self::Bool {
        let Self { hash, data: _ } = self;
        let empty = Fp::zero();
        field::equal(empty, *hash, w).var()
    }
    fn pop(&self, w: &mut Self::W) -> Opt<(Self::Elt, Self)> {
        let Self { data, hash } = self;
        let input_is_empty = self.is_empty(w);
        let hd_r = match data.first() {
            None => {
                let data = StackFrame::default();
                let hash = data.hash();
                MyCow::Own(WithHash { data, hash })
            }
            Some(x) => MyCow::Borrow(&x.elt),
        };
        let tl_r = data.get(1..).unwrap_or(&[]);
        let elt = hd_r.exists_elt(w);
        let stack = w.exists(match tl_r {
            [] => Fp::zero(),
            [x, ..] => x.stack_hash,
        });
        let stack_frame_hash = elt.hash(w);
        let h2 = call_stack_digest_checked_cons(stack_frame_hash, stack, w);
        let is_equal = field::equal(*hash, h2, w);
        Boolean::assert_any(&[input_is_empty.as_boolean(), is_equal], w);
        Opt {
            is_some: input_is_empty.neg().as_boolean(),
            data: (
                elt,
                Self {
                    data: tl_r.to_vec(),
                    hash: stack,
                },
            ),
        }
    }
    fn push(elt: Self::Elt, onto: Self, w: &mut Self::W) -> Self {
        let Self {
            data: r_tl,
            hash: h_tl,
        } = onto;

        let h = call_stack_digest_checked_cons(elt.hash(w), h_tl, w);

        let r = {
            let hd = {
                let frame = elt;
                let hash = frame.hash(w);
                let data = StackFrame {
                    caller: frame.data.caller,
                    caller_caller: frame.data.caller_caller,
                    calls: frame.data.calls.data,
                };
                WithHash { data, hash }
            };
            let tl = r_tl;

            [WithStackHash {
                elt: hd,
                stack_hash: h,
            }]
            .into_iter()
            .chain(tl)
            .collect::<Vec<_>>()
        };

        Self { data: r, hash: h }
    }
}
impl CallStackInterface for WithHash<Vec<WithStackHash<WithHash<StackFrame>>>> {
    type StackFrame = StackFrameChecked;
}

impl GlobalStateInterface for GlobalStateForProof {
    type Ledger = LedgerWithHash;
    type W = Witness<Fp>;
    type Bool = SnarkBool;
    type SignedAmount = CheckedSigned<Fp, CheckedAmount<Fp>>;
    type GlobalSlotSinceGenesis = SnarkGlobalSlot;

    fn first_pass_ledger(&self) -> Self::Ledger {
        self.first_pass_ledger.clone()
    }
    fn set_first_pass_ledger(
        &mut self,
        should_update: Self::Bool,
        ledger: &Self::Ledger,
        w: &mut Self::W,
    ) {
        let ledger = match should_update.as_boolean() {
            Boolean::True => ledger.clone(),
            Boolean::False => self.first_pass_ledger.clone(),
        };
        w.exists_no_check_on_bool(should_update, ledger.hash);
        self.first_pass_ledger = ledger;
    }
    fn second_pass_ledger(&self) -> Self::Ledger {
        self.second_pass_ledger.clone()
    }
    fn set_second_pass_ledger(
        &mut self,
        should_update: Self::Bool,
        ledger: &Self::Ledger,
        w: &mut Self::W,
    ) {
        let ledger = match should_update.as_boolean() {
            Boolean::True => ledger.clone(),
            Boolean::False => self.second_pass_ledger.clone(),
        };
        w.exists_no_check(ledger.hash);
        self.second_pass_ledger = ledger;
    }
    fn fee_excess(&self) -> Self::SignedAmount {
        self.fee_excess.clone()
    }
    fn supply_increase(&self) -> Self::SignedAmount {
        self.supply_increase.clone()
    }
    fn set_fee_excess(&mut self, fee_excess: Self::SignedAmount) {
        self.fee_excess = fee_excess;
    }
    fn set_supply_increase(&mut self, supply_increase: Self::SignedAmount) {
        self.supply_increase = supply_increase;
    }
    fn block_global_slot(&self) -> Self::GlobalSlotSinceGenesis {
        self.block_global_slot.clone()
    }
}

pub type SnarkIndex = CheckedIndex<Fp>;

impl IndexInterface for SnarkIndex {
    fn zero() -> Self {
        <CheckedIndex<Fp> as crate::proofs::numbers::nat::CheckedNat<_, 32>>::zero()
    }
    fn succ(&self) -> Self {
        <CheckedIndex<Fp> as crate::proofs::numbers::nat::CheckedNat<_, 32>>::succ(self)
    }
}

impl GlobalSlotSinceGenesisInterface for SnarkGlobalSlot {
    type W = Witness<Fp>;
    type Bool = SnarkBool;

    fn equal(&self, other: &Self, w: &mut Self::W) -> Self::Bool {
        <Self as CheckedNat<_, 32>>::equal(&self, other, w).var()
    }
}

fn signature_verifies(
    shifted: &InnerCurve<Fp>,
    payload_digest: Fp,
    signature: &mina_signer::Signature,
    pk: &CompressedPubKey,
    w: &mut Witness<Fp>,
) -> Boolean {
    let pk = decompress_var(pk, w);

    let mut inputs = Inputs::new();
    inputs.append_field(payload_digest);

    checked_chunked_signature_verify(shifted, &pk, signature, inputs, w)
}

impl AccountUpdateInterface for SnarkAccountUpdate {
    type W = Witness<Fp>;
    type CallForest = SnarkCallForest;
    type SingleData = ZkappSingleData;
    type Bool = SnarkBool;
    type SignedAmount = SnarkSignedAmount;

    fn body(&self) -> &crate::scan_state::transaction_logic::zkapp_command::Body {
        let Self {
            body,
            authorization: _,
        } = self;
        let WithHash { data, hash: _ } = body;
        data
    }
    fn set(&mut self, new: Self) {
        *self = new;
    }
    fn verification_key_hash(&self) -> Fp {
        self.body().authorization_kind.vk_hash()
    }
    fn is_proved(&self) -> Self::Bool {
        self.body()
            .authorization_kind
            .is_proved()
            .to_boolean()
            .var()
    }
    fn is_signed(&self) -> Self::Bool {
        self.body()
            .authorization_kind
            .is_signed()
            .to_boolean()
            .var()
    }
    fn check_authorization(
        &self,
        will_succeed: Self::Bool,
        commitment: Fp,
        calls: &Self::CallForest,
        single_data: &Self::SingleData,
        w: &mut Self::W,
    ) -> CheckAuthorizationResult<Self::Bool> {
        use crate::scan_state::transaction_logic::zkapp_statement::TransactionCommitment;
        use crate::ControlTag::{NoneGiven, Proof, Signature};

        let Self::CallForest {
            data: _,
            hash: calls,
        } = calls;
        let Self {
            body: account_update,
            authorization: control,
        } = self;

        let auth_type = single_data.spec().auth_type;
        let proof_verifies = match auth_type {
            Proof => {
                let stmt = ZkappStatement {
                    account_update: TransactionCommitment(account_update.hash),
                    calls: TransactionCommitment(*calls),
                };
                single_data.set_zkapp_input(stmt);
                single_data.set_must_verify(will_succeed.as_boolean());
                Boolean::True.constant()
            }
            Signature | NoneGiven => Boolean::False.constant(),
        };
        let signature_verifies = match auth_type {
            NoneGiven | Proof => Boolean::False.constant(),
            Signature => {
                use crate::scan_state::transaction_logic::zkapp_command::Control;
                let signature = w.exists({
                    match control {
                        Control::Signature(s) => MyCow::Borrow(s),
                        Control::NoneGiven => MyCow::Own(mina_signer::Signature::dummy()),
                        Control::Proof(_) => unreachable!(),
                    }
                });
                let payload_digest = commitment;
                let shifted = create_shifted_inner_curve(w);
                signature_verifies(
                    &shifted,
                    payload_digest,
                    &signature,
                    &account_update.public_key,
                    w,
                )
                .var()
            }
        };
        CheckAuthorizationResult {
            proof_verifies,
            signature_verifies,
        }
    }
    fn increment_nonce(&self) -> Self::Bool {
        self.body().increment_nonce.to_boolean().var()
    }
    fn use_full_commitment(&self) -> Self::Bool {
        self.body().use_full_commitment.to_boolean().var()
    }
    fn account_precondition_nonce_is_constant(&self, w: &mut Self::W) -> Self::Bool {
        let nonce = self.body().preconditions.account.nonce();
        let (is_check, ClosedInterval { lower, upper }) = match nonce {
            OrIgnore::Check(interval) => (Boolean::True, interval.clone()),
            OrIgnore::Ignore => (Boolean::False, ClosedInterval::min_max()),
        };
        let is_constant = lower.to_checked().equal(&upper.to_checked(), w);
        is_check.and(&is_constant, w).var()
    }
    fn implicit_account_creation_fee(&self) -> Self::Bool {
        self.body().implicit_account_creation_fee.to_boolean().var()
    }
    fn balance_change(&self) -> Self::SignedAmount {
        self.body().balance_change.to_checked()
    }
}

impl LocalStateInterface for zkapp_logic::LocalState<ZkappSnark> {
    type Z = ZkappSnark;
    type Bool = SnarkBool;
    type W = Witness<Fp>;

    fn add_check(
        local: &mut zkapp_logic::LocalState<Self::Z>,
        _failure: TransactionFailure,
        b: Self::Bool,
        w: &mut Self::W,
    ) {
        local.success = local.success.and(&b, w);
    }

    fn add_new_failure_status_bucket(_local: &mut zkapp_logic::LocalState<Self::Z>) {
        // nothing
    }
}

pub enum FlaggedOption<T> {
    Some(T),
    None,
}

impl<T> From<Option<T>> for FlaggedOption<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => Self::Some(v),
            None => Self::None,
        }
    }
}

impl<T, F> Check<Fp> for (FlaggedOption<&T>, F)
where
    T: Check<Fp>,
    F: Fn() -> T,
{
    fn check(&self, w: &mut Witness<Fp>) {
        let (or_ignore, default_fn) = self;
        let value = match or_ignore {
            FlaggedOption::Some(this) => MyCow::Borrow(*this),
            FlaggedOption::None => MyCow::Own(default_fn()),
        };
        value.check(w);
    }
}

impl<T, F> ToFieldElements<Fp> for (FlaggedOption<&T>, F)
where
    T: ToFieldElements<Fp>,
    F: Fn() -> T,
{
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let (or_ignore, default_fn) = self;

        match or_ignore {
            FlaggedOption::Some(this) => {
                Boolean::True.to_field_elements(fields);
                this.to_field_elements(fields);
            }
            FlaggedOption::None => {
                Boolean::False.to_field_elements(fields);
                let default = default_fn();
                default.to_field_elements(fields);
            }
        };
    }
}

// dummy_vk_hash

pub struct AccountUnhashed(pub Box<Account>);

impl ToFieldElements<Fp> for AccountUnhashed {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        use crate::VotingFor;

        let Account {
            public_key,
            token_id: TokenId(token_id),
            token_symbol,
            balance,
            nonce,
            receipt_chain_hash: ReceiptChainHash(receipt_chain_hash),
            delegate,
            voting_for: VotingFor(voting_for),
            timing,
            permissions,
            zkapp,
        } = &*self.0;

        // Important: Any changes here probably needs the same changes in `Account`
        public_key.to_field_elements(fields);
        token_id.to_field_elements(fields);
        token_symbol.to_field_elements(fields);
        balance.to_field_elements(fields);
        nonce.to_field_elements(fields);
        receipt_chain_hash.to_field_elements(fields);
        let delegate = MyCow::borrow_or_else(delegate, CompressedPubKey::empty);
        delegate.to_field_elements(fields);
        voting_for.to_field_elements(fields);
        timing.to_field_elements(fields);
        permissions.to_field_elements(fields);
        MyCow::borrow_or_else(zkapp, crate::ZkAppAccount::default).to_field_elements(fields);
    }
}

impl Check<Fp> for AccountUnhashed {
    fn check(&self, w: &mut Witness<Fp>) {
        let Account {
            public_key: _,
            token_id: _,
            token_symbol,
            balance,
            nonce,
            receipt_chain_hash: _,
            delegate: _,
            voting_for: _,
            timing,
            permissions,
            zkapp,
        } = &*self.0;

        token_symbol.check(w);
        balance.check(w);
        nonce.check(w);
        timing.check(w);
        permissions.check(w);
        (
            FlaggedOption::from(zkapp.as_ref()),
            crate::ZkAppAccount::default,
        )
            .check(w);
    }
}

pub type SnarkAccount = WithLazyHash<Box<Account>>;

impl AccountInterface for SnarkAccount {
    type W = Witness<Fp>;
    type D = ZkappSingleData;
    type Bool = SnarkBool;
    type Balance = SnarkBalance;
    type GlobalSlot = SnarkGlobalSlot;

    fn register_verification_key(&self, data: &Self::D, w: &mut Self::W) {
        use crate::ControlTag::*;

        match data.spec().auth_type {
            Proof => {
                let vk = self
                    .zkapp
                    .as_ref()
                    .unwrap()
                    .verification_key
                    .as_ref()
                    .unwrap();
                let vk = w.exists(vk);
                vk.checked_hash_with_param(VerificationKey::HASH_PARAM, w);
            }
            Signature | NoneGiven => {}
        }
    }
    fn get(&self) -> &crate::Account {
        let Self { data, .. } = self;
        &*data
    }
    fn get_mut(&mut self) -> &mut crate::Account {
        let Self { data, .. } = self;
        &mut *data
    }
    fn set_delegate(&mut self, new: CompressedPubKey) {
        let Self { data: account, .. } = self;
        account.delegate = if new == CompressedPubKey::empty() {
            None
        } else {
            Some(new)
        };
    }
    fn zkapp(&self) -> MyCow<ZkAppAccount> {
        match &self.zkapp {
            Some(zkapp) => MyCow::Borrow(zkapp),
            None => MyCow::Own(ZkAppAccount::default()),
        }
    }
    fn zkapp_mut(&mut self) -> &mut ZkAppAccount {
        // `unwrap`: `make_zkapp` is supposed to be called before `zkapp_mut`
        self.data.zkapp.as_mut().unwrap()
    }
    fn verification_key_hash(&self) -> Fp {
        // TODO: We shouldn't compute the hash here
        let zkapp = self.zkapp();
        MyCow::borrow_or_else(&zkapp.verification_key, VerificationKey::dummy).hash()
    }
    fn set_token_id(&mut self, token_id: TokenId) {
        let Self { data: account, .. } = self;
        account.token_id = token_id;
    }
    fn is_timed(&self) -> Self::Bool {
        let Self { data: account, .. } = self;
        account.timing.is_timed().to_boolean().var()
    }
    fn balance(&self) -> Self::Balance {
        let Self { data: account, .. } = self;
        account.balance.to_checked()
    }
    fn set_balance(&mut self, balance: Self::Balance) {
        self.data.balance = balance.to_inner(); // TODO: Overflow ?
    }
    fn check_timing(
        &self,
        txn_global_slot: &Self::GlobalSlot,
        w: &mut Self::W,
    ) -> (Self::Bool, crate::Timing) {
        let mut invalid_timing = Option::<Boolean>::None;
        let timed_balance_check = |b: Boolean, _w: &mut Witness<Fp>| {
            invalid_timing = Some(b.neg());
        };
        let account = self.get();
        let (_min_balance, timing) = check_timing(
            account,
            None,
            txn_global_slot.clone(),
            timed_balance_check,
            w,
        );
        (invalid_timing.unwrap().var(), timing)
    }
    fn make_zkapp(&mut self) {
        if self.data.zkapp.is_none() {
            self.data.zkapp = Some(ZkAppAccount::default());
        }
    }
    fn unmake_zkapp(&mut self) {
        let Some(zkapp) = self.data.zkapp.as_mut() else {
            panic!("invalid state"); // `unmake_zkapp` must be called after `make_zkapp`
        };
        if zkapp == &ZkAppAccount::default() {
            self.data.zkapp = None;
        }
    }
    fn proved_state(&self) -> Self::Bool {
        let zkapp = self.zkapp.as_ref().unwrap(); // `make_zkapp` was already call
        zkapp.proved_state.to_boolean().var()
    }
    fn set_proved_state(&mut self, proved_state: Self::Bool) {
        let zkapp = self.data.zkapp.as_mut().unwrap(); // `make_zkapp` was already call
        zkapp.proved_state = proved_state.as_boolean().as_bool();
    }
    fn app_state(&self) -> [Fp; 8] {
        let zkapp = self.zkapp.as_ref().unwrap(); // `make_zkapp` was already call
        zkapp.app_state
    }
    fn last_action_slot(&self) -> Self::GlobalSlot {
        let zkapp = self.zkapp.as_ref().unwrap(); // `make_zkapp` was already call
        zkapp.last_action_slot.to_checked()
    }
    fn set_last_action_slot(&mut self, slot: Self::GlobalSlot) {
        let zkapp = self.data.zkapp.as_mut().unwrap(); // `make_zkapp` was already call
        zkapp.last_action_slot = slot.to_inner();
    }
}

fn implied_root(account: &SnarkAccount, incl: &[(Boolean, Fp)], w: &mut Witness<Fp>) -> Fp {
    let mut param = String::with_capacity(16);
    incl.iter()
        .enumerate()
        .fold(account.hash(w), |accum: Fp, (depth, (is_right, h))| {
            let hashes = match is_right {
                Boolean::False => [accum, *h],
                Boolean::True => [*h, accum],
            };
            param.clear();
            write!(&mut param, "MinaMklTree{:03}", depth).unwrap();
            w.exists(hashes);
            checked_hash(param.as_str(), &hashes, w)
        })
}

impl LedgerInterface for LedgerWithHash {
    type W = Witness<Fp>;
    type AccountUpdate = SnarkAccountUpdate;
    type Account = SnarkAccount;
    type Bool = SnarkBool;
    type InclusionProof = Vec<(Boolean, Fp)>;

    fn empty() -> Self {
        let mut ledger = <SparseLedger as crate::sparse_ledger::LedgerIntf>::empty(0);
        let hash = ledger.merkle_root();
        Self { ledger, hash }
    }
    fn get_account(
        &self,
        account_update: &Self::AccountUpdate,
        w: &mut Self::W,
    ) -> (Self::Account, Self::InclusionProof) {
        let Self {
            ledger,
            hash: _root,
        } = self;
        let idx = ledger.find_index_exn(account_update.body.account_id());
        let account = w.exists(AccountUnhashed(ledger.get_exn(&idx)));
        // TODO: Don't clone here
        let account2 = account.0.clone();
        let account = WithLazyHash::new(account.0, move |w: &mut Witness<Fp>| {
            let zkapp = MyCow::borrow_or_default(&account2.zkapp);
            zkapp.checked_hash_with_param(ZkAppAccount::HASH_PARAM, w);
            account2.checked_hash(w)
        });
        let inclusion = w.exists(
            ledger
                .clone()
                .path_exn(idx)
                .into_iter()
                .map(|path| match path {
                    crate::MerklePath::Left(h) => (Boolean::False, h),
                    crate::MerklePath::Right(h) => (Boolean::True, h),
                })
                .collect::<Vec<_>>(),
        );
        (account, inclusion)
    }
    fn set_account(&mut self, (a, incl): (Self::Account, Self::InclusionProof), w: &mut Self::W) {
        let Self { ledger, hash } = self;
        let new_hash = implied_root(&a, &incl, w);
        let idx = ledger.find_index_exn(a.id());
        ledger.set_exn(idx, a.data);
        *hash = new_hash;
    }
    fn check_inclusion(
        &self,
        (account, incl): &(Self::Account, Self::InclusionProof),
        w: &mut Self::W,
    ) {
        implied_root(account, incl, w);
    }
    fn check_account(
        public_key: &mina_signer::CompressedPubKey,
        token_id: &TokenId,
        account: (&Self::Account, &Self::InclusionProof),
        w: &mut Self::W,
    ) -> Self::Bool {
        let (WithLazyHash { data: account, .. }, _) = account;
        let is_new = checked_equal_compressed_key_const_and(
            &account.public_key,
            &CompressedPubKey::empty(),
            w,
        );
        let is_same = checked_equal_compressed_key(public_key, &account.public_key, w);
        Boolean::assert_any(&[is_new, is_same], w);
        let is_same_token = field::equal(token_id.0, account.token_id.0, w);
        Boolean::assert_any(&[is_new, is_same_token], w);
        is_new.var()
    }
    fn exists_no_check(self, w: &mut Self::W) -> Self {
        w.exists_no_check(self.hash);
        self
    }
    fn exists_no_check_on_bool(self, b: Self::Bool, w: &mut Self::W) -> Self {
        w.exists_no_check_on_bool(b, self.hash);
        self
    }
}

pub struct SnarkAccountId;
pub struct SnarkTokenId;
pub type SnarkBool = CircuitVar<Boolean>;
pub type SnarkAmount = CheckedAmount<Fp>;
pub type SnarkSignedAmount = CheckedSigned<Fp, CheckedAmount<Fp>>;
pub type SnarkBalance = CheckedBalance<Fp>;
pub struct SnarkTransactionCommitment;
pub struct SnarkVerificationKeyHash;
pub struct SnarkController;
pub struct SnarkTxnVersion;
pub struct SnarkSetOrKeep;
pub struct SnarkGlobalSlotSpan;
pub struct SnarkActions;
pub type SnarkGlobalSlot = CheckedSlot<Fp>;
pub struct SnarkReceiptChainHash;

impl AccountIdInterface for SnarkAccountId {
    type W = Witness<Fp>;

    fn derive_token_id(account_id: &AccountId, w: &mut Self::W) -> TokenId {
        TokenId(account_id.checked_hash_with_param(AccountId::DERIVE_TOKEN_ID_HASH_PARAM, w))
    }
}

impl TokenIdInterface for SnarkTokenId {
    type W = Witness<Fp>;
    type Bool = SnarkBool;

    fn equal(a: &TokenId, b: &TokenId, w: &mut Self::W) -> Self::Bool {
        field::equal(a.0, b.0, w).var()
    }
}

impl VerificationKeyHashInterface for SnarkVerificationKeyHash {
    type W = Witness<Fp>;
    type Bool = SnarkBool;

    fn equal(a: Fp, b: Fp, w: &mut Self::W) -> Self::Bool {
        field::equal(a, b, w).var()
    }
}

impl BoolInterface for SnarkBool {
    type W = Witness<Fp>;
    type FailureStatusTable = ();

    fn as_boolean(&self) -> Boolean {
        self.as_boolean()
    }
    fn of_boolean(b: Boolean) -> Self {
        CircuitVar::Var(b)
    }
    fn true_() -> Self {
        CircuitVar::Constant(Boolean::True)
    }
    fn false_() -> Self {
        CircuitVar::Constant(Boolean::False)
    }
    fn neg(&self) -> Self {
        self.neg()
    }
    fn or(a: Self, b: Self, w: &mut Self::W) -> Self {
        a.or(&b, w)
    }
    fn and(a: Self, b: Self, w: &mut Self::W) -> Self {
        a.and(&b, w)
    }
    fn equal(a: Self, b: Self, w: &mut Self::W) -> Self {
        a.equal_bool(&b, w)
    }
    fn all(bs: &[Self], w: &mut Self::W) -> Self {
        SnarkBool::all(bs, w)
    }
    fn assert_any(bs: &[Self], w: &mut Self::W) {
        SnarkBool::assert_any::<Fp>(bs, w);
    }
    fn assert_with_failure_status_tbl(
        _b: Self,
        _table: &Self::FailureStatusTable,
    ) -> Result<(), String> {
        Ok(())
    }
}

impl BalanceInterface for SnarkBalance {
    type W = Witness<Fp>;
    type Bool = SnarkBool;
    type Amount = SnarkAmount;
    type SignedAmount = SnarkSignedAmount;

    fn add_signed_amount_flagged(
        &self,
        signed_amount: Self::SignedAmount,
        w: &mut Self::W,
    ) -> (Self, Self::Bool) {
        let (balance, failed) = SnarkBalance::add_signed_amount_flagged(self, signed_amount, w);
        (balance, failed.var())
    }
}

impl TransactionCommitmentInterface for SnarkTransactionCommitment {
    type AccountUpdate = SnarkAccountUpdate;
    type CallForest = SnarkCallForest;
    type W = Witness<Fp>;

    fn empty() -> Fp {
        Fp::zero()
    }

    fn commitment(account_updates: &Self::CallForest) -> Fp {
        let Self::CallForest {
            data: _,
            hash: account_updates_hash,
        } = account_updates;
        *account_updates_hash
    }

    fn full_commitment(
        account_updates: &Self::AccountUpdate,
        memo_hash: Fp,
        commitment: Fp,
        w: &mut Self::W,
    ) -> Fp {
        let fee_payer_hash = account_updates.body.hash;

        [memo_hash, fee_payer_hash, commitment]
            .checked_hash_with_param(ACCOUNT_UPDATE_CONS_HASH_PARAM, w)
    }
}

fn encode_auth(auth: &AuthRequired) -> AuthRequiredEncoded<CircuitVar<Boolean>> {
    let AuthRequiredEncoded {
        constant,
        signature_necessary,
        signature_sufficient,
    } = auth.encode();

    AuthRequiredEncoded {
        constant: constant.to_boolean().var(),
        signature_necessary: signature_necessary.to_boolean().var(),
        signature_sufficient: signature_sufficient.to_boolean().var(),
    }
}

// TODO: Dedup with the one in `account.rs`
fn eval_no_proof(
    auth: &AuthRequired,
    signature_verifies: SnarkBool,
    w: &mut Witness<Fp>,
) -> SnarkBool {
    let AuthRequiredEncoded {
        constant,
        signature_necessary: _,
        signature_sufficient,
    } = encode_auth(auth);

    let a = constant.neg().and(&signature_verifies, w);
    let b = constant.or(&a, w);
    signature_sufficient.and(&b, w)
}

fn eval_proof(auth: &AuthRequired, w: &mut Witness<Fp>) -> SnarkBool {
    let AuthRequiredEncoded {
        constant,
        signature_necessary,
        signature_sufficient,
    } = encode_auth(auth);

    let impossible = constant.and(&signature_sufficient.neg(), w);
    signature_necessary.neg().and(&impossible.neg(), w)
}

fn verification_key_perm_fallback_to_signature_with_older_version(
    auth: &AuthRequired,
    w: &mut Witness<Fp>,
) -> AuthRequired {
    let AuthRequiredEncoded {
        signature_sufficient,
        ..
    } = encode_auth(auth);

    let on_true = SnarkBranch::make(w, |_| AuthRequired::Signature);
    let on_false = SnarkBranch::make(w, |_| auth.clone());

    w.on_if(
        signature_sufficient.neg(),
        BranchParam { on_true, on_false },
    )
}

impl ControllerInterface for SnarkController {
    type W = Witness<Fp>;
    type Bool = SnarkBool;
    type SingleData = ZkappSingleData;

    fn check(
        _proof_verifies: Self::Bool,
        signature_verifies: Self::Bool,
        auth: &AuthRequired,
        single_data: &Self::SingleData,
        w: &mut Self::W,
    ) -> Self::Bool {
        use crate::ControlTag::{NoneGiven, Proof, Signature};

        match single_data.spec().auth_type {
            Proof => eval_proof(auth, w),
            Signature | NoneGiven => eval_no_proof(auth, signature_verifies, w),
        }
    }

    fn verification_key_perm_fallback_to_signature_with_older_version(
        auth: &AuthRequired,
        w: &mut Self::W,
    ) -> AuthRequired {
        verification_key_perm_fallback_to_signature_with_older_version(auth, w)
    }
}

impl TxnVersionInterface for SnarkTxnVersion {
    type W = Witness<Fp>;
    type Bool = SnarkBool;

    fn equal_to_current(version: TxnVersion, w: &mut Self::W) -> Self::Bool {
        let current = TXN_VERSION_CURRENT.to_checked();
        let version = version.to_checked();
        version.equal(&current, w).var()
    }

    fn older_than_current(version: TxnVersion, w: &mut Self::W) -> Self::Bool {
        let current = TXN_VERSION_CURRENT.to_checked();
        let version = version.to_checked();
        version.less_than(&current, w).var()
    }
}

impl SetOrKeepInterface for SnarkSetOrKeep {
    type Bool = SnarkBool;

    fn is_keep<T: Clone>(set_or_keep: &SetOrKeep<T>) -> Self::Bool {
        match set_or_keep {
            SetOrKeep::Set(_) => CircuitVar::Var(Boolean::False),
            SetOrKeep::Keep => CircuitVar::Var(Boolean::True),
        }
    }
    fn is_set<T: Clone>(set_or_keep: &SetOrKeep<T>) -> Self::Bool {
        match set_or_keep {
            SetOrKeep::Set(_) => CircuitVar::Var(Boolean::True),
            SetOrKeep::Keep => CircuitVar::Var(Boolean::False),
        }
    }
}

impl GlobalSlotSpanInterface for SnarkGlobalSlotSpan {
    type W = Witness<Fp>;
    type Bool = SnarkBool;
    type SlotSpan = SlotSpan;

    fn greater_than(this: &Self::SlotSpan, other: &Self::SlotSpan, w: &mut Self::W) -> Self::Bool {
        let this = this.to_checked::<Fp>();
        let other = other.to_checked::<Fp>();

        this.const_greater_than(&other, w).var()
    }
}

impl ActionsInterface for SnarkActions {
    type W = Witness<Fp>;
    type Bool = SnarkBool;

    fn is_empty(actions: &zkapp_command::Actions, w: &mut Self::W) -> Self::Bool {
        use zkapp_command::MakeEvents;

        let hash = zkapp_command::events_to_field(actions);
        field::equal(hash, zkapp_command::Actions::empty_hash(), w).var()
    }

    fn push_events(event: Fp, actions: &zkapp_command::Actions, w: &mut Self::W) -> Fp {
        use zkapp_command::MakeEvents;

        let hash = zkapp_command::events_to_field(actions);
        checked_hash(zkapp_command::Actions::HASH_PREFIX, &[event, hash], w)
    }
}

impl ReceiptChainHashInterface for SnarkReceiptChainHash {
    type W = Witness<Fp>;
    type Index = SnarkIndex;

    fn cons_zkapp_command_commitment(
        index: Self::Index,
        element: Fp,
        other: ReceiptChainHash,
        w: &mut Self::W,
    ) -> ReceiptChainHash {
        let mut inputs = Inputs::new();

        inputs.append(&index);
        inputs.append_field(element);
        inputs.append(&other);

        ReceiptChainHash(checked_hash("MinaReceiptUC", &inputs.to_fields(), w))
    }
}

pub struct SnarkBranch;

impl BranchInterface for SnarkBranch {
    type W = Witness<Fp>;

    fn make<T, F>(w: &mut Self::W, run: F) -> BranchEvaluation<T, Self::W, F>
    where
        F: FnOnce(&mut Self::W) -> T,
    {
        // We run the closure as soon as `SnarkBranch::make` is called
        BranchEvaluation::Evaluated(run(w), PhantomData)
    }
}
