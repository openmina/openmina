#![allow(unused)]

use std::marker::PhantomData;

use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

use crate::{
    proofs::{
        field::{Boolean, FieldWitness, ToBoolean},
        to_field_elements::ToFieldElements,
        transaction::Check,
    },
    scan_state::{
        currency::{Amount, Balance, Index, Magnitude, Signed, Slot, SlotSpan, TxnVersion},
        transaction_logic::{
            account_check_timing,
            local_state::{CallStack, StackFrame},
            protocol_state::GlobalStateSkeleton,
            zkapp_command::{
                self, AccountPreconditions, AccountUpdate, CallForest, CheckAuthorizationResult,
                SetOrKeep,
            },
            zkapp_statement::TransactionCommitment,
            TimingValidation, TransactionFailure,
        },
        zkapp_logic::controller_check,
    },
    sparse_ledger::LedgerIntf,
    Account, AccountId, Mask, MyCow, TokenId, ZkAppAccount, TXN_VERSION_CURRENT,
};

use super::{
    intefaces::{
        AccountIdInterface, AccountInterface, AccountUpdateInterface, ActionsInterface,
        AmountInterface, BalanceInterface, BoolInterface, BranchEvaluation, BranchInterface,
        BranchParam, CallForestInterface, CallStackInterface, ControllerInterface,
        GlobalSlotSinceGenesisInterface, GlobalSlotSpanInterface, GlobalStateInterface,
        IndexInterface, LedgerInterface, LocalStateInterface, Opt, ReceiptChainHashInterface,
        SetOrKeepInterface, SignedAmountBranchParam, SignedAmountInterface, StackFrameInterface,
        StackFrameMakeParams, StackInterface, TokenIdInterface, TransactionCommitmentInterface,
        TxnVersionInterface, VerificationKeyHashInterface, WitnessGenerator, ZkappApplication,
        ZkappHandler,
    },
    zkapp_logic,
};

pub type GlobalStateForNonSnark<L> = GlobalStateSkeleton<
    L,              // ledger
    Signed<Amount>, // fee_excess & supply_increase
    Slot,           // block_global_slot
>;

type NonSnarkVerificationKeyHash = Option<Fp>;
pub struct NonSnarkController;
pub struct NonSnarkSetOrKeep;
pub struct NonSnarkGlobalSlotSpan;
pub struct NonSnarkActions;
pub struct NonSnarkReceiptChainHash;
pub struct NonSnarkHandler<L>(PhantomData<L>);

pub struct ZkappNonSnark<L>(PhantomData<L>);

impl<
        L: LedgerIntf
            + Clone
            + ToFieldElements<Fp>
            + LedgerInterface<
                W = (),
                AccountUpdate = AccountUpdate,
                Account = crate::Account,
                Bool = bool,
            >,
    > ZkappApplication for ZkappNonSnark<L>
{
    type Ledger = L;
    type SignedAmount = Signed<Amount>;
    type Amount = Amount;
    type Balance = Balance;
    type Index = Index;
    type GlobalSlotSinceGenesis = Slot;
    type StackFrame = StackFrame;
    type CallForest = CallForest<AccountUpdate>;
    type CallStack = CallStack;
    type GlobalState = GlobalStateForNonSnark<L>;
    type AccountUpdate = AccountUpdate;
    type AccountId = AccountId;
    type TokenId = TokenId;
    type Bool = bool;
    type TransactionCommitment = TransactionCommitment;
    type FailureStatusTable = Vec<Vec<TransactionFailure>>;
    type LocalState = zkapp_logic::LocalState<Self>;
    type Account = Account;
    type VerificationKeyHash = NonSnarkVerificationKeyHash;
    type SingleData = ();
    type Controller = NonSnarkController;
    type TxnVersion = TxnVersion;
    type SetOrKeep = NonSnarkSetOrKeep;
    type GlobalSlotSpan = NonSnarkGlobalSlotSpan;
    type Actions = NonSnarkActions;
    type ReceiptChainHash = NonSnarkReceiptChainHash;
    type Handler = NonSnarkHandler<L>;
    type Branch = NonSnarkBranch;
    type WitnessGenerator = ();
}

