#![allow(unused)]

use mina_hasher::Fp;
use mina_signer::CompressedPubKey;

use crate::{
    scan_state::{
        scan_state::ConstraintConstants,
        transaction_logic::{protocol_state::GlobalStateSkeleton, TransactionFailure},
    },
    zkapps::intefaces::*,
    MyCow, TokenId,
};

use super::{
    witness::{Boolean, ToBoolean},
    zkapp::{Eff, StartDataSkeleton},
};

pub enum IsStart<T> {
    Yes(T),
    No,
    Compute(T),
}

pub enum PerformResult {
    // Bool(bool),
    // LocalState(LocalStateEnv<L>),
    // Account(Box<Account>),
}

pub struct Handler<Z: ZkappApplication> {
    pub perform: fn(Eff<Z>, &mut Z::WitnessGenerator) -> PerformResult,
}

pub struct GetNextAccountUpdateResult<Z: ZkappApplication> {
    pub account_update: Z::AccountUpdate,
    pub caller_id: TokenId,
    pub account_update_forest: Z::CallForest,
    pub new_call_stack: Z::CallStack,
    pub new_frame: Z::StackFrame,
}

fn assert_(_b: Boolean) -> Result<(), String> {
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

fn get_next_account_update<Z: ZkappApplication>(
    current_forest: Z::StackFrame,
    call_stack: Z::CallStack,
    w: &mut Z::WitnessGenerator,
) -> GetNextAccountUpdateResult<Z> {
    let (current_forest, call_stack) = {
        let (next_forest, next_call_stack) = pop_call_stack::<Z>(&call_stack, w);
        let current_is_empty = current_forest.calls().is_empty(w);
        let right = w.exists_no_check(match current_is_empty {
            Boolean::True => next_call_stack,
            Boolean::False => call_stack,
        });
        let left = match current_is_empty {
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
            w.exists_no_check(match remainder_of_current_forest_empty {
                Boolean::True => MyCow::Borrow(&call_stack),
                Boolean::False => MyCow::Own(on_false),
            })
        };
        let on_true = w.exists_no_check(match remainder_of_current_forest_empty {
            Boolean::True => MyCow::Borrow(&popped_call_stack),
            Boolean::False => MyCow::Borrow(&call_stack),
        });
        w.exists_no_check(match account_update_forest_empty {
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
            match remainder_of_current_forest_empty {
                Boolean::True => newly_popped_frame,
                Boolean::False => remainder_of_current_forest_frame,
            }
            .on_if(w)
        };
        match account_update_forest_empty {
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

#[derive(Debug, Clone)]
pub struct LocalState<Z: ZkappApplication> {
    pub stack_frame: Z::StackFrame,
    pub call_stack: Z::CallStack,
    pub transaction_commitment: Fp,
    pub full_transaction_commitment: Fp,
    pub excess: Z::SignedAmount,
    pub supply_increase: Z::SignedAmount,
    pub ledger: Z::Ledger,
    pub success: Boolean,
    pub account_update_index: Z::Index,
    pub failure_status_tbl: Z::FailureStatusTable,
    pub will_succeed: Boolean,
}

pub type GlobalState<Z> = GlobalStateSkeleton<
    <Z as ZkappApplication>::Ledger,                 // ledger
    <Z as ZkappApplication>::SignedAmount,           // fee_excess & supply_increase
    <Z as ZkappApplication>::GlobalSlotSinceGenesis, // block_global_slot
>;

pub type StartData<Z> = StartDataSkeleton<
    <Z as ZkappApplication>::CallForest, // account_updates
    Boolean,                             // will_succeed
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
            IsStart::Yes(_) => assert_(is_empty_call_forest)?,
            IsStart::No => assert_(is_empty_call_forest.neg())?,
        };
        match is_start {
            IsStart::Yes(_) => Boolean::True,
            IsStart::No => Boolean::False,
            IsStart::Compute(_) => is_empty_call_forest,
        }
    };

    let will_succeed = match &is_start {
        IsStart::Compute(start_data) => w.exists_no_check(match is_start2 {
            Boolean::True => start_data.will_succeed,
            Boolean::False => local_state.will_succeed,
        }),
        IsStart::Yes(start_data) => start_data.will_succeed,
        IsStart::No => local_state.will_succeed,
    };
    local_state.ledger = w.exists_no_check(match is_start2 {
        Boolean::True => global_state.first_pass_ledger(),
        Boolean::False => local_state.ledger.clone(),
    });
    local_state.will_succeed = will_succeed;

    let ((account_update, remaining, call_stack), account_update_forest, (mut a, inclusion_proof)) = {
        let (to_pop, call_stack) = {
            match &is_start {
                IsStart::Compute(start_data) => {
                    // We decompose this way because of OCaml evaluation order
                    let right = w.exists_no_check(match is_start2 {
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
                        match is_start2 {
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
                let tx_commitment = w.exists_no_check(match is_start2 {
                    Boolean::True => tx_commitment_on_start,
                    Boolean::False => local_state.transaction_commitment,
                });
                let full_tx_commitment = w.exists_no_check(match is_start2 {
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
            w.exists_no_check(match self_delegate {
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

    // let local_state =
    //   h.perform
    //     (Check_account_precondition
    //        (account_update, a, account_is_new, local_state) )
    // in
    // let protocol_state_predicate_satisfied =
    //   h.perform
    //     (Check_protocol_state_precondition
    //        ( Account_update.protocol_state_precondition account_update
    //        , global_state ) )
    // in
    // let local_state =
    //   Local_state.add_check local_state Protocol_state_precondition_unsatisfied
    //     protocol_state_predicate_satisfied
    // in
    // let local_state =
    //   let valid_while_satisfied =
    //     h.perform
    //       (Check_valid_while_precondition
    //          ( Account_update.valid_while_precondition account_update
    //          , global_state ) )
    //   in
    //   Local_state.add_check local_state Valid_while_precondition_unsatisfied
    //     valid_while_satisfied
    // in
    // let `Proof_verifies proof_verifies, `Signature_verifies signature_verifies =
    //   let commitment =
    //     Inputs.Transaction_commitment.if_
    //       (Inputs.Account_update.use_full_commitment account_update)
    //       ~then_:local_state.full_transaction_commitment
    //       ~else_:local_state.transaction_commitment
    //   in
    //   Inputs.Account_update.check_authorization
    //     ~will_succeed:local_state.will_succeed ~commitment
    //     ~calls:account_update_forest account_update
    // in

    eprintln!("DONE");
    std::process::exit(0);
}
