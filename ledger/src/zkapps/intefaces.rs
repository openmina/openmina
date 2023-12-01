use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

use crate::proofs::numbers::currency::{CheckedAmount, CheckedSigned};
use crate::proofs::numbers::nat::{CheckedIndex, CheckedSlot};
use crate::proofs::to_field_elements::ToFieldElements;
use crate::proofs::witness::{Boolean, Check, FieldWitness, Witness};
use crate::proofs::wrap::CircuitVar;
use crate::proofs::zkapp::{GlobalStateForProof, LedgerWithHash, WithStackHash, ZkappSingleData};
use crate::proofs::zkapp_logic;
use crate::scan_state::currency::{self, SlotSpan};
use crate::scan_state::transaction_logic::local_state::{StackFrame, StackFrameChecked};
use crate::scan_state::transaction_logic::zkapp_command::{
    self, AccountUpdate, AccountUpdateSkeleton, CallForest, CheckAuthorizationResult, SetOrKeep,
    WithHash,
};
use crate::scan_state::transaction_logic::TransactionFailure;
use crate::sparse_ledger::LedgerIntf;
use crate::{Account, AccountId, AuthRequired, MyCow, TokenId, ZkAppAccount};

pub trait WitnessGenerator<F: FieldWitness> {
    type Bool: BoolInterface;

    fn exists<T>(&mut self, data: T) -> T
    where
        T: ToFieldElements<F> + Check<F>;

    fn exists_no_check<T>(&mut self, data: T) -> T
    where
        T: ToFieldElements<F>;

    /// Here `b` might be a `CircuitVar::Constant`, in that case we don't call
    /// `Witness::exists_no_check` for the value.
    /// https://github.com/openmina/snarky/blob/ff2631f47bb644f7a31fd30be16ee0e5ff5279fa/src/base/utils.ml#L155
    ///
    /// TODO: Ideally we should replace `exists_no_check` above with this `exists_no_check_on_bool`,
    /// but it's more parameters to type, and most of the time `b` is not a constant
    fn exists_no_check_on_bool<T>(&mut self, b: Self::Bool, data: T) -> T
    where
        T: ToFieldElements<F>;
}

use WitnessGenerator as W;

use super::snark::{
    SnarkAccount, SnarkAccountId, SnarkActions, SnarkAmount, SnarkBalance, SnarkBool,
    SnarkController, SnarkGlobalSlotSpan, SnarkSetOrKeep, SnarkTokenId, SnarkTransactionCommitment,
    SnarkVerificationKeyHash,
};

pub struct Opt<T> {
    pub is_some: Boolean,
    pub data: T,
}

impl<A, B> Opt<(A, B)> {
    pub fn unzip(self) -> (Opt<A>, Opt<B>) {
        let Self {
            is_some,
            data: (a, b),
        } = self;
        let a = Opt { is_some, data: a };
        let b = Opt { is_some, data: b };
        (a, b)
    }
}

pub trait AmountInterface
where
    Self: Sized,
{
    type W: WitnessGenerator<Fp>;
    type Bool: BoolInterface;

    fn zero() -> Self;
    fn equal(&self, other: &Self) -> Self::Bool;
    fn add_flagged(&self, other: &Self, w: &mut Self::W) -> (Self, Self::Bool);
    fn add_signed_flagged(&self, signed: &impl SignedAmountInterface) -> (Self, Self::Bool);
    fn of_constant_fee(fee: currency::Fee) -> Self;
}

pub trait SignedAmountInterface
where
    Self: Sized,
{
    type W: WitnessGenerator<Fp>;
    type Bool: BoolInterface;
    type Amount: AmountInterface;

    fn zero() -> Self;
    fn is_neg(&self) -> Self::Bool;
    fn equal(&self, other: &Self, w: &mut Self::W) -> Self::Bool;
    fn is_non_neg(&self) -> Self::Bool;
    fn negate(&self) -> Self;
    fn add_flagged(&self, other: &Self, w: &mut Self::W) -> (Self, Self::Bool);
    fn of_unsigned(unsigned: Self::Amount) -> Self;
    fn value(&self) -> Fp;
}

pub trait BalanceInterface
where
    Self: Sized,
{
    type W: WitnessGenerator<Fp>;
    type Bool: BoolInterface;
    type Amount: AmountInterface;
    type SignedAmount: SignedAmountInterface;

    fn sub_amount_flagged(&self, amount: Self::Amount) -> (Self, Self::Bool);
    fn add_signed_amount_flagged(
        &self,
        signed_amount: Self::SignedAmount,
        w: &mut Self::W,
    ) -> (Self, Self::Bool);
}

pub trait IndexInterface
where
    Self: Sized,
{
    fn zero() -> Self;
    fn succ(&self) -> Self;
}