impl<F: FieldWitness> WitnessGenerator<F> for () {
    type Bool = bool;

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

impl<
        L: LedgerIntf
            + Clone
            + ToFieldElements<Fp>
            + LedgerInterface<
                W = (),
                AccountUpdate = AccountUpdate,
                Account = crate::Account,
                Bool = bool,
            >,
    > ZkappHandler for NonSnarkHandler<L>
{
    type Z = ZkappNonSnark<L>;
    type AccountUpdate = AccountUpdate;
    type Account = Account;
    type Bool = bool;
    type W = ();
    type GlobalState = GlobalStateForNonSnark<L>;

    fn check_account_precondition(
        account_update: &Self::AccountUpdate,
        account: &Self::Account,
        new_account: Self::Bool,
        local_state: &mut zkapp_logic::LocalState<ZkappNonSnark<L>>,
        w: &mut Self::W,
    ) {
        let AccountPreconditions(precondition_account) = &account_update.body.preconditions.account;
        let check = |failure, b| {
            zkapp_logic::LocalState::<ZkappNonSnark<L>>::add_check(local_state, failure, b, w);
        };
        precondition_account.zcheck(new_account, check, account);
    }

    fn check_protocol_state_precondition(
        protocol_state_predicate: &zkapp_command::ZkAppPreconditions,
        global_state: &mut Self::GlobalState,
        w: &mut Self::W,
    ) -> Self::Bool {
        protocol_state_predicate
            .zcheck(&global_state.protocol_state)
            .is_ok()
    }

    fn check_valid_while_precondition(
        valid_while: &zkapp_command::Numeric<crate::scan_state::currency::Slot>,
        global_state: &mut Self::GlobalState,
        w: &mut Self::W,
    ) -> Self::Bool {
        valid_while
            .zcheck(
                "valid_while_precondition".to_string(),
                global_state.block_global_slot,
            )
            .is_ok()
    }

    fn init_account(
        _account_update: &Self::AccountUpdate,
        account: &Self::Account,
    ) -> Self::Account {
        account.clone()
    }
}

impl ReceiptChainHashInterface for NonSnarkReceiptChainHash {
    type W = ();
    type Index = Index;

    fn cons_zkapp_command_commitment(
        index: Self::Index,
        element: Fp,
        other: crate::ReceiptChainHash,
        w: &mut Self::W,
    ) -> crate::ReceiptChainHash {
        use crate::scan_state::transaction_logic::cons_zkapp_command_commitment;
        use crate::scan_state::zkapp_logic::ZkAppCommandElt;

        cons_zkapp_command_commitment(
            index,
            ZkAppCommandElt::ZkAppCommandCommitment(crate::ReceiptChainHash(element)),
            &other,
        )
    }
}

impl ActionsInterface for NonSnarkActions {
    type W = ();
    type Bool = bool;

    fn is_empty(
        actions: &crate::scan_state::transaction_logic::zkapp_command::Actions,
        _w: &mut Self::W,
    ) -> Self::Bool {
        actions.is_empty()
    }

    fn push_events(
        event: Fp,
        actions: &crate::scan_state::transaction_logic::zkapp_command::Actions,
        _w: &mut Self::W,
    ) -> Fp {
        actions.push_events(event)
    }
}

impl SetOrKeepInterface for NonSnarkSetOrKeep {
    type Bool = bool;

    fn is_keep<T: Clone>(set_or_keep: &SetOrKeep<T>) -> Self::Bool {
        set_or_keep.is_keep()
    }

    fn is_set<T: Clone>(set_or_keep: &SetOrKeep<T>) -> Self::Bool {
        set_or_keep.is_set()
    }
}

impl GlobalSlotSpanInterface for NonSnarkGlobalSlotSpan {
    type W = ();
    type Bool = bool;
    type SlotSpan = SlotSpan;

