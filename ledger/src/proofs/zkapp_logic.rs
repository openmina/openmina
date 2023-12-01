#![allow(unused)]

use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

use crate::scan_state::transaction_logic::zkapp_command::{Actions, SetOrKeep};
use crate::{
    proofs::{numbers::nat::CheckedNat, witness::transaction_snark::CONSTRAINT_CONSTANTS},
    scan_state::{
        currency::{Amount, Magnitude, SlotSpan},
        scan_state::ConstraintConstants,
        transaction_logic::{
            protocol_state::GlobalStateSkeleton, zkapp_command::CheckAuthorizationResult,
            TransactionFailure,
        },
    },
    zkapps::intefaces::*,
    AuthRequired, AuthRequiredEncoded, MyCow, TokenId, VerificationKey,
};
use ark_ff::{One, Zero};

use super::{
    witness::{Boolean, ToBoolean},
    zkapp::{Eff, StartDataSkeleton, ZkappSingleData},
};

pub enum IsStart<T> {
    Yes(T),
    No,
    Compute(T),
}

pub enum PerformResult<Z: ZkappApplication> {
    None,
    Bool(Z::Bool),
    // Bool(bool),
    // LocalState(LocalStateEnv<L>),
    // Account(Box<Account>),
}

pub struct Handler<Z: ZkappApplication> {
    pub perform: fn(Eff<Z>, &mut Z::WitnessGenerator) -> PerformResult<Z>,
}

pub struct GetNextAccountUpdateResult<Z: ZkappApplication> {
    pub account_update: Z::AccountUpdate,
    pub caller_id: TokenId,
    pub account_update_forest: Z::CallForest,
    pub new_call_stack: Z::CallStack,
    pub new_frame: Z::StackFrame,
}

fn assert_<Z: ZkappApplication>(_b: Z::Bool) -> Result<(), String> {
    // Used only for circuit generation (add constraints)
    // https://github.com/MinaProtocol/mina/blob/e44ddfe1ca54b3855e1ed336d89f6230d35aeb8c/src/lib/transaction_logic/zkapp_command_logic.ml#L929

    // TODO: In non-witness generation, we raise an exception
    Ok(())
}

fn stack_frame_default<Z: ZkappApplication>(w: &mut Z::WitnessGenerator) -> Z::StackFrame {
    Z::StackFrame::make(
        StackFrameMakeParams {
            caller: TokenId::default(),
            caller_caller: TokenId::default(),
            calls: &Z::CallForest::empty(),
        },
        w,
    )
}

fn pop_call_stack<Z: ZkappApplication>(
    s: &Z::CallStack,
    w: &mut Z::WitnessGenerator,
) -> (Z::StackFrame, Z::CallStack) {
    let res = s.pop(w);
    let (next_frame, next_call_stack) = res.unzip();

    let right = w.exists_no_check(match next_call_stack.is_some {
        Boolean::True => next_call_stack.data,
        Boolean::False => Z::CallStack::empty(),
    });

    let on_none = stack_frame_default::<Z>(w);
    let left = match next_frame.is_some {
        Boolean::True => next_frame.data,
        Boolean::False => on_none,
    }
    .on_if(w);

    (left, right)
}

// We don't use `AuthRequired::to_field_elements`, because in OCaml `Controller.if_`
// push values in reverse order (because of OCaml evaluation order)
// https://github.com/MinaProtocol/mina/blob/4283d70c8c5c1bd9eebb0d3e449c36fb0bf0c9af/src/lib/mina_base/permissions.ml#L174
fn controller_exists<Z: ZkappApplication>(
    auth: AuthRequired,
    w: &mut Z::WitnessGenerator,
) -> AuthRequired {
    let AuthRequiredEncoded {
        constant,
        signature_necessary,
        signature_sufficient,
    } = auth.encode();

    w.exists_no_check([signature_sufficient, signature_necessary, constant]);
    auth
}

