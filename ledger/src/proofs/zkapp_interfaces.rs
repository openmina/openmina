use crate::scan_state::currency;
use crate::sparse_ledger::LedgerIntf;

use super::numbers::currency as checked_currency;
use super::witness::Boolean;

trait AmountInterface
where
    Self: Sized,
{
    fn zero() -> Self;
    fn equal(&self, other: &Self) -> Boolean;
    fn add_flagged(&self, other: &Self) -> (Self, Boolean);
    fn add_signed_flagged(&self, signed: &impl SignedAmountInterface) -> (Self, Boolean);
    fn of_constant_fee(fee: currency::Fee) -> Self;
}

trait SignedAmountInterface
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

trait BalanceInterface
where
    Self: Sized,
{
    type Amount: AmountInterface;
    type SignedAmount: SignedAmountInterface;
    fn sub_amount_flagged(&self, amount: Self::Amount) -> (Self, Boolean);
    fn add_signed_amount_flagged(&self, signed_amount: Self::SignedAmount) -> (Self, Boolean);
}

trait ReceiptChainHashElementInterface
where
    Self: Sized,
{
    fn of_commitment(commitment: impl ReceiptChainHashInterface) -> Self;
}

trait ReceiptChainHashInterface {
    type TransactionCommitment;
    type Index;
    fn cons_zkapp_command_commitment(
        index: Self::Index,
        element: impl ReceiptChainHashElementInterface,
        other: &Self,
    ) -> Self;
}

trait GlobalSlotSinceGenesisInterface {
    fn zero() -> Self;
    fn greater_than(&self, other: &Self) -> Boolean;
    fn equal(&self, other: &Self) -> Boolean;
}

trait GlobalSlotSpanInterface {
    fn zero() -> Self;
    fn greater_than(&self, other: &Self) -> Boolean;
}

// pub type GlobalState<L> = GlobalStateSkeleton<L, Signed<Amount>, Slot>;

// #[derive(Debug, Clone)]
// pub struct GlobalStateSkeleton<L, SignedAmount, Slot> {
//     pub first_pass_ledger: L,
//     pub second_pass_ledger: L,
//     pub fee_excess: SignedAmount,
//     pub supply_increase: SignedAmount,
//     pub protocol_state: ProtocolStateView,
//     /// Slot of block when the transaction is applied.
//     /// NOTE: This is at least 1 slot after the protocol_state's view,
//     /// which is for the *previous* slot.
//     pub block_global_slot: Slot,
// }

trait ZkappApplication {
    type GlobalState: GlobalStateInterface;
    type LocalState;
}

trait GlobalStateInterface {
    type Ledger: LedgerIntf;
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
