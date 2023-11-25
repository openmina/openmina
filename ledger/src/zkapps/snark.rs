use ark_ff::Zero;
use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

use crate::{
    proofs::{
        numbers::{
            currency::{CheckedAmount, CheckedSigned},
            nat::{CheckedIndex, CheckedSlot},
        },
        to_field_elements::ToFieldElements,
        witness::{
            field, transaction_snark::checked_hash, Boolean, Check, FieldWitness, ToBoolean,
            Witness,
        },
        zkapp::{GlobalStateForProof, LedgerWithHash, WithStackHash},
        zkapp_logic,
    },
    scan_state::transaction_logic::{
        local_state::{StackFrame, StackFrameChecked, StackFrameCheckedFrame},
        zkapp_command::{
            AccountUpdate, AccountUpdateSkeleton, CallForest, Tree, WithHash,
            ACCOUNT_UPDATE_CONS_HASH_PARAM,
        },
        TransactionFailure,
    },
    Account, AccountId, MyCow, ToInputs, TokenId, VerificationKey,
};

use super::intefaces::{
    AccountIdInterface, AccountUpdateInterface, AmountInterface, BoolInterface,
    CallForestInterface, CallStackInterface, GlobalSlotSinceGenesisInterface, GlobalStateInterface,
    IndexInterface, LedgerInterface, LocalStateInterface, Opt, SignedAmountInterface,
    StackFrameInterface, StackFrameMakeParams, StackInterface, TokenIdInterface,
    TransactionCommitmentInterface, WitnessGenerator, ZkappSnark,
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

impl AccountUpdateInterface for SnarkAccountUpdate {
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
}

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

    fn add_new_failure_status_bucket(self) -> Self {
        todo!()
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

impl<T, F> Check<Fp> for (FlaggedOption<T>, F)
where
    T: Check<Fp>,
    F: Fn() -> T,
{
    fn check(&self, w: &mut Witness<Fp>) {
        let (or_ignore, default_fn) = self;
        let value = match or_ignore {
            FlaggedOption::Some(this) => MyCow::Borrow(this),
            FlaggedOption::None => MyCow::Own(default_fn()),
        };
        crate::proofs::witness::Check::check(&*value, w);
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

        let zkapp = MyCow::borrow_or_default(zkapp);
        let crate::ZkAppAccount {
            app_state,
            verification_key,
            zkapp_version,
            action_state,
            last_action_slot,
            proved_state,
            zkapp_uri,
        } = &*zkapp;

        // app_state.to_field_elements(fields);

        // (FlaggedOption::from(verification_key.as_ref()), VerificationKey::dummy).to_field_elements(fields);

        // verification_key.to_field_elements(fields);
    }
}

impl LedgerInterface for LedgerWithHash {
    type W = Witness<Fp>;
    type AccountUpdate = SnarkAccountUpdate;
    type InclusionProof = Vec<(Boolean, Fp)>;

    fn empty() -> Self {
        todo!()
    }

    fn get_account(
        &self,
        account_update: &Self::AccountUpdate,
        w: &mut Self::W,
    ) -> (Account, Self::InclusionProof) {
        let Self { ledger, hash: root } = self;

        let idx = ledger.find_index_exn(account_update.body.account_id());
        let account = w.exists_no_check(AccountUnhashed(ledger.get_exn(&idx)));

        // let get_account { account_update; _ } ((_root, ledger) : t) =
        //   let idx =
        //     V.map ledger ~f:(fun l -> idx l (body_id account_update.data))
        //   in
        //   let account =
        //     exists Mina_base.Account.Checked.Unhashed.typ
        //       ~compute:(fun () ->
        //         Sparse_ledger.get_exn (V.get ledger) (V.get idx) )
        //   in
        //   let account = Account.account_with_hash account in
        //   let incl =
        //     exists
        //       Typ.(
        //         list ~length:constraint_constants.ledger_depth
        //           (Boolean.typ * field))
        //       ~compute:(fun () ->
        //         List.map
        //           (Sparse_ledger.path_exn (V.get ledger) (V.get idx))
        //           ~f:(fun x ->
        //             match x with
        //             | `Left h ->
        //                 (false, h)
        //             | `Right h ->
        //                 (true, h) ) )
        //   in
        //   (account, incl)

        todo!()
    }

    fn set_account(&mut self, account: (crate::Account, Self::InclusionProof), w: &mut Self::W) {
        todo!()
    }

    fn check_inclusion(&self, account: &(crate::Account, Self::InclusionProof), w: &mut Self::W) {
        todo!()
    }

    fn check_account(
        public_key: &mina_signer::CompressedPubKey,
        token_id: &TokenId,
        account: &(crate::Account, Self::InclusionProof),
        w: &mut Self::W,
    ) -> Boolean {
        todo!()
    }
}

pub struct SnarkAccountId;
pub struct SnarkTokenId;
pub struct SnarkBool;
pub struct SnarkTransactionCommitment;

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
        todo!()
    }

    fn full_commitment(
        account_updates: &Self::AccountUpdate,
        memo_hash: Fp,
        w: &mut Self::W,
    ) -> Fp {
        todo!()
    }
}
