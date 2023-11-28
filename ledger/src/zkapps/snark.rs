use std::fmt::Write;

use ark_ff::Zero;
use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

use crate::{
    checked_equal_compressed_key, checked_equal_compressed_key_const_and,
    proofs::{
        numbers::{
            common::ForZkappCheck,
            currency::{CheckedAmount, CheckedCurrency, CheckedSigned},
            nat::{CheckedIndex, CheckedNat, CheckedSlot},
        },
        to_field_elements::ToFieldElements,
        witness::{
            create_shifted_inner_curve, decompress_var, field,
            transaction_snark::{checked_hash, checked_signature_verify},
            Boolean, Check, FieldWitness, InnerCurve, ToBoolean, Witness,
        },
        zkapp::{GlobalStateForProof, LedgerWithHash, WithStackHash, ZkappSingleData},
        zkapp_logic,
    },
    scan_state::{
        currency::{Magnitude, MinMax},
        transaction_logic::{
            local_state::{StackFrame, StackFrameChecked, StackFrameCheckedFrame, WithLazyHash},
            zkapp_command::{
                AccountUpdate, AccountUpdateSkeleton, AuthorizationKind, CallForest,
                ClosedInterval, OrIgnore, Tree, WithHash, ACCOUNT_UPDATE_CONS_HASH_PARAM,
            },
            zkapp_statement::ZkappStatement,
            TransactionFailure,
        },
    },
    Account, AccountId, MyCow, ToInputs, TokenId, VerificationKey, ZkAppAccount,
};

use super::intefaces::{
    AccountIdInterface, AccountInterface, AccountUpdateInterface, AmountInterface, BoolInterface,
    CallForestInterface, CallStackInterface, GlobalSlotSinceGenesisInterface, GlobalStateInterface,
    IndexInterface, LedgerInterface, LocalStateInterface, Opt, SignedAmountInterface,
    StackFrameInterface, StackFrameMakeParams, StackInterface, TokenIdInterface,
    TransactionCommitmentInterface, VerificationKeyHashInterface, WitnessGenerator, ZkappSnark,
};

use super::intefaces::WitnessGenerator as W;

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

type SnarkAccountUpdate =
    AccountUpdateSkeleton<WithHash<crate::scan_state::transaction_logic::zkapp_command::Body>>;
type SnarkCallForest = WithHash<CallForest<AccountUpdate>>;

impl CallForestInterface for SnarkCallForest {
    type W = Witness<Fp>;
    type AccountUpdate = SnarkAccountUpdate;

    fn empty() -> Self {
        WithHash {
            data: CallForest::empty(),
            hash: Fp::zero(),
        }
    }
    fn is_empty(&self, w: &mut Self::W) -> Boolean {
        let Self { hash, data: _ } = self;
        let empty = Fp::zero();
        field::equal(empty, *hash, w)
    }
    fn pop_exn(&self, w: &mut Self::W) -> ((Self::AccountUpdate, Self), Self) {
        let Self { data, hash } = self;
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
        let hash_cons =
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

    fn caller(&self) -> crate::TokenId {
        let Self {
            data:
                StackFrameCheckedFrame {
                    caller,
                    caller_caller: _,
                    calls: _,
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
                },
            ..
        } = self;
        caller_caller.clone()
    }
    fn calls(&self) -> &Self::Calls {
        &self.calls
    }
    fn make(params: StackFrameMakeParams<'_, Self::Calls>, w: &mut Self::W) -> Self {
        let StackFrameMakeParams {
            caller,
            caller_caller,
            calls,
        } = params;

        let frame = StackFrameCheckedFrame {
            caller,
            caller_caller,
            calls: calls.clone(),
        };
        Self::of_frame(frame)
    }
    fn on_if(self, w: &mut Self::W) -> Self {
        let frame: &StackFrameCheckedFrame = &*self;
        w.exists_no_check(frame);
        self
    }
}

/// Call_stack_digest.Checked.cons
fn call_stack_digest_checked_cons(h: Fp, t: Fp, w: &mut Witness<Fp>) -> Fp {
    checked_hash("MinaActUpStckFrmCons", &[h, t], w)
}

impl StackInterface for WithHash<Vec<WithStackHash<WithHash<StackFrame>>>> {
    type Elt = StackFrameChecked;
    type W = Witness<Fp>;