    fn greater_than(this: &Self::SlotSpan, other: &Self::SlotSpan, w: &mut Self::W) -> Self::Bool {
        this > other
    }
}

impl TxnVersionInterface for TxnVersion {
    type W = ();
    type Bool = bool;

    fn equal_to_current(version: TxnVersion, w: &mut Self::W) -> Self::Bool {
        version == TXN_VERSION_CURRENT
    }

    fn older_than_current(version: TxnVersion, w: &mut Self::W) -> Self::Bool {
        version < TXN_VERSION_CURRENT
    }
}

impl VerificationKeyHashInterface for NonSnarkVerificationKeyHash {
    type W = ();
    type Bool = bool;

    fn equal(a: &Self, b: &Self, _w: &mut Self::W) -> Self::Bool {
        a == b
    }
}

impl TransactionCommitmentInterface for TransactionCommitment {
    type AccountUpdate = AccountUpdate;
    type CallForest = CallForest<AccountUpdate>;
    type W = ();

    fn empty() -> Fp {
        let TransactionCommitment(fp) = TransactionCommitment::empty();
        fp
    }
    fn commitment(account_updates: &Self::CallForest) -> Fp {
        let account_updates_hash = account_updates.hash();
        let TransactionCommitment(fp) = TransactionCommitment::create(account_updates_hash);
        fp
    }
    fn full_commitment(
        account_update: &Self::AccountUpdate,
        memo_hash: Fp,
        commitment: Fp,
        w: &mut Self::W,
    ) -> Fp {
        // when called from Zkapp_command_logic.apply, the account_update is the fee payer
        let fee_payer_hash = account_update.digest();
        let TransactionCommitment(fp) =
            TransactionCommitment(commitment).create_complete(memo_hash, fee_payer_hash);
        fp
    }
}

impl AccountIdInterface for AccountId {
    type W = ();

    fn derive_token_id(account_id: &AccountId, w: &mut Self::W) -> TokenId {
        account_id.derive_token_id()
    }
}

impl TokenIdInterface for TokenId {
    type W = ();
    type Bool = bool;

    fn equal(a: &TokenId, b: &TokenId, w: &mut Self::W) -> Self::Bool {
        a == b
    }
}

impl<
        L: LedgerIntf
            + Clone
            + ToFieldElements<Fp>
            + LedgerInterface<
                W = (),
                AccountUpdate = AccountUpdate,
                Account = crate::Account,
                Bool = bool,
            >,
    > LocalStateInterface for zkapp_logic::LocalState<ZkappNonSnark<L>>
{
    type Z = ZkappNonSnark<L>;
    type Bool = bool;
    type W = ();

    fn add_check(
        local: &mut zkapp_logic::LocalState<Self::Z>,
        failure: TransactionFailure,
        b: Self::Bool,
        w: &mut Self::W,
    ) {
        if !b {
            local.failure_status_tbl[0].insert(0, failure);
        }
        local.success = local.success && b;
    }

    fn add_new_failure_status_bucket(local: &mut zkapp_logic::LocalState<Self::Z>) {
        local.failure_status_tbl.insert(0, Vec::new());
    }
}

impl AmountInterface for Amount {
    type W = ();
    type Bool = bool;
    fn zero() -> Self {
        <Amount as Magnitude>::zero()
    }
    fn of_constant_fee(fee: crate::scan_state::currency::Fee) -> Self {
        Amount::of_fee(&fee)
    }
}

impl BalanceInterface for Balance {
    type W = ();
    type Bool = bool;
    type Amount = Amount;
    type SignedAmount = Signed<Amount>;

    fn add_signed_amount_flagged(
        &self,
        signed_amount: Self::SignedAmount,
        _w: &mut Self::W,
    ) -> (Self, Self::Bool) {
        self.add_signed_amount_flagged(signed_amount)
    }
}

impl SignedAmountInterface for Signed<Amount> {
    type W = ();
    type Bool = bool;
    type Amount = Amount;