pub trait ReceiptChainHashElementInterface
where
    Self: Sized,
{
    fn of_commitment(commitment: impl ReceiptChainHashInterface) -> Self;
}

pub trait ReceiptChainHashInterface {
    type TransactionCommitment;
    type Index;
    fn cons_zkapp_command_commitment(
        index: Self::Index,
        element: impl ReceiptChainHashElementInterface,
        other: &Self,
    ) -> Self;
}

pub trait GlobalSlotSinceGenesisInterface {
    type W: WitnessGenerator<Fp>;
    type Bool: BoolInterface;

    fn zero() -> Self;
    fn greater_than(&self, other: &Self) -> Self::Bool;
    fn equal(&self, other: &Self, w: &mut Self::W) -> Self::Bool;
}

pub trait GlobalSlotSpanInterface {
    type W: WitnessGenerator<Fp>;
    type Bool: BoolInterface;
    type SlotSpan;

    fn zero() -> Self;
    fn greater_than(this: &Self::SlotSpan, other: &Self::SlotSpan, w: &mut Self::W) -> Self::Bool;
}

pub trait CallForestInterface
where
    Self: Sized,
{
    type W: WitnessGenerator<Fp>;
    type AccountUpdate: AccountUpdateInterface;
    type Bool: BoolInterface;

    fn empty() -> Self;
    fn is_empty(&self, w: &mut Self::W) -> Self::Bool;
    fn pop_exn(&self, w: &mut Self::W) -> ((Self::AccountUpdate, Self), Self);
}

pub struct StackFrameMakeParams<'a, Calls> {
    pub caller: TokenId,
    pub caller_caller: TokenId,
    pub calls: &'a Calls,
}

pub trait StackFrameInterface {
    type Calls: CallForestInterface<W = Self::W>;
    type W: WitnessGenerator<Fp>;

    fn caller(&self) -> TokenId;
    fn caller_caller(&self) -> TokenId;
    fn calls(&self) -> &Self::Calls;
    fn make(params: StackFrameMakeParams<'_, Self::Calls>, w: &mut Self::W) -> Self;
    fn on_if(self, w: &mut Self::W) -> Self;
}

pub trait StackInterface
where
    Self: Sized,
{
    type Elt;
    type W: WitnessGenerator<Fp>;
    type Bool: BoolInterface;

    fn empty() -> Self;
    fn is_empty(&self, w: &mut Self::W) -> Self::Bool;
    fn pop_exn(&self) -> (Self::Elt, Self);
    fn pop(&self, w: &mut Self::W) -> Opt<(Self::Elt, Self)>;
    fn push(elt: Self::Elt, onto: Self, w: &mut Self::W) -> Self;
}

pub trait CallStackInterface
where
    Self: Sized + StackInterface,
{
    type StackFrame: StackFrameInterface;
}

pub trait GlobalStateInterface {
    type Ledger;
    type SignedAmount: SignedAmountInterface;
    type GlobalSlotSinceGenesis: GlobalSlotSinceGenesisInterface;

    fn first_pass_ledger(&self) -> Self::Ledger;
    #[must_use]
    fn set_first_pass_ledger(&self) -> Self::Ledger;

    fn second_pass_ledger(&self) -> Self::Ledger;
    #[must_use]
    fn set_second_pass_ledger(&self) -> Self::Ledger;

    fn fee_excess(&self) -> Self::SignedAmount;
    fn set_fee_excess(&mut self, fee_excess: Self::SignedAmount);

    fn supply_increase(&self) -> Self::SignedAmount;
    fn set_supply_increase(&mut self, supply_increase: Self::SignedAmount);

    fn block_global_slot(&self) -> Self::GlobalSlotSinceGenesis;
}

pub trait LocalStateInterface {
    type Z: ZkappApplication;
    type W: WitnessGenerator<Fp>;
    type Bool: BoolInterface;

