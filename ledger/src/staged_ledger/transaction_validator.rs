use openmina_core::constants::ConstraintConstants;

use crate::{
    scan_state::{
        currency::Slot,
        transaction_logic::{
            self,
            protocol_state::ProtocolStateView,
            signed_command::SignedCommand,
            transaction_applied::{SignedCommandApplied, TransactionApplied},
            transaction_partially_applied::TransactionPartiallyApplied,
            Transaction,
        },
    },
    Mask,
};

fn within_mask<F, R, E>(base_ledger: Mask, fun: F) -> Result<R, E>
where
    F: Fn(&mut Mask) -> Result<R, E>,
{
    let mut mask = base_ledger.make_child();
    let res = fun(&mut mask);

    if res.is_ok() {
        mask.commit();
    }

    res
}

pub fn apply_transaction_first_pass(
    constraint_constants: &ConstraintConstants,
    global_slot: Slot,
    txn_state_view: &ProtocolStateView,
    ledger: &mut Mask,
    transaction: &Transaction,
) -> Result<TransactionPartiallyApplied<Mask>, String> {
    within_mask(ledger.clone(), |ledger| {
        transaction_logic::apply_transaction_first_pass(
            constraint_constants,
            global_slot,
            txn_state_view,
            ledger,
            transaction,
        )
    })
}

pub fn apply_transactions(
    constraint_constants: &ConstraintConstants,
    global_slot: Slot,
    txn_state_view: &ProtocolStateView,
    ledger: &mut Mask,
    txns: Vec<Transaction>,
) -> Result<Vec<TransactionApplied>, String> {
    within_mask(ledger.clone(), |ledger| {
        transaction_logic::apply_transactions(
            constraint_constants,
            global_slot,
            txn_state_view,
            ledger,
            &txns,
        )
    })
}

pub fn apply_user_command(
    constraint_constants: &ConstraintConstants,
    txn_state_view: &ProtocolStateView,
    txn_global_slot: &Slot,
    ledger: &mut Mask,
    user_command: &SignedCommand,
) -> Result<SignedCommandApplied, String> {
    within_mask(ledger.clone(), |ledger| {
        transaction_logic::apply_user_command(
            constraint_constants,
            txn_state_view,
            txn_global_slot,
            ledger,
            user_command,
        )
    })
}