    fn zero() -> Self {
        Self::zero()
    }
    fn is_neg(&self) -> Self::Bool {
        self.is_neg()
    }
    fn equal(&self, other: &Self, _w: &mut Self::W) -> Self::Bool {
        (self == other)
    }
    fn is_non_neg(&self) -> Self::Bool {
        self.is_non_neg()
    }
    fn negate(&self) -> Self {
        self.negate()
    }
    fn add_flagged(&self, other: &Self, _w: &mut Self::W) -> (Self, Self::Bool) {
        self.add_flagged(*other)
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
        match b {
            true => on_true,
            false => on_false,
        }
    }
}

impl GlobalSlotSinceGenesisInterface for Slot {
    type W = ();
    type Bool = bool;

    fn equal(&self, other: &Self, w: &mut Self::W) -> Self::Bool {
        self == other
    }
}

impl<F: FieldWitness> ToFieldElements<F> for StackFrame {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        unreachable!()
    }
}

impl<L: LedgerIntf + Clone> GlobalStateInterface for GlobalStateForNonSnark<L> {
    type Ledger = L;
    type W = ();
    type Bool = bool;
    type SignedAmount = Signed<Amount>;
    type GlobalSlotSinceGenesis = Slot;

    fn first_pass_ledger(&self) -> Self::Ledger {
        self.first_pass_ledger.create_masked()
    }
    fn set_first_pass_ledger(
        &mut self,
        should_update: Self::Bool,
        ledger: &Self::Ledger,
        _w: &mut Self::W,
    ) {
        if should_update {
            self.first_pass_ledger.apply_mask(ledger.clone());
        }
    }
    fn second_pass_ledger(&self) -> Self::Ledger {
        self.second_pass_ledger.create_masked()
    }
    fn set_second_pass_ledger(
        &mut self,
        should_update: Self::Bool,
        ledger: &Self::Ledger,
        _w: &mut Self::W,
    ) {
        if should_update {
            self.second_pass_ledger.apply_mask(ledger.clone());
        }
    }
    fn fee_excess(&self) -> Self::SignedAmount {
        self.fee_excess
    }
    fn set_fee_excess(&mut self, fee_excess: Self::SignedAmount) {
        self.fee_excess = fee_excess;
    }
    fn supply_increase(&self) -> Self::SignedAmount {
        self.supply_increase
    }
    fn set_supply_increase(&mut self, supply_increase: Self::SignedAmount) {
        self.supply_increase = supply_increase;
    }
    fn block_global_slot(&self) -> Self::GlobalSlotSinceGenesis {
        self.block_global_slot
    }
}

impl StackFrameInterface for StackFrame {
    type Calls = CallForest<AccountUpdate>;
    type W = ();
    type Bool = bool;

    fn caller(&self) -> crate::TokenId {
        let Self {
            caller,
            caller_caller: _,
            calls: _,
        } = self;
        caller.clone()
    }
    fn caller_caller(&self) -> crate::TokenId {
        let Self {
            caller: _,
            caller_caller,
            calls: _,
        } = self;
        caller_caller.clone()
    }
    fn calls(&self) -> &CallForest<AccountUpdate> {
        let Self {
            caller: _,
            caller_caller: _,
            calls,
        } = self;
        calls
    }
    fn make(params: StackFrameMakeParams<'_, Self::Calls>) -> Self {
        let StackFrameMakeParams {
            caller,
            caller_caller,
            calls,
        } = params;
        Self {
            caller,
            caller_caller,
            calls: calls.clone(),
        }
    }
    fn make_default(params: StackFrameMakeParams<'_, Self::Calls>) -> Self {
        Self::make(params) // No difference in non-snark
    }
    fn on_if<F: FnOnce(&mut Self::W) -> Self, F2: FnOnce(&mut Self::W) -> Self>(
        b: Self::Bool,
        param: BranchParam<Self, Self::W, F, F2>,
        w: &mut Self::W,
    ) -> Self {
        let BranchParam { on_true, on_false } = param;

        match b {
            true => on_true.eval(w),
            false => on_false.eval(w),
        }
    }
}

impl<F: FieldWitness> ToFieldElements<F> for CallStack {
    fn to_field_elements(&self, fields: &mut Vec<F>) {
        unreachable!()
    }
}

