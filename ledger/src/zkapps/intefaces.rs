use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

use crate::proofs::numbers::currency::{CheckedAmount, CheckedSigned};
use crate::proofs::numbers::nat::{CheckedIndex, CheckedSlot};
use crate::proofs::to_field_elements::ToFieldElements;
use crate::proofs::witness::{Boolean, Check, FieldWitness, Witness};
use crate::proofs::zkapp::{GlobalStateForProof, LedgerWithHash, WithStackHash, ZkappSingleData};
use crate::proofs::zkapp_logic;
use crate::scan_state::currency;
use crate::scan_state::transaction_logic::local_state::{StackFrame, StackFrameChecked};
use crate::scan_state::transaction_logic::zkapp_command::{
    AccountUpdate, AccountUpdateSkeleton, CallForest, WithHash,
};
use crate::scan_state::transaction_logic::TransactionFailure;
use crate::sparse_ledger::LedgerIntf;
use crate::{Account, AccountId, MyCow, TokenId, ZkAppAccount};

pub trait WitnessGenerator<F: FieldWitness> {
    fn exists<T>(&mut self, data: T) -> T
    where
        T: ToFieldElements<F> + Check<F>;

    fn exists_no_check<T>(&mut self, data: T) -> T
    where
        T: ToFieldElements<F>;
}

use WitnessGenerator as W;

use super::snark::{
    SnarkAccount, SnarkAccountId, SnarkBool, SnarkTokenId, SnarkTransactionCommitment,
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
    fn zero() -> Self;
    fn equal(&self, other: &Self) -> Boolean;
    fn add_flagged(&self, other: &Self) -> (Self, Boolean);
    fn add_signed_flagged(&self, signed: &impl SignedAmountInterface) -> (Self, Boolean);
    fn of_constant_fee(fee: currency::Fee) -> Self;
}

pub trait SignedAmountInterface
where
    Self: Sized,
{
    fn zero() -> Self;
    fn is_neg(&self) -> Boolean;
    fn equal(&self, other: &Self) -> Boolean;
    fn is_non_neg(&self) -> Boolean;
    fn negate(&self) -> Self;
    fn add_flagged(&self, other: &Self) -> (Self, Boolean);
    fn of_unsigned(fee: impl AmountInterface) -> Self;
}

pub trait BalanceInterface
where
    Self: Sized,
{
    type Amount: AmountInterface;
    type SignedAmount: SignedAmountInterface;
    fn sub_amount_flagged(&self, amount: Self::Amount) -> (Self, Boolean);
    fn add_signed_amount_flagged(&self, signed_amount: Self::SignedAmount) -> (Self, Boolean);
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
    fn zero() -> Self;
    fn greater_than(&self, other: &Self) -> Boolean;
    fn equal(&self, other: &Self) -> Boolean;
}

pub trait GlobalSlotSpanInterface {
    fn zero() -> Self;
    fn greater_than(&self, other: &Self) -> Boolean;
}

pub trait CallForestInterface
where
    Self: Sized,
{
    type W: WitnessGenerator<Fp>;
    type AccountUpdate: AccountUpdateInterface;

    fn empty() -> Self;
    fn is_empty(&self, w: &mut Self::W) -> Boolean;
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

    fn empty() -> Self;
    fn is_empty(&self, w: &mut Self::W) -> Boolean;
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

    fn first_pass_ledger(&self) -> Self::Ledger;
    #[must_use]
    fn set_first_pass_ledger(&self) -> Self::Ledger;

    fn second_pass_ledger(&self) -> Self::Ledger;
    #[must_use]
    fn set_second_pass_ledger(&self) -> Self::Ledger;

    fn fee_excess(&self) -> Self::SignedAmount;
    fn supply_increase(&self) -> Self::SignedAmount;
}

pub trait LocalStateInterface {
    type Z: ZkappApplication;
    type W: WitnessGenerator<Fp>;

    fn add_check(
        local: &mut zkapp_logic::LocalState<Self::Z>,
        failure: TransactionFailure,
        b: Boolean,
        w: &mut Self::W,
    );
    fn add_new_failure_status_bucket(local: &mut zkapp_logic::LocalState<Self::Z>);
}

