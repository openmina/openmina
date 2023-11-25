use mina_hasher::Fp;

use crate::proofs::numbers::currency::{CheckedAmount, CheckedSigned};
use crate::proofs::numbers::nat::{CheckedIndex, CheckedSlot};
use crate::proofs::to_field_elements::ToFieldElements;
use crate::proofs::witness::{Boolean, Check, FieldWitness, Witness};
use crate::proofs::zkapp::{GlobalStateForProof, LedgerWithHash, WithStackHash};
use crate::scan_state::currency;
use crate::scan_state::transaction_logic::local_state::{StackFrame, StackFrameChecked};
use crate::scan_state::transaction_logic::zkapp_command::{AccountUpdate, CallForest, WithHash};
use crate::sparse_ledger::LedgerIntf;
use crate::TokenId;

pub trait WitnessGenerator<F: FieldWitness> {
    fn exists<T>(&mut self, data: T) -> T
    where
        T: ToFieldElements<F> + Check<F>;

    fn exists_no_check<T>(&mut self, data: T) -> T
    where
        T: ToFieldElements<F>;
}

use WitnessGenerator as W;

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

    fn empty() -> Self;
    fn is_empty(&self, w: &mut Self::W) -> Boolean;
    fn pop_exn(&self, w: &mut Self::W) -> ((AccountUpdate, Self), Self);
}

pub trait StackFrameInterface {
    type Calls: CallForestInterface<W = Self::W>;
    type W: WitnessGenerator<Fp>;

    fn caller(&self) -> TokenId;
    fn caller_caller(&self) -> TokenId;
    fn calls(&self) -> &Self::Calls;
    fn make(caller: TokenId, caller_caller: TokenId, calls: &Self::Calls, w: &mut Self::W) -> Self;
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
    fn push(&self, elt: Self::Elt) -> Self;
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

pub trait ZkappApplication {
    type Ledger: LedgerIntf + Clone + ToFieldElements<Fp>;
    type SignedAmount: SignedAmountInterface;
    type Amount: AmountInterface;
    type Index: IndexInterface;
    type GlobalSlotSinceGenesis: GlobalSlotSinceGenesisInterface;
    type StackFrame: StackFrameInterface<W = Self::WitnessGenerator, Calls = Self::CallForest>
        + ToFieldElements<Fp>;
    type CallForest: CallForestInterface<W = Self::WitnessGenerator>;
    type CallStack: CallStackInterface<W = Self::WitnessGenerator, Elt = Self::StackFrame>
        + ToFieldElements<Fp>;
    type GlobalState: GlobalStateInterface<Ledger = Self::Ledger, SignedAmount = Self::SignedAmount>;
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
    type WitnessGenerator = Witness<Fp>;
}
// WithHash<CallForest<AccountUpdate>>