impl StackInterface for CallStack {
    type Elt = StackFrame;
    type W = ();
    type Bool = bool;

    fn empty() -> Self {
        Self::default()
    }
    fn is_empty(&self, _w: &mut Self::W) -> Self::Bool {
        self.is_empty()
    }
    fn pop(&self, _w: &mut Self::W) -> Opt<(Self::Elt, Self)> {
        Opt::from_option(self.pop())
    }
    fn push(elt: Self::Elt, onto: Self, _w: &mut Self::W) -> Self {
        onto.push(&elt)
    }
}

impl CallStackInterface for CallStack {
    type StackFrame = StackFrame;
}

impl IndexInterface for Index {
    fn zero() -> Self {
        <Index as Magnitude>::zero()
    }
    fn succ(&self) -> Self {
        self.incr()
    }
}

impl CallForestInterface for CallForest<AccountUpdate> {
    type W = ();
    type AccountUpdate = AccountUpdate;
    type Bool = bool;

    fn empty() -> Self {
        Self::empty()
    }
    fn is_empty(&self, _w: &mut Self::W) -> Self::Bool {
        self.is_empty()
    }
    fn pop_exn(&self, _w: &mut Self::W) -> ((AccountUpdate, Self), Self) {
        self.pop_exn()
    }
}

impl BoolInterface for bool {
    type W = ();
    type FailureStatusTable = Vec<Vec<TransactionFailure>>;