fn get_next_account_update<Z: ZkappApplication>(
    current_forest: Z::StackFrame,
    call_stack: Z::CallStack,
    w: &mut Z::WitnessGenerator,
) -> GetNextAccountUpdateResult<Z> {
    let (current_forest, call_stack) = {
        let (next_forest, next_call_stack) = pop_call_stack::<Z>(&call_stack, w);
        let current_is_empty = current_forest.calls().is_empty(w);
        let right = w.exists_no_check(match current_is_empty.as_boolean() {
            Boolean::True => next_call_stack,
            Boolean::False => call_stack,
        });
        let left = match current_is_empty.as_boolean() {
            Boolean::True => next_forest,
            Boolean::False => current_forest,
        }
        .on_if(w);
        (left, right)
    };

    let ((account_update, account_update_forest), remainder_of_current_forest) =
        current_forest.calls().pop_exn(w);

    let may_use_token = &account_update.body().may_use_token;
    let may_use_parents_own_token = may_use_token.parents_own_token().to_boolean();
    let may_use_token_inherited_from_parent = may_use_token.inherit_from_parent().to_boolean();

    let on_false = w.exists_no_check(match may_use_parents_own_token {
        Boolean::True => current_forest.caller(),
        Boolean::False => TokenId::default(),
    });
    let caller_id = w.exists_no_check(match may_use_token_inherited_from_parent {
        Boolean::True => current_forest.caller_caller(),
        Boolean::False => on_false,
    });

    let account_update_forest_empty = account_update_forest.is_empty(w);
    let remainder_of_current_forest_empty = remainder_of_current_forest.is_empty(w);
    let (newly_popped_frame, popped_call_stack) = pop_call_stack::<Z>(&call_stack, w);

    let remainder_of_current_forest_frame = Z::StackFrame::make(
        StackFrameMakeParams {
            caller: current_forest.caller(),
            caller_caller: current_forest.caller_caller(),
            calls: &remainder_of_current_forest,
        },
        w,
    );
    let new_call_stack = {
        let on_false = {
            let on_false = Z::CallStack::push(
                remainder_of_current_forest_frame.clone(),
                call_stack.clone(),
                w,
            );
            w.exists_no_check(match remainder_of_current_forest_empty.as_boolean() {
                Boolean::True => MyCow::Borrow(&call_stack),
                Boolean::False => MyCow::Own(on_false),
            })
        };
        let on_true = w.exists_no_check(match remainder_of_current_forest_empty.as_boolean() {
            Boolean::True => MyCow::Borrow(&popped_call_stack),
            Boolean::False => MyCow::Borrow(&call_stack),
        });
        w.exists_no_check(match account_update_forest_empty.as_boolean() {
            Boolean::True => on_true,
            Boolean::False => on_false,
        })
    };
    let new_frame = {
        let on_false = {
            let caller = Z::AccountId::derive_token_id(&account_update.body().account_id(), w);
            let caller_caller = caller_id.clone();
            Z::StackFrame::make(
                StackFrameMakeParams {
                    caller,
                    caller_caller,
                    calls: &account_update_forest,
                },
                w,
            )
        };
        let on_true = {
            match remainder_of_current_forest_empty.as_boolean() {
                Boolean::True => newly_popped_frame,
                Boolean::False => remainder_of_current_forest_frame,
            }
            .on_if(w)
        };
        match account_update_forest_empty.as_boolean() {
            Boolean::True => on_true,
            Boolean::False => on_false,
        }
        .on_if(w)
    };
    GetNextAccountUpdateResult {
        account_update,
        caller_id,
        account_update_forest,
        new_call_stack: new_call_stack.to_owned(),
        new_frame,
    }
}

fn update_action_state<Z: ZkappApplication>(
    action_state: &[Fp; 5],
    actions: &Actions,
    txn_global_slot: Z::GlobalSlotSinceGenesis,
    last_action_slot: Z::GlobalSlotSinceGenesis,
    w: &mut Z::WitnessGenerator,
) -> ([Fp; 5], <Z as ZkappApplication>::GlobalSlotSinceGenesis) {
    let [s1, s2, s3, s4, s5] = action_state.clone();
    let is_empty = Z::Actions::is_empty(actions, w);
    let s1_updated = Z::Actions::push_events(s1, actions, w);
    let s1_new = w.exists_no_check(match is_empty.as_boolean() {
        Boolean::True => s1,
        Boolean::False => s1_updated,
    });
    let is_this_slot = Z::GlobalSlotSinceGenesis::equal(&txn_global_slot, &last_action_slot, w);
    let is_empty_or_this_slot = Z::Bool::or(is_empty, is_this_slot, w);

    let s5 = w.exists_no_check(match is_empty_or_this_slot.as_boolean() {
        Boolean::True => s5,
        Boolean::False => s4,
    });
    let s4 = w.exists_no_check(match is_empty_or_this_slot.as_boolean() {
        Boolean::True => s4,
        Boolean::False => s3,
    });
    let s3 = w.exists_no_check(match is_empty_or_this_slot.as_boolean() {
        Boolean::True => s3,
        Boolean::False => s2,
    });
    let s2 = w.exists_no_check(match is_empty_or_this_slot.as_boolean() {
        Boolean::True => s2,
        Boolean::False => s1,
    });
    let last_action_slot = w.exists_no_check(match is_empty.as_boolean() {
        Boolean::True => last_action_slot,
        Boolean::False => txn_global_slot,
    });
    ([s1_new, s2, s3, s4, s5], last_action_slot)
}