pub trait AccountUpdateInterface
where
    Self: Sized,
{
    // Only difference in our Rust code is the `WithHash`
    fn body(&self) -> &crate::scan_state::transaction_logic::zkapp_command::Body;
    fn set(&mut self, new: Self);
    fn verification_key_hash(&self) -> Fp;
    fn is_proved(&self) -> Boolean;
    fn is_signed(&self) -> Boolean;
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

    fn equal(a: &TokenId, b: &TokenId, w: &mut Self::W) -> Boolean;
}

pub trait BoolInterface {
    type W: WitnessGenerator<Fp>;

    fn or(a: Boolean, b: Boolean, w: &mut Self::W) -> Boolean;
    fn and(a: Boolean, b: Boolean, w: &mut Self::W) -> Boolean;
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
    type D;

    fn register_verification_key(&self, data: &Self::D, w: &mut Self::W);
    fn get(&self) -> &crate::Account;
    fn get_mut(&mut self) -> &mut crate::Account;
    fn set_delegate(&mut self, new: CompressedPubKey);
    fn zkapp(&self) -> MyCow<ZkAppAccount>;
    fn verification_key_hash(&self) -> Fp;
}

pub trait LedgerInterface {
    type W: WitnessGenerator<Fp>;
    type AccountUpdate: AccountUpdateInterface;
    type Account: AccountInterface;
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
    ) -> Boolean;
}

pub trait VerificationKeyHashInterface {
    type W: WitnessGenerator<Fp>;

    fn equal(a: Fp, b: Fp, w: &mut Self::W) -> Boolean;
}

pub trait ZkappApplication {
    type Ledger: LedgerIntf
        + Clone
        + ToFieldElements<Fp>
        + LedgerInterface<
            W = Self::WitnessGenerator,
            AccountUpdate = Self::AccountUpdate,
            Account = Self::Account,
        >;
    type SignedAmount: SignedAmountInterface;
    type Amount: AmountInterface;
    type Index: IndexInterface;
    type GlobalSlotSinceGenesis: GlobalSlotSinceGenesisInterface;
    type StackFrame: StackFrameInterface<W = Self::WitnessGenerator, Calls = Self::CallForest>
        + ToFieldElements<Fp>
        + Clone;
    type CallForest: CallForestInterface<
        W = Self::WitnessGenerator,
        AccountUpdate = Self::AccountUpdate,
    >;
    type CallStack: CallStackInterface<W = Self::WitnessGenerator, Elt = Self::StackFrame>
        + ToFieldElements<Fp>
        + Clone;
    type GlobalState: GlobalStateInterface<Ledger = Self::Ledger, SignedAmount = Self::SignedAmount>;
    type AccountUpdate: AccountUpdateInterface;
    type AccountId: AccountIdInterface<W = Self::WitnessGenerator>;
    type TokenId: TokenIdInterface<W = Self::WitnessGenerator>;
    type Bool: BoolInterface<W = Self::WitnessGenerator>;
    type TransactionCommitment: TransactionCommitmentInterface<
        W = Self::WitnessGenerator,
        AccountUpdate = Self::AccountUpdate,
        CallForest = Self::CallForest,
    >;
    type FailureStatusTable;
    type LocalState: LocalStateInterface<W = Self::WitnessGenerator, Z = Self>;
    type Account: AccountInterface<W = Self::WitnessGenerator, D = Self::SingleData>;
    type VerificationKeyHash: VerificationKeyHashInterface<W = Self::WitnessGenerator>;
    type SingleData;
    type WitnessGenerator: WitnessGenerator<Fp>;
}

pub struct ZkappSnark;

impl ZkappApplication for ZkappSnark {
    type Ledger = LedgerWithHash;
    type SignedAmount = CheckedSigned<Fp, CheckedAmount<Fp>>;
    type Amount = CheckedAmount<Fp>;
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
    type Bool = SnarkBool;
    type TransactionCommitment = SnarkTransactionCommitment;
    type FailureStatusTable = ();
    type LocalState = zkapp_logic::LocalState<Self>;
    type Account = SnarkAccount;
    type VerificationKeyHash = SnarkVerificationKeyHash;
    type SingleData = ZkappSingleData;
    type WitnessGenerator = Witness<Fp>;
}