    fn as_boolean(&self) -> Boolean {
        self.to_boolean()
    }
    fn of_boolean(b: Boolean) -> Self {
        b.as_bool()
    }
    fn true_() -> Self {
        true
    }
    fn false_() -> Self {
        false
    }
    fn neg(&self) -> Self {
        !self
    }
    fn or(a: Self, b: Self, w: &mut Self::W) -> Self {
        a || b
    }
    fn and(a: Self, b: Self, w: &mut Self::W) -> Self {
        a && b
    }
    fn equal(a: Self, b: Self, w: &mut Self::W) -> Self {
        a == b
    }
    fn all(bs: &[Self], w: &mut Self::W) -> Self {
        bs.iter().all(|b| *b)
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

impl AccountInterface for Account {
    type W = ();
    type Bool = bool;
    type Balance = Balance;
    type GlobalSlot = Slot;
    type D = ();
    type VerificationKeyHash = NonSnarkVerificationKeyHash;

    fn register_verification_key(&self, data: &Self::D, w: &mut Self::W) {
        // Nothing
    }

    fn get(&self) -> &crate::Account {
        self
    }

    fn get_mut(&mut self) -> &mut crate::Account {
        self
    }

    fn set_delegate(&mut self, new: CompressedPubKey) {
        self.delegate = if new == CompressedPubKey::empty() {
            None
        } else {
            Some(new)
        };
    }

    fn zkapp(&self) -> MyCow<ZkAppAccount> {
        match self.zkapp.as_ref() {
            Some(zkapp) => MyCow::Borrow(zkapp),
            None => MyCow::Own(ZkAppAccount::default()),
        }
    }

    fn zkapp_mut(&mut self) -> &mut ZkAppAccount {
        // `unwrap`: `make_zkapp` is supposed to be called before `zkapp_mut`
        self.zkapp.as_mut().unwrap()
    }

    fn verification_key_hash(&self) -> NonSnarkVerificationKeyHash {
        Some(self.zkapp.as_ref()?.verification_key.as_ref()?.hash())
    }

    fn set_token_id(&mut self, token_id: TokenId) {
        self.token_id = token_id;
    }

    fn is_timed(&self) -> Self::Bool {
        match &self.timing {
            crate::Timing::Untimed => false,
            crate::Timing::Timed { .. } => true,
        }
    }

    fn balance(&self) -> Self::Balance {
        self.balance
    }

    fn set_balance(&mut self, balance: Self::Balance) {
        self.balance = balance;
    }

    fn check_timing(
        &self,
        txn_global_slot: &Self::GlobalSlot,
        w: &mut Self::W,
    ) -> (TimingValidation<Self::Bool>, crate::Timing) {
        account_check_timing(txn_global_slot, self)
    }

    fn make_zkapp(&mut self) {
        if self.zkapp.is_none() {
            // ZkAppAccount::default
            self.zkapp = Some(Box::default());
        }
    }

    fn unmake_zkapp(&mut self) {
        if self.zkapp.as_ref().map(|z| z.is_default()).unwrap_or(false) {
            self.zkapp = None;
        }
    }

    fn proved_state(&self) -> Self::Bool {
        self.zkapp().proved_state
    }

    fn set_proved_state(&mut self, proved_state: Self::Bool) {
        self.zkapp_mut().proved_state = proved_state;
    }

    fn app_state(&self) -> [Fp; 8] {
        self.zkapp().app_state
    }

    fn last_action_slot(&self) -> Self::GlobalSlot {
        self.zkapp().last_action_slot
    }

    fn set_last_action_slot(&mut self, slot: Self::GlobalSlot) {
        self.zkapp_mut().last_action_slot = slot;
    }
}

impl AccountUpdateInterface for AccountUpdate {
    type W = ();
    type SingleData = ();
    type CallForest = CallForest<AccountUpdate>;
    type Bool = bool;
    type SignedAmount = Signed<Amount>;
    type VerificationKeyHash = NonSnarkVerificationKeyHash;

    fn body(&self) -> &crate::scan_state::transaction_logic::zkapp_command::Body {
        let Self {
            body,
            authorization: _,
        } = self;
        body
    }
    fn is_proved(&self) -> Self::Bool {
        self.body().authorization_kind.is_proved()
    }
    fn is_signed(&self) -> Self::Bool {
        self.body().authorization_kind.is_signed()
    }
    fn verification_key_hash(&self) -> Self::VerificationKeyHash {
        use crate::scan_state::transaction_logic::zkapp_command::AuthorizationKind::*;

        match &self.body().authorization_kind {
            Proof(vk_hash) => Some(*vk_hash),
            NoneGiven | Signature => None,
        }
    }
    fn check_authorization(
        &self,
        will_succeed: Self::Bool,
        commitment: Fp,
        calls: &Self::CallForest,
        single_data: &Self::SingleData,
        w: &mut Self::W,
    ) -> CheckAuthorizationResult<Self::Bool> {
        use crate::scan_state::transaction_logic::zkapp_command::Control::*;

        let (proof_verifies, signature_verifies) = match &self.authorization {
            Signature(_) => (false, true),
            Proof(_) => (true, false),
            NoneGiven => (false, false),
        };
        CheckAuthorizationResult {
            proof_verifies,
            signature_verifies,
        }
    }
    fn increment_nonce(&self) -> Self::Bool {
        self.body().increment_nonce
    }
    fn use_full_commitment(&self) -> Self::Bool {
        self.body().use_full_commitment
    }
    fn account_precondition_nonce_is_constant(&self, w: &mut Self::W) -> Self::Bool {
        let nonce = self.body().preconditions.account.nonce();
        nonce.is_constant()
    }
    fn implicit_account_creation_fee(&self) -> Self::Bool {
        self.body().implicit_account_creation_fee
    }
    fn balance_change(&self) -> Self::SignedAmount {
        self.body().balance_change
    }
}

impl ControllerInterface for NonSnarkController {
    type W = ();
    type Bool = bool;
    type SingleData = ();

    fn check(
        proof_verifies: Self::Bool,
        signature_verifies: Self::Bool,
        auth: &crate::AuthRequired,
        _single_data: &Self::SingleData,
        _w: &mut Self::W,
    ) -> Result<Self::Bool, String> {
        controller_check(proof_verifies, signature_verifies, *auth)
    }

    fn verification_key_perm_fallback_to_signature_with_older_version(
        auth: &crate::AuthRequired,
        w: &mut Self::W,
    ) -> crate::AuthRequired {
        auth.verification_key_perm_fallback_to_signature_with_older_version()
    }
}

pub struct NonSnarkBranch;

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