    fn add_check(
        local: &mut zkapp_logic::LocalState<Self::Z>,
        failure: TransactionFailure,
        b: Self::Bool,
        w: &mut Self::W,
    );
    fn add_new_failure_status_bucket(local: &mut zkapp_logic::LocalState<Self::Z>);
}

pub trait AccountUpdateInterface
where
    Self: Sized,
{
    type W: WitnessGenerator<Fp>;
    type SingleData;
    type CallForest: CallForestInterface;
    type Bool: BoolInterface;
    type SignedAmount: SignedAmountInterface;

    // Only difference in our Rust code is the `WithHash`
    fn body(&self) -> &crate::scan_state::transaction_logic::zkapp_command::Body;
    fn set(&mut self, new: Self);
    fn verification_key_hash(&self) -> Fp;
    fn is_proved(&self) -> Self::Bool;
    fn is_signed(&self) -> Self::Bool;
    fn check_authorization(
        &self,
        will_succeed: Self::Bool,
        commitment: Fp,
        calls: &Self::CallForest,
        data: &Self::SingleData,
        w: &mut Self::W,
    ) -> CheckAuthorizationResult<Self::Bool>;
    fn increment_nonce(&self) -> Self::Bool;
    fn use_full_commitment(&self) -> Self::Bool;
    fn account_precondition_nonce_is_constant(&self, w: &mut Self::W) -> Self::Bool;
    fn implicit_account_creation_fee(&self) -> Self::Bool;
    fn balance_change(&self) -> Self::SignedAmount;
}

pub trait AccountIdInterface
where
    Self: Sized,
{
    type W: WitnessGenerator<Fp>;

    fn derive_token_id(account_id: &AccountId, w: &mut Self::W) -> TokenId;
}

pub trait TokenIdInterface
where
    Self: Sized,
{
    type W: WitnessGenerator<Fp>;
    type Bool: BoolInterface;

    fn equal(a: &TokenId, b: &TokenId, w: &mut Self::W) -> Self::Bool;
}

pub trait ControllerInterface {
    type W: WitnessGenerator<Fp>;
    type Bool: BoolInterface;
    type SingleData;

    fn check(
        proof_verifies: Self::Bool,
        signature_verifies: Self::Bool,
        auth: &AuthRequired,
        data: &Self::SingleData,
        w: &mut Self::W,
    ) -> Self::Bool;
}

pub trait BoolInterface
where
    Self: Sized,
{
    type W: WitnessGenerator<Fp>;

    fn as_boolean(&self) -> Boolean;
    fn true_() -> Self;
    fn false_() -> Self;
    fn neg(&self) -> Self;
    fn or(a: Self, b: Self, w: &mut Self::W) -> Self;
    fn and(a: Self, b: Self, w: &mut Self::W) -> Self;
    fn equal(a: Self, b: Self, w: &mut Self::W) -> Self;
    fn all(bs: &[Self], w: &mut Self::W) -> Self;
}

pub trait TransactionCommitmentInterface {
    type AccountUpdate: AccountUpdateInterface;
    type CallForest: CallForestInterface;
    type W: WitnessGenerator<Fp>;

    fn commitment(account_updates: &Self::CallForest, w: &mut Self::W) -> Fp;
    fn full_commitment(
        account_updates: &Self::AccountUpdate,
        memo_hash: Fp,
        commitment: Fp,
        w: &mut Self::W,
    ) -> Fp;
}

pub trait AccountInterface
where
    Self: Sized,
{
    type W: WitnessGenerator<Fp>;
    type Bool: BoolInterface;
    type Balance: BalanceInterface;
    type GlobalSlot: GlobalSlotSinceGenesisInterface;
    type D;

    fn register_verification_key(&self, data: &Self::D, w: &mut Self::W);
    fn get(&self) -> &crate::Account;
    fn get_mut(&mut self) -> &mut crate::Account;
    fn set_delegate(&mut self, new: CompressedPubKey);
    fn zkapp(&self) -> MyCow<ZkAppAccount>;
    fn verification_key_hash(&self) -> Fp;
    fn set_token_id(&mut self, token_id: TokenId);
    fn is_timed(&self) -> Self::Bool;
    fn balance(&self) -> Self::Balance;
    fn set_balance(&mut self, balance: Self::Balance);
    fn check_timing(
        &self,
        txn_global_slot: &Self::GlobalSlot,
        w: &mut Self::W,
    ) -> (Self::Bool, crate::Timing);
    fn make_zkapp(&mut self);
    fn unmake_zkapp(&mut self);
    fn proved_state(&self) -> Self::Bool;
    fn set_proved_state(&mut self, proved_state: Self::Bool);
    fn app_state(&self) -> [Fp; 8];
    fn last_action_slot(&self) -> Self::GlobalSlot;
    fn set_last_action_slot(&mut self, slot: Self::GlobalSlot);
}

pub trait LedgerInterface {
    type W: WitnessGenerator<Fp>;
    type AccountUpdate: AccountUpdateInterface;
    type Account: AccountInterface;
    type Bool: BoolInterface;
    type InclusionProof;

    fn empty() -> Self;
    fn get_account(
        &self,
        account_update: &Self::AccountUpdate,
        w: &mut Self::W,
    ) -> (Self::Account, Self::InclusionProof);
    fn set_account(&mut self, account: (Self::Account, Self::InclusionProof), w: &mut Self::W);
    fn check_inclusion(&self, account: &(Self::Account, Self::InclusionProof), w: &mut Self::W);
    fn check_account(
        public_key: &CompressedPubKey,
        token_id: &TokenId,
        account: (&Self::Account, &Self::InclusionProof),
        w: &mut Self::W,
    ) -> Self::Bool;
}

pub trait VerificationKeyHashInterface {
    type W: WitnessGenerator<Fp>;
    type Bool: BoolInterface;