    fn empty() -> Self {
        WithHash {
            data: Vec::new(),
            hash: Fp::zero(),
        }
    }
    fn is_empty(&self, w: &mut Self::W) -> Boolean {
        let Self { hash, data: _ } = self;
        let empty = Fp::zero();
        field::equal(empty, *hash, w)
    }
    fn pop_exn(&self) -> (Self::Elt, Self) {
        todo!()
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
        Boolean::assert_any(&[input_is_empty, is_equal], w);
        Opt {
            is_some: input_is_empty.neg(),
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

fn signature_verifies(
    shifted: &InnerCurve<Fp>,
    payload_digest: Fp,
    signature: &mina_signer::Signature,
    pk: &CompressedPubKey,
    w: &mut Witness<Fp>,
) -> Boolean {
    let pk = decompress_var(pk, w);

    let mut inputs = crate::proofs::witness::legacy_input::LegacyInput::new();
    inputs.append_field(payload_digest);

    checked_signature_verify(shifted, &pk, signature, inputs, w)
}

impl AccountUpdateInterface for SnarkAccountUpdate {
    type W = Witness<Fp>;
    type CallForest = SnarkCallForest;
    type SingleData = ZkappSingleData;

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
    fn is_proved(&self) -> Boolean {
        self.body().authorization_kind.is_proved().to_boolean()
    }
    fn is_signed(&self) -> Boolean {
        self.body().authorization_kind.is_signed().to_boolean()
    }
    fn check_authorization(
        &self,
        will_succeed: Boolean,
        commitment: Fp,
        calls: &Self::CallForest,
        data: &Self::SingleData,
        w: &mut Self::W,
    ) {
        let Self::CallForest {
            data: _,
            hash: calls,
        } = calls;

        let Self {
            body: account_update,
            authorization: control,
        } = self;

        use crate::scan_state::transaction_logic::zkapp_statement::TransactionCommitment;
        use crate::ControlTag::{NoneGiven, Proof, Signature};

        let auth_type = data.spec().auth_type;
        let proof_verifies = match auth_type {
            Proof => {
                let stmt = ZkappStatement {
                    account_update: TransactionCommitment(account_update.hash),
                    calls: TransactionCommitment(*calls),
                };
                data.set_zkapp_input(stmt);
                data.set_must_verify(will_succeed);
                Boolean::True
            }
            Signature | NoneGiven => Boolean::False,
        };

        dbg!(auth_type);

        let signature_verifies = match auth_type {
            NoneGiven | Proof => Boolean::False,
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
            }
        };
    }
}

//   let signature_verifies =
//     match auth_type with
//     | None_given | Proof ->
//         Boolean.false_
//     | Signature ->
//         let signature =
//           exists Signature_lib.Schnorr.Chunked.Signature.typ
//             ~compute:(fun () ->
//               match V.get control with
//               | Signature s ->
//                   s
//               | None_given ->
//                   Signature.dummy
//               | Proof _ ->
//                   assert false )
//         in
//         run_checked
//           (let%bind (module S) =
//              Tick.Inner_curve.Checked.Shifted.create ()
//            in
//            signature_verifies
//              ~shifted:(module S)
//              ~payload_digest:commitment signature
//              account_update.data.public_key )
//   in
//   ( `Proof_verifies proof_verifies
//   , `Signature_verifies signature_verifies )

impl LocalStateInterface for zkapp_logic::LocalState<ZkappSnark> {
    type Z = ZkappSnark;
    type W = Witness<Fp>;

    fn add_check(
        local: &mut zkapp_logic::LocalState<Self::Z>,
        failure: TransactionFailure,
        b: Boolean,
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

pub struct AccountUnhashed(Box<Account>);

impl ToFieldElements<Fp> for AccountUnhashed {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        use crate::{ReceiptChainHash, VotingFor};

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
        (
            FlaggedOption::from(zkapp.as_ref()),
            crate::ZkAppAccount::default,
        )
            .to_field_elements(fields);
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
            permissions: _,
            zkapp,
        } = &*self.0;

        token_symbol.check(w);
        balance.check(w);
        nonce.check(w);
        timing.check(w);
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
                todo!()
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
    fn verification_key_hash(&self) -> Fp {
        // TODO: We shouldn't compute the hash here
        let zkapp = self.zkapp();
        MyCow::borrow_or_else(&zkapp.verification_key, VerificationKey::dummy).hash()
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
    type InclusionProof = Vec<(Boolean, Fp)>;

    fn empty() -> Self {
        todo!()
    }

    fn get_account(
        &self,
        account_update: &Self::AccountUpdate,
        w: &mut Self::W,
    ) -> (Self::Account, Self::InclusionProof) {
        let Self { ledger, hash: root } = self;

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

    fn set_account(&mut self, account: (Self::Account, Self::InclusionProof), w: &mut Self::W) {
        todo!()
    }

    fn check_inclusion(
        &self,
        (account, incl): &(Self::Account, Self::InclusionProof),
        w: &mut Self::W,
    ) {
        let Self { ledger, hash: root } = self;
        implied_root(account, incl, w);
    }

    fn check_account(
        public_key: &mina_signer::CompressedPubKey,
        token_id: &TokenId,
        account: (&Self::Account, &Self::InclusionProof),
        w: &mut Self::W,
    ) -> Boolean {
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
        is_new
    }
}

pub struct SnarkAccountId;
pub struct SnarkTokenId;
pub struct SnarkBool;
pub struct SnarkTransactionCommitment;
pub struct SnarkVerificationKeyHash;

impl AccountIdInterface for SnarkAccountId {
    type W = Witness<Fp>;

    fn derive_token_id(account_id: &AccountId, w: &mut Self::W) -> TokenId {
        TokenId(account_id.checked_hash_with_param(AccountId::DERIVE_TOKEN_ID_HASH_PARAM, w))
    }
}

impl TokenIdInterface for SnarkTokenId {
    type W = Witness<Fp>;

    fn equal(a: &TokenId, b: &TokenId, w: &mut Self::W) -> Boolean {
        field::equal(a.0, b.0, w)
    }
}

impl VerificationKeyHashInterface for SnarkVerificationKeyHash {
    type W = Witness<Fp>;

    fn equal(a: Fp, b: Fp, w: &mut Self::W) -> Boolean {
        field::equal(a, b, w)
    }
}

impl BoolInterface for SnarkBool {
    type W = Witness<Fp>;

    fn or(a: Boolean, b: Boolean, w: &mut Self::W) -> Boolean {
        a.or(&b, w)
    }

    fn and(a: Boolean, b: Boolean, w: &mut Self::W) -> Boolean {
        a.and(&b, w)
    }
}

impl TransactionCommitmentInterface for SnarkTransactionCommitment {
    type AccountUpdate = SnarkAccountUpdate;
    type CallForest = SnarkCallForest;
    type W = Witness<Fp>;

    fn commitment(account_updates: &Self::CallForest, w: &mut Self::W) -> Fp {
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