#[derive(Debug, Clone)]
pub struct LocalState<Z: ZkappApplication> {
    pub stack_frame: Z::StackFrame,
    pub call_stack: Z::CallStack,
    pub transaction_commitment: Fp,
    pub full_transaction_commitment: Fp,
    pub excess: Z::SignedAmount,
    pub supply_increase: Z::SignedAmount,
    pub ledger: Z::Ledger,
    pub success: Z::Bool,
    pub account_update_index: Z::Index,
    pub failure_status_tbl: Z::FailureStatusTable,
    pub will_succeed: Z::Bool,
}

pub type GlobalState<Z> = GlobalStateSkeleton<
    <Z as ZkappApplication>::Ledger,                 // ledger
    <Z as ZkappApplication>::SignedAmount,           // fee_excess & supply_increase
    <Z as ZkappApplication>::GlobalSlotSinceGenesis, // block_global_slot
>;

pub type StartData<Z> = StartDataSkeleton<
    <Z as ZkappApplication>::CallForest, // account_updates
    <Z as ZkappApplication>::Bool,       // will_succeed
>;

pub fn apply<Z>(
    _constraint_constants: &ConstraintConstants,
    is_start: IsStart<StartData<Z>>,
    h: &Handler<Z>,
    (global_state, mut local_state): (Z::GlobalState, LocalState<Z>),
    data: Z::SingleData,
    w: &mut Z::WitnessGenerator,
) -> Result<(Z::GlobalState, LocalState<Z>), String>
where
    Z: ZkappApplication,
{
    let is_start2 = {
        let is_empty_call_forest = local_state.stack_frame.calls().is_empty(w);
        match is_start {
            IsStart::Compute(_) => (),
            IsStart::Yes(_) => assert_::<Z>(is_empty_call_forest)?,
            IsStart::No => assert_::<Z>(is_empty_call_forest.neg())?,
        };
        match is_start {
            IsStart::Yes(_) => Z::Bool::true_(),
            IsStart::No => Z::Bool::false_(),
            IsStart::Compute(_) => is_empty_call_forest,
        }
    };

    let will_succeed = match &is_start {
        IsStart::Compute(start_data) => w.exists_no_check(match is_start2.as_boolean() {
            Boolean::True => start_data.will_succeed,
            Boolean::False => local_state.will_succeed,
        }),
        IsStart::Yes(start_data) => start_data.will_succeed,
        IsStart::No => local_state.will_succeed,
    };
    local_state.ledger = w.exists_no_check(match is_start2.as_boolean() {
        Boolean::True => global_state.first_pass_ledger(),
        Boolean::False => local_state.ledger.clone(),
    });
    local_state.will_succeed = will_succeed;

    let ((account_update, remaining, call_stack), account_update_forest, (mut a, inclusion_proof)) = {
        let (to_pop, call_stack) = {
            match &is_start {
                IsStart::Compute(start_data) => {
                    // We decompose this way because of OCaml evaluation order
                    let right = w.exists_no_check(match is_start2.as_boolean() {
                        Boolean::True => Z::CallStack::empty(),
                        Boolean::False => local_state.call_stack.clone(),
                    });
                    let left = {
                        let on_true = Z::StackFrame::make(
                            StackFrameMakeParams {
                                caller: TokenId::default(),
                                caller_caller: TokenId::default(),
                                calls: &start_data.account_updates,
                            },
                            w,
                        );
                        match is_start2.as_boolean() {
                            Boolean::True => on_true,
                            Boolean::False => local_state.stack_frame.clone(),
                        }
                        .on_if(w)
                    };
                    (left, right)
                }
                IsStart::Yes(start_data) => {
                    // We decompose this way because of OCaml evaluation order
                    let right = Z::CallStack::empty();
                    let left = Z::StackFrame::make(
                        StackFrameMakeParams {
                            caller: TokenId::default(),
                            caller_caller: TokenId::default(),
                            calls: &start_data.account_updates,
                        },
                        w,
                    );
                    (left, right)
                }
                IsStart::No => (
                    local_state.stack_frame.clone(),
                    local_state.call_stack.clone(),
                ),
            }
        };

        let GetNextAccountUpdateResult {
            account_update,
            caller_id,
            account_update_forest,
            new_call_stack: call_stack,
            new_frame: remaining,
        } = get_next_account_update::<Z>(to_pop, call_stack, w);

        let _local_state = {
            let default_token_or_token_owner_was_caller = {
                let account_update_token_id = &account_update.body().token_id;
                let snd = Z::TokenId::equal(account_update_token_id, &caller_id, w);
                let fst = Z::TokenId::equal(account_update_token_id, &TokenId::default(), w);
                Z::Bool::or(fst, snd, w)
            };
            Z::LocalState::add_check(
                &mut local_state,
                TransactionFailure::TokenOwnerNotCaller,
                default_token_or_token_owner_was_caller,
                w,
            );
        };

        let acct = local_state.ledger.get_account(&account_update, w);
        local_state.ledger.check_inclusion(&acct, w);

        let (transaction_commitment, full_transaction_commitment) = match is_start {
            IsStart::No => (
                local_state.transaction_commitment,
                local_state.full_transaction_commitment,
            ),
            IsStart::Yes(start_data) | IsStart::Compute(start_data) => {
                let tx_commitment_on_start =
                    Z::TransactionCommitment::commitment(remaining.calls(), w);
                let full_tx_commitment_on_start = Z::TransactionCommitment::full_commitment(
                    &account_update,
                    start_data.memo_hash,
                    tx_commitment_on_start,
                    w,
                );
                let tx_commitment = w.exists_no_check(match is_start2.as_boolean() {
                    Boolean::True => tx_commitment_on_start,
                    Boolean::False => local_state.transaction_commitment,
                });
                let full_tx_commitment = w.exists_no_check(match is_start2.as_boolean() {
                    Boolean::True => full_tx_commitment_on_start,
                    Boolean::False => local_state.full_transaction_commitment,
                });
                (tx_commitment, full_tx_commitment)
            }
        };

        local_state.transaction_commitment = transaction_commitment;
        local_state.full_transaction_commitment = full_transaction_commitment;

        (
            (account_update, remaining, call_stack),
            account_update_forest,
            acct,
        )
    };

    local_state.stack_frame = remaining;
    local_state.call_stack = call_stack;
    Z::LocalState::add_new_failure_status_bucket(&mut local_state);

    a.register_verification_key(&data, w);

    let account_is_new = Z::Ledger::check_account(
        &account_update.body().public_key,
        &account_update.body().token_id,
        (&a, &inclusion_proof),
        w,
    );

    let _a = {
        let self_delegate = {
            let account_update_token_id = &account_update.body().token_id;
            let is_default_token =
                Z::TokenId::equal(account_update_token_id, &TokenId::default(), w);
            Z::Bool::and(account_is_new, is_default_token, w)
        };
        a.set_delegate(
            w.exists_no_check(match self_delegate.as_boolean() {
                Boolean::True => account_update.body().public_key.clone(),
                Boolean::False => a
                    .get()
                    .delegate
                    .clone()
                    .unwrap_or_else(CompressedPubKey::empty),
            }),
        )
    };

    let matching_verification_key_hashes = {
        let is_not_proved = account_update.is_proved().neg();
        let is_same_vk = Z::VerificationKeyHash::equal(
            a.verification_key_hash(),
            account_update.verification_key_hash(),
            w,
        );
        Z::Bool::or(is_not_proved, is_same_vk, w)
    };
    Z::LocalState::add_check(
        &mut local_state,
        TransactionFailure::UnexpectedVerificationKeyHash,
        matching_verification_key_hashes,
        w,
    );

    (h.perform)(
        Eff::CheckAccountPrecondition(&account_update, &a, account_is_new, &mut local_state),
        w,
    );

    let protocol_state_precondition = &account_update.body().preconditions.network;
    let PerformResult::Bool(protocol_state_predicate_satisfied) = (h.perform)(
        Eff::CheckProtocolStatePrecondition(protocol_state_precondition, &global_state),
        w,
    ) else {
        panic!("invalid state");
    };
    Z::LocalState::add_check(
        &mut local_state,
        TransactionFailure::ProtocolStatePreconditionUnsatisfied,
        protocol_state_predicate_satisfied,
        w,
    );

    let _local_state = {
        let valid_while = &account_update.body().preconditions.valid_while;
        let PerformResult::Bool(valid_while_satisfied) = (h.perform)(
            Eff::CheckValidWhilePrecondition(valid_while, &global_state),
            w,
        ) else {
            panic!("invalid state");
        };
        Z::LocalState::add_check(
            &mut local_state,
            TransactionFailure::ValidWhilePreconditionUnsatisfied,
            valid_while_satisfied,
            w,
        );
    };

    let CheckAuthorizationResult {
        proof_verifies,
        signature_verifies,
    } = {
        let use_full_commitment = account_update.body().use_full_commitment.to_boolean();
        let commitment = w.exists_no_check(match use_full_commitment {
            Boolean::True => local_state.full_transaction_commitment,
            Boolean::False => local_state.transaction_commitment,
        });
        account_update.check_authorization(
            local_state.will_succeed,
            commitment,
            &account_update_forest,
            &data,
            w,
        )
    };
    assert_::<Z>(Z::Bool::equal(
        proof_verifies,
        account_update.is_proved(),
        w,
    ));
    assert_::<Z>(Z::Bool::equal(
        signature_verifies,
        account_update.is_signed(),
        w,
    ));

    Z::LocalState::add_check(
        &mut local_state,
        TransactionFailure::FeePayerNonceMustIncrease,
        Z::Bool::or(account_update.increment_nonce(), is_start2.neg(), w),
        w,
    );
    Z::LocalState::add_check(
        &mut local_state,
        TransactionFailure::FeePayerMustBeSigned,
        Z::Bool::or(signature_verifies, is_start2.neg(), w),
        w,
    );

    let _local_state = {
        let precondition_has_constant_nonce =
            account_update.account_precondition_nonce_is_constant(w);
        let increments_nonce_and_constrains_its_old_value = Z::Bool::and(
            account_update.increment_nonce(),
            precondition_has_constant_nonce,
            w,
        );
        let depends_on_the_fee_payers_nonce_and_isnt_the_fee_payer =
            Z::Bool::and(account_update.use_full_commitment(), is_start2.neg(), w);
        let does_not_use_a_signature = signature_verifies.neg();
        let first = Z::Bool::or(
            increments_nonce_and_constrains_its_old_value,
            depends_on_the_fee_payers_nonce_and_isnt_the_fee_payer,
            w,
        );
        let second = Z::Bool::or(first, does_not_use_a_signature, w);
        Z::LocalState::add_check(
            &mut local_state,
            TransactionFailure::ZkappCommandReplayCheckFailed,
            second,
            w,
        );
    };

    a.set_token_id(account_update.body().token_id.clone());

    let account_update_token = &account_update.body().token_id;
    let account_update_token_is_default =
        Z::TokenId::equal(&TokenId::default(), account_update_token, w);
    let account_is_untimed = a.is_timed().neg();

    // Set account timing.
    let (_a, _local_state) = {
        let timing = &account_update.body().update.timing;
        let has_permission = {
            let set_timing = &a.get().permissions.set_timing;
            Z::Controller::check(proof_verifies, signature_verifies, set_timing, &data, w)
        };
        let is_keep = Z::SetOrKeep::is_keep(timing);
        let v_and = Z::Bool::and(account_is_untimed, has_permission, w);
        Z::LocalState::add_check(
            &mut local_state,
            TransactionFailure::UpdateNotPermittedTiming,
            Z::Bool::or(is_keep, v_and, w),
            w,
        );
        let timing = w.exists_no_check({
            use crate::scan_state::transaction_logic::zkapp_command::SetOrKeep;
            match timing {
                SetOrKeep::Set(timing) => timing.clone().to_account_timing(),
                SetOrKeep::Keep => a.get().timing.clone(),
            }
        });
        assert_::<Z>(Z::GlobalSlotSpan::greater_than(
            &timing.to_record().vesting_period,
            &SlotSpan::zero(),
            w,
        ));
        a.get_mut().timing = timing;
        ((), ())
    };
    let account_creation_fee =
        Z::Amount::of_constant_fee(CONSTRAINT_CONSTANTS.account_creation_fee);
    let implicit_account_creation_fee = account_update.implicit_account_creation_fee();
    Z::LocalState::add_check(
        &mut local_state,
        TransactionFailure::CannotPayCreationFeeInToken,
        Z::Bool::or(
            implicit_account_creation_fee.neg(),
            account_update_token_is_default,
            w,
        ),
        w,
    );

    // Compute the change to the account balance.
    let (_local_state, actual_balance_change) = {
        let balance_change = account_update.balance_change();
        let neg_creation_fee = { Z::SignedAmount::of_unsigned(account_creation_fee).negate() };
        // This `exists_no_check` exists because of this, we don't do it in `add_flagged`
        // because it can be executed or not, depending on the caller:
        // https://github.com/MinaProtocol/mina/blob/4283d70c8c5c1bd9eebb0d3e449c36fb0bf0c9af/src/lib/currency/currency.ml#L591
        w.exists_no_check(balance_change.value());
        let (balance_change_for_creation, creation_overflow) =
            Z::SignedAmount::add_flagged(&balance_change, &neg_creation_fee, w);
        let pay_creation_fee = Z::Bool::and(account_is_new, implicit_account_creation_fee, w);
        let creation_overflow = Z::Bool::and(pay_creation_fee, creation_overflow, w);
        let balance_change = w.exists_no_check(match pay_creation_fee.as_boolean() {
            Boolean::True => balance_change_for_creation,
            Boolean::False => balance_change,
        });
        // This 2nd `exists_no_check` is because of this:
        // https://github.com/MinaProtocol/mina/blob/03644c5748f76254c52a30c44f665bf19d1eb35b/src/lib/currency/currency.ml#L636
        w.exists_no_check(balance_change.value());
        let first = Z::Bool::or(
            creation_overflow,
            Z::SignedAmount::is_neg(&balance_change),
            w,
        );
        Z::LocalState::add_check(
            &mut local_state,
            TransactionFailure::AmountInsufficientToCreateAccount,
            Z::Bool::and(pay_creation_fee, first, w).neg(),
            w,
        );
        ((), balance_change)
    };

    // Apply balance change.
    let (_a, _local_state) = {
        let pay_creation_fee_from_excess =
            Z::Bool::and(account_is_new, implicit_account_creation_fee.neg(), w);
        let (balance, failed1) =
            Z::Balance::add_signed_amount_flagged(&a.balance(), actual_balance_change.clone(), w);
        Z::LocalState::add_check(
            &mut local_state,
            TransactionFailure::Overflow,
            failed1.neg(),
            w,
        );
        let account_creation_fee =
            Z::Amount::of_constant_fee(CONSTRAINT_CONSTANTS.account_creation_fee);
        let _local_state = {
            // This `exists_no_check` exists because of this, we don't do it in `add_flagged`
            // because it can be executed or not, depending on the caller:
            // https://github.com/MinaProtocol/mina/blob/4283d70c8c5c1bd9eebb0d3e449c36fb0bf0c9af/src/lib/currency/currency.ml#L591
            w.exists_no_check(local_state.excess.value());
            let (excess_minus_creation_fee, excess_update_failed) = Z::SignedAmount::add_flagged(
                &local_state.excess,
                &Z::SignedAmount::of_unsigned(account_creation_fee.clone()).negate(),
                w,
            );
            Z::LocalState::add_check(
                &mut local_state,
                TransactionFailure::LocalExcessOverflow,
                Z::Bool::and(pay_creation_fee_from_excess, excess_update_failed, w).neg(),
                w,
            );
            local_state.excess =
                w.exists_no_check(match pay_creation_fee_from_excess.as_boolean() {
                    Boolean::True => excess_minus_creation_fee,
                    Boolean::False => local_state.excess,
                });
            // This 2nd `exists_no_check` is because of this:
            // https://github.com/MinaProtocol/mina/blob/03644c5748f76254c52a30c44f665bf19d1eb35b/src/lib/currency/currency.ml#L636
            w.exists_no_check(local_state.excess.value());
        };

        let _local_state = {
            // This `exists_no_check` exists because of this, we don't do it in `add_flagged`
            // because it can be executed or not, depending on the caller:
            // https://github.com/MinaProtocol/mina/blob/4283d70c8c5c1bd9eebb0d3e449c36fb0bf0c9af/src/lib/currency/currency.ml#L591
            w.exists_no_check(local_state.supply_increase.value());
            let (supply_increase_minus_creation_fee, supply_increase_update_failed) =
                Z::SignedAmount::add_flagged(
                    &local_state.supply_increase,
                    &Z::SignedAmount::of_unsigned(account_creation_fee).negate(),
                    w,
                );
            Z::LocalState::add_check(
                &mut local_state,
                TransactionFailure::LocalSupplyIncreaseOverflow,
                Z::Bool::and(account_is_new, supply_increase_update_failed, w).neg(),
                w,
            );
            local_state.supply_increase = w.exists_no_check(match account_is_new.as_boolean() {
                Boolean::True => supply_increase_minus_creation_fee,
                Boolean::False => local_state.supply_increase,
            });
            // This 2nd `exists_no_check` is because of this:
            // https://github.com/MinaProtocol/mina/blob/03644c5748f76254c52a30c44f665bf19d1eb35b/src/lib/currency/currency.ml#L636
            w.exists_no_check(local_state.supply_increase.value());
        };

        let is_receiver = actual_balance_change.is_non_neg();
        let _local_state = {
            let controller = controller_exists::<Z>(
                match is_receiver.as_boolean() {
                    Boolean::True => a.get().permissions.receive,
                    Boolean::False => a.get().permissions.send,
                },
                w,
            );
            let has_permission =
                Z::Controller::check(proof_verifies, signature_verifies, &controller, &data, w);
            let first = Z::SignedAmount::equal(&Z::SignedAmount::zero(), &actual_balance_change, w);
            Z::LocalState::add_check(
                &mut local_state,
                TransactionFailure::UpdateNotPermittedBalance,
                Z::Bool::or(has_permission, first, w),
                w,
            );
        };
        Z::Account::set_balance(&mut a, balance);
        ((), ())
    };

    let txn_global_slot = global_state.block_global_slot();
    // Check timing with current balance
    let (_a, _local_state) = {
        let (invalid_timing, timing) = Z::Account::check_timing(&a, &txn_global_slot, w);
        Z::LocalState::add_check(
            &mut local_state,
            TransactionFailure::SourceMinimumBalanceViolation,
            invalid_timing.neg(),
            w,
        );
        a.get_mut().timing = timing;
        ((), ())
    };
    Z::Account::make_zkapp(&mut a);
    // Check that the account can be accessed with the given authorization.
    let _local_state = {
        let has_permission = {
            let access = &a.get().permissions.access;
            Z::Controller::check(proof_verifies, signature_verifies, access, &data, w)
        };
        Z::LocalState::add_check(
            &mut local_state,
            TransactionFailure::UpdateNotPermittedAccess,
            has_permission,
            w,
        );
    };

    // Update app state.
    let (_a, _local_state) = {
        let app_state = &account_update.body().update.app_state;
        let keeping_app_state = {
            let is_all_keep: [_; 8] = std::array::from_fn(|i| Z::SetOrKeep::is_keep(&app_state[i]));
            assert_eq!(is_all_keep.len(), app_state.len()); // TODO: Use `array::each_ref` when stable
            Z::Bool::all(&is_all_keep, w)
        };
        let changing_entire_app_state = {
            let is_all_set: [_; 8] = std::array::from_fn(|i| Z::SetOrKeep::is_set(&app_state[i]));
            assert_eq!(is_all_set.len(), app_state.len()); // TODO: Use `array::each_ref` when stable
            Z::Bool::all(&is_all_set, w)
        };
        let proved_state = {
            let on_false = {
                let on_true = {
                    w.exists_no_check(match changing_entire_app_state.as_boolean() {
                        Boolean::True => Z::Bool::true_(),
                        Boolean::False => a.proved_state(),
                    })
                };
                w.exists_no_check_on_bool(
                    proof_verifies,
                    match proof_verifies.as_boolean() {
                        Boolean::True => on_true,
                        Boolean::False => Z::Bool::false_(),
                    },
                )
            };
            w.exists_no_check(match keeping_app_state.as_boolean() {
                Boolean::True => a.proved_state(),
                Boolean::False => on_false,
            })
        };
        a.set_proved_state(proved_state);
        let has_permission = {
            let edit_state = &a.get().permissions.edit_state;
            Z::Controller::check(proof_verifies, signature_verifies, edit_state, &data, w)
        };
        Z::LocalState::add_check(
            &mut local_state,
            TransactionFailure::UpdateNotPermittedAppState,
            Z::Bool::or(keeping_app_state, has_permission, w),
            w,
        );
        let app_state: [Fp; 8] = app_state
            .iter()
            .zip(a.app_state())
            .map(|(set_or_keep, state)| {
                w.exists_no_check(match set_or_keep {
                    SetOrKeep::Set(s) => *s,
                    SetOrKeep::Keep => state,
                })
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        // `unwrap`: We called `make_zkapp` before
        a.get_mut().zkapp.as_mut().unwrap().app_state = app_state;
        ((), ())
    };

    // Set verification key.
    let (_a, _local_state) = {
        let verification_key = &account_update.body().update.verification_key;
        let has_permission = {
            let set_verification_key = &a.get().permissions.set_verification_key;
            Z::Controller::check(
                proof_verifies,
                signature_verifies,
                set_verification_key,
                &data,
                w,
            )
        };
        Z::LocalState::add_check(
            &mut local_state,
            TransactionFailure::UpdateNotPermittedVerificationKey,
            Z::Bool::or(Z::SetOrKeep::is_keep(verification_key), has_permission, w),
            w,
        );
        // `unwrap`: We called `make_zkapp` before
        let zkapp = a.get().zkapp.as_ref().unwrap();
        w.exists_no_check(match verification_key {
            SetOrKeep::Set(key) => key.hash,
            SetOrKeep::Keep => {
                MyCow::borrow_or_else(&zkapp.verification_key, VerificationKey::dummy).hash()
            }
        });
        // Made here https://github.com/MinaProtocol/mina/blob/5c92fbdbf083a74a8b9530d3d727cc7b03dcce8a/src/lib/mina_base/zkapp_basic.ml#L82
        w.exists_no_check(verification_key.is_set());
        let verification_key = match verification_key {
            SetOrKeep::Set(vk) => Some(vk.data.clone()),
            SetOrKeep::Keep => zkapp.verification_key.clone(),
        };
        // `unwrap`: We called `make_zkapp` before
        a.get_mut().zkapp.as_mut().unwrap().verification_key = verification_key;
        ((), ())
    };

    // Update action state.
    let (_a, _local_state) = {
        let actions = &account_update.body().actions;
        let last_action_slot = a.last_action_slot();
        let action_state = &a.get().zkapp.as_ref().unwrap().action_state;
        let (action_state, last_action_slot) =
            update_action_state::<Z>(action_state, actions, txn_global_slot, last_action_slot, w);
        let is_empty = Z::Actions::is_empty(actions, w);
        let has_permission = {
            let edit_action_state = &a.get().permissions.edit_action_state;
            Z::Controller::check(
                proof_verifies,
                signature_verifies,
                edit_action_state,
                &data,
                w,
            )
        };
        Z::LocalState::add_check(
            &mut local_state,
            TransactionFailure::UpdateNotPermittedActionState,
            Z::Bool::or(is_empty, has_permission, w),
            w,
        );
        // `unwrap`: We called `make_zkapp` before
        a.get_mut().zkapp.as_mut().unwrap().action_state = action_state;
        Z::Account::set_last_action_slot(&mut a, last_action_slot);
        ((), ())
    };

    // Update zkApp URI.
    let (_a, _local_state) = {
        let zkapp_uri = &account_update.body().update.zkapp_uri;
        let has_permission = {
            let set_zkapp_uri = &a.get().permissions.set_zkapp_uri;
            Z::Controller::check(proof_verifies, signature_verifies, set_zkapp_uri, &data, w)
        };
        Z::LocalState::add_check(
            &mut local_state,
            TransactionFailure::UpdateNotPermittedZkappUri,
            Z::Bool::or(Z::SetOrKeep::is_keep(zkapp_uri), has_permission, w),
            w,
        );
        let zkapp = a.zkapp();
        let zkapp_uri = w.exists_no_check(match zkapp_uri {
            SetOrKeep::Set(zkapp_uri) => Some(zkapp_uri),
            SetOrKeep::Keep => Some(&zkapp.zkapp_uri),
        });
        // `unwrap`: We called `make_zkapp` before
        a.get_mut().zkapp.as_mut().unwrap().zkapp_uri = zkapp_uri.cloned().unwrap();
        ((), ())
    };

    Z::Account::unmake_zkapp(&mut a);

    // Update token symbol.
    let (_a, _local_state) = {
        let token_symbol = &account_update.body().update.token_symbol;
        let has_permission = {
            let set_token_symbol = &a.get().permissions.set_token_symbol;
            Z::Controller::check(
                proof_verifies,
                signature_verifies,
                set_token_symbol,
                &data,
                w,
            )
        };
        Z::LocalState::add_check(
            &mut local_state,
            TransactionFailure::UpdateNotPermittedTokenSymbol,
            Z::Bool::or(Z::SetOrKeep::is_keep(token_symbol), has_permission, w),
            w,
        );
        let token_symbol = w.exists_no_check({
            match token_symbol {
                SetOrKeep::Set(token_symbol) => token_symbol.clone(),
                SetOrKeep::Keep => a.get().token_symbol.clone(),
            }
        });
        a.get_mut().token_symbol = token_symbol;
        ((), ())
    };

    eprintln!("DONE");
    std::process::exit(0);
}