    fn equal(a: Fp, b: Fp, w: &mut Self::W) -> Self::Bool;
}

pub trait SetOrKeepInterface {
    type Bool: BoolInterface;
    fn is_keep<T: Clone>(set_or_keep: &SetOrKeep<T>) -> Self::Bool;
    fn is_set<T: Clone>(set_or_keep: &SetOrKeep<T>) -> Self::Bool;
}

pub trait ActionsInterface {
    type W: WitnessGenerator<Fp>;
    type Bool: BoolInterface;

    fn is_empty(actions: &zkapp_command::Actions, w: &mut Self::W) -> Self::Bool;
    fn push_events(event: Fp, actions: &zkapp_command::Actions, w: &mut Self::W) -> Fp;
}

pub trait ZkappApplication {
    type Ledger: LedgerIntf
        + Clone
        + ToFieldElements<Fp>
        + LedgerInterface<
            W = Self::WitnessGenerator,
            AccountUpdate = Self::AccountUpdate,
            Account = Self::Account,
            Bool = Self::Bool,
        >;
    type SignedAmount: SignedAmountInterface<W = Self::WitnessGenerator, Bool = Self::Bool, Amount = Self::Amount>
        + std::fmt::Debug
        + Clone
        + ToFieldElements<Fp>;
    type Amount: AmountInterface<W = Self::WitnessGenerator, Bool = Self::Bool> + Clone;
    type Balance: BalanceInterface<
        W = Self::WitnessGenerator,
        Bool = Self::Bool,
        Amount = Self::Amount,
        SignedAmount = Self::SignedAmount,
    >;
    type Index: IndexInterface;
    type GlobalSlotSinceGenesis: GlobalSlotSinceGenesisInterface<W = Self::WitnessGenerator, Bool = Self::Bool>
        + ToFieldElements<Fp>;
    type StackFrame: StackFrameInterface<W = Self::WitnessGenerator, Calls = Self::CallForest>
        + ToFieldElements<Fp>
        + Clone;
    type CallForest: CallForestInterface<
        W = Self::WitnessGenerator,
        AccountUpdate = Self::AccountUpdate,
        Bool = Self::Bool,
    >;
    type CallStack: CallStackInterface<W = Self::WitnessGenerator, Elt = Self::StackFrame, Bool = Self::Bool>
        + ToFieldElements<Fp>
        + Clone;
    type GlobalState: GlobalStateInterface<
        Ledger = Self::Ledger,
        SignedAmount = Self::SignedAmount,
        GlobalSlotSinceGenesis = Self::GlobalSlotSinceGenesis,
    >;
    type AccountUpdate: AccountUpdateInterface<
        W = Self::WitnessGenerator,
        CallForest = Self::CallForest,
        SingleData = Self::SingleData,
        Bool = Self::Bool,
        SignedAmount = Self::SignedAmount,
    >;
    type AccountId: AccountIdInterface<W = Self::WitnessGenerator>;
    type TokenId: TokenIdInterface<W = Self::WitnessGenerator, Bool = Self::Bool>;
    type Bool: BoolInterface<W = Self::WitnessGenerator>
        + ToFieldElements<Fp>
        + Clone
        + Copy
        + std::fmt::Debug;
    type TransactionCommitment: TransactionCommitmentInterface<
        W = Self::WitnessGenerator,
        AccountUpdate = Self::AccountUpdate,
        CallForest = Self::CallForest,
    >;
    type FailureStatusTable;
    type LocalState: LocalStateInterface<W = Self::WitnessGenerator, Z = Self, Bool = Self::Bool>;
    type Account: AccountInterface<
        W = Self::WitnessGenerator,
        D = Self::SingleData,
        Bool = Self::Bool,
        Balance = Self::Balance,
        GlobalSlot = Self::GlobalSlotSinceGenesis,
    >;
    type VerificationKeyHash: VerificationKeyHashInterface<
        W = Self::WitnessGenerator,
        Bool = Self::Bool,
    >;
    type Controller: ControllerInterface<
        W = Self::WitnessGenerator,
        Bool = Self::Bool,
        SingleData = Self::SingleData,
    >;
    type SetOrKeep: SetOrKeepInterface<Bool = Self::Bool>;
    type GlobalSlotSpan: GlobalSlotSpanInterface<
        W = Self::WitnessGenerator,
        Bool = Self::Bool,
        SlotSpan = SlotSpan,
    >;
    type Actions: ActionsInterface<W = Self::WitnessGenerator, Bool = Self::Bool>;
    type SingleData;
    type WitnessGenerator: WitnessGenerator<Fp, Bool = Self::Bool>;
}

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
    type SetOrKeep = SnarkSetOrKeep;
    type GlobalSlotSpan = SnarkGlobalSlotSpan;
    type Actions = SnarkActions;
    type WitnessGenerator = Witness<Fp>;
}
