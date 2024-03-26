use ark_ff::Zero;
use mina_hasher::Fp;
use mina_signer::CompressedPubKey;
use openmina_core::constants::ConstraintConstants;

use crate::{
    check_permission, hash_with_kimchi,
    scan_state::{
        currency::{Amount, Index, Magnitude, Sgn, Signed, Slot},
        transaction_logic::{
            account_check_timing, cons_zkapp_command_commitment, get_account, is_timed,
            local_state::{CallStack, LocalStateEnv, StackFrame},
            protocol_state::GlobalState,
            set_account,
            zkapp_command::{
                self, AccountUpdate, CallForest, CheckAuthorizationResult, OrIgnore, SetOrKeep,
            },
            Env, TimingValidation, TransactionFailure,
        },
    },
    sparse_ledger::LedgerIntf,
    Account, AuthRequired, ControlTag, Mask, ReceiptChainHash, SetVerificationKey, Timing, TokenId,
    ZkAppAccount, TXN_VERSION_CURRENT,
};

use super::{
    currency::SlotSpan,
    transaction_logic::{
        zkapp_command::{Actions, ACCOUNT_UPDATE_CONS_HASH_PARAM},
        Eff, ExistingOrNew, PerformResult,
    },
};

/*
    In the OCaml code "asserts" are used to raise an "Assert_failure" exception that is
    catched and turn into an error code. We will mimic a similar behaviour using the Result
    type and an "__assert" macro.
    This code won't panic!
*/

macro_rules! __assert {
    ($cond:expr $(,)?) => {{
        if !$cond {
            let file = file!();
            let line = line!();
            return Err(format!("Assert_failure {file}:{line}"));
        }
        Ok::<(), String>(())
    }};
}

pub struct StartData {
    pub account_updates: CallForest<AccountUpdate>,
    pub memo_hash: Fp,
    pub will_succeed: bool,
}

pub enum IsStart {
    Yes(StartData),
    No,
    Compute(StartData),
}

pub struct Handler<L: LedgerIntf + Clone> {
    pub perform: fn(Eff<L>) -> PerformResult<L>,
}

pub fn commitment(account_updates: CallForest<AccountUpdate>) -> ReceiptChainHash {
    ReceiptChainHash(account_updates.hash())
}

pub fn full_commitment(
    account_update: AccountUpdate,
    memo_hash: Fp,
    commitment: ReceiptChainHash,
) -> ReceiptChainHash {
    let fee_payer_hash = account_update.digest();
    ReceiptChainHash(hash_with_kimchi(
        ACCOUNT_UPDATE_CONS_HASH_PARAM,
        &[memo_hash, fee_payer_hash, commitment.0],
    ))
}

pub fn controller_check(
    proof_verifies: bool,
    signature_verifies: bool,
    perm: AuthRequired,
) -> Result<bool, String> {
    __assert!(!(proof_verifies && signature_verifies))?;
    let tag = if proof_verifies {
        ControlTag::Proof
    } else if signature_verifies {
        ControlTag::Signature
    } else {
        ControlTag::NoneGiven
    };
    Ok(check_permission(perm, tag))
}

#[derive(Clone)]
pub enum ZkAppCommandElt {
    ZkAppCommandCommitment(ReceiptChainHash),
}

fn assert_with_failure_status_tbl(
    b: bool,
    failure_status_tbl: Vec<Vec<TransactionFailure>>,
) -> Result<(), String> {
    if !b && !(failure_status_tbl.is_empty()) {
        Err(format!("{:?}", failure_status_tbl))
    } else {
        __assert!(b)
    }
}

// https://github.com/MinaProtocol/mina/blob/32a91613c388a71f875581ad72276e762242f802/src/lib/mina_ledger/ledger.ml#L211
fn empty_ledger(depth: usize) -> Mask {
    Mask::new_unattached(depth)
    //mask.set_parent(parent, None)
}

fn pop_call_stack(s: &CallStack) -> (StackFrame, CallStack) {
    if let Some(a) = s.pop() {
        a
    } else {
        (StackFrame::default(), CallStack::new())
    }
}

/// https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/transaction_logic/mina_transaction_logic.ml#L1545
fn account_verification_key_hash(account: &Account) -> Option<Fp> {
    Some(account.zkapp.as_ref()?.verification_key.as_ref()?.hash())
}

pub struct GetNextAccountUpdateResult {
    pub account_update: AccountUpdate,
    pub caller_id: TokenId,
    pub account_update_forest: CallForest<AccountUpdate>,
    pub new_call_stack: CallStack,
    pub new_frame: StackFrame,
}

pub fn get_next_account_update(
    current_forest: StackFrame,
    call_stack: CallStack,
) -> Result<GetNextAccountUpdateResult, String> {
    let (next_forest, next_call_stack) = pop_call_stack(&call_stack);
    let (current_forest, call_stack) = if current_forest.calls.is_empty() {
        (next_forest, next_call_stack)
    } else {
        (current_forest, call_stack)
    };

    let ((account_update, account_update_forest), remainder_of_current_forest) =
        current_forest.calls.pop_exn();

    let may_use_parents_own_token = account_update.may_use_parents_own_token();
    let may_use_token_inherited_from_parent = account_update.may_use_token_inherited_from_parent();

    let caller_id = if may_use_token_inherited_from_parent {
        current_forest.caller_caller.clone()
    } else if may_use_parents_own_token {
        current_forest.caller.clone()
    } else {
        TokenId::default()
    };

    // Cases:
    //  - [account_update_forest] is empty, [remainder_of_current_forest] is empty.
    //  Pop from the call stack to get another forest, which is guaranteed to be non-empty.
    //  The result of popping becomes the "current forest".
    //  - [account_update_forest] is empty, [remainder_of_current_forest] is non-empty.
    //  Push nothing to the stack. [remainder_of_current_forest] becomes new "current forest"
    //  - [account_update_forest] is non-empty, [remainder_of_current_forest] is empty.
    //  Push nothing to the stack. [account_update_forest] becomes new "current forest"
    //  - [account_update_forest] is non-empty, [remainder_of_current_forest] is non-empty:
    //  Push [remainder_of_current_forest] to the stack. [account_update_forest] becomes new "current forest".

    let account_update_forest_empty = account_update_forest.is_empty();
    let remainder_of_current_forest_empty = remainder_of_current_forest.is_empty();
    let (newly_popped_frame, popped_call_stack) = pop_call_stack(&call_stack);
    let remainder_of_current_forest_frame = StackFrame {
        caller: current_forest.caller.clone(),
        caller_caller: current_forest.caller_caller,
        calls: remainder_of_current_forest,
    };

    let new_call_stack = if account_update_forest_empty {
        if remainder_of_current_forest_empty {
            popped_call_stack
        } else {
            call_stack
        }
    } else if remainder_of_current_forest_empty {
        call_stack
    } else {
        call_stack.push(&remainder_of_current_forest_frame)
    };

    let new_frame = if account_update_forest_empty {
        if remainder_of_current_forest_empty {
            newly_popped_frame
        } else {
            remainder_of_current_forest_frame
        }
    } else {
        let caller = account_update.account_id().derive_token_id();
        let caller_caller = caller_id.clone();

        StackFrame {
            caller,
            caller_caller,
            calls: account_update_forest.clone(),
        }
    };
    Ok(GetNextAccountUpdateResult {
        account_update,
        account_update_forest,
        new_frame,
        new_call_stack,
        caller_id,
    })
}

pub fn check_account<L>(
    public_key: CompressedPubKey,
    token_id: TokenId,
    (account, loc): (&Account, &ExistingOrNew<L::Location>),
) -> Result<bool, String>
where
    L: LedgerIntf,
{
    __assert!(public_key == account.public_key)?;
    __assert!(token_id == account.token_id)?;
    // IsNew?
    match loc {
        ExistingOrNew::Existing(_) => Ok(false),
        ExistingOrNew::New => Ok(true),
    }
}

pub fn make_zkapp(a: Account) -> Account {
    let zkapp = if let zkapp @ Some(_) = a.zkapp {
        zkapp
    } else {
        Some(ZkAppAccount::default())
    };
    Account { zkapp, ..a }
}

pub fn update_action_state(
    action_state: [Fp; 5],
    actions: Actions,
    txn_global_slot: Slot,
    last_action_slot: Slot,
) -> ([Fp; 5], Slot) {
    let [_s1, _s2, _s3, _s4, _s5] = action_state;
    let is_empty = actions.is_empty();
    let s1_updated = actions.push_events(_s1);
    let s1 = if is_empty { _s1 } else { s1_updated };
    let is_this_slot = txn_global_slot == last_action_slot;
    let is_empty_or_this_slot = is_empty || is_this_slot;
    let (s5, s4, s3, s2) = if is_empty_or_this_slot {
        (_s5, _s4, _s3, _s2)
    } else {
        (_s4, _s3, _s2, _s1)
    };
    let last_action_slot = if is_empty {
        last_action_slot
    } else {
        txn_global_slot
    };
    ([s1, s2, s3, s4, s5], last_action_slot)
}

pub fn unmake_zkapp(a: Account) -> Account {
    let zkapp = a.zkapp.filter(|zkapp| &ZkAppAccount::default() != zkapp);
    Account { zkapp, ..a }
}

pub fn apply<L>(
    constraint_constants: &ConstraintConstants,
    is_start: IsStart,
    _h: &Handler<L>,
    (global_state, local_state): (GlobalState<L>, LocalStateEnv<L>),
) -> Result<(GlobalState<L>, LocalStateEnv<L>), String>
where
    L: LedgerIntf + Clone,
{
    let is_empty_call_forest = local_state.stack_frame.calls.is_empty();

    match is_start {
        IsStart::Compute(_) => (),
        IsStart::Yes(_) => __assert!(is_empty_call_forest)?,
        IsStart::No => __assert!(!is_empty_call_forest)?,
    };

    let is_start_ = match is_start {
        IsStart::Yes(_) => true,
        IsStart::No => false,
        IsStart::Compute(_) => is_empty_call_forest,
    };

    let will_succeed = match &is_start {
        IsStart::Compute(start_data) => {
            if is_start_ {
                start_data.will_succeed
            } else {
                local_state.will_succeed
            }
        }
        IsStart::Yes(start_data) => start_data.will_succeed,
        IsStart::No => local_state.will_succeed,
    };

    let mut local_state = local_state;

    if is_start_ {
        local_state.ledger = global_state.first_pass_ledger();
    }
    local_state.will_succeed = will_succeed;

    let (
        (account_update, remaining, call_stack),
        account_update_forest,
        local_state,
        (a, inclusion_proof),
    ) = {
        let (to_pop, call_stack) = match &is_start {
            IsStart::Compute(start_data) => {
                if is_start_ {
                    (
                        StackFrame {
                            caller: TokenId::default(),
                            caller_caller: TokenId::default(),
                            calls: start_data.account_updates.clone(),
                        },
                        CallStack::new(),
                    )
                } else {
                    (
                        local_state.stack_frame.clone(),
                        local_state.call_stack.clone(),
                    )
                }
            }
            IsStart::Yes(start_data) => (
                StackFrame {
                    caller: TokenId::default(),
                    caller_caller: TokenId::default(),
                    calls: start_data.account_updates.clone(),
                },
                CallStack::new(),
            ),
            IsStart::No => (
                local_state.stack_frame.clone(),
                local_state.call_stack.clone(),
            ),
        };

        let GetNextAccountUpdateResult {
            account_update,
            account_update_forest,
            new_frame: remaining,
            new_call_stack: call_stack,
            caller_id,
        } = get_next_account_update(to_pop, call_stack)?;

        let mut local_state = local_state.add_check(
            TransactionFailure::TokenOwnerNotCaller,
            account_update.token_id() == TokenId::default()
                || account_update.token_id() == caller_id,
        );

        let (a, inclusion_proof) =
            get_account(&mut local_state.ledger, account_update.account_id());

        let acct = (a, inclusion_proof);

        let (transaction_commitment, full_transaction_commitment) = match &is_start {
            IsStart::No => (
                local_state.transaction_commitment,
                local_state.full_transaction_commitment,
            ),
            IsStart::Yes(start_data) | IsStart::Compute(start_data) => {
                let tx_commitment_on_start = commitment(remaining.calls.clone());
                let full_tx_commitment_on_start = full_commitment(
                    account_update.clone(),
                    start_data.memo_hash,
                    tx_commitment_on_start.clone(),
                );
                if is_start_ {
                    (tx_commitment_on_start, full_tx_commitment_on_start)
                } else {
                    (
                        local_state.transaction_commitment,
                        local_state.full_transaction_commitment,
                    )
                }
            }
        };

        let local_state = LocalStateEnv {
            transaction_commitment,
            full_transaction_commitment,
            ..local_state
        };
        (
            (account_update, remaining, call_stack),
            account_update_forest,
            local_state,
            acct,
        )
    };

    let local_state = LocalStateEnv {
        stack_frame: remaining.clone(),
        call_stack,
        ..local_state
    };
    let local_state = local_state.add_new_failure_status_bucket();
    let account_is_new = check_account::<L>(
        account_update.public_key(),
        account_update.token_id(),
        (&a, &inclusion_proof),
    )?;

    // delegate to public key if new account using default token
    let a = {
        let self_delegate = {
            let account_update_token_id = account_update.token_id();
            account_is_new && account_update_token_id.is_default()
        };
        // in-SNARK, a new account has the empty public key here
        // in that case, use the public key from the account update, not the account

        let delegate = if self_delegate {
            account_update.public_key()
        } else {
            a.delegate.unwrap_or_else(CompressedPubKey::empty)
        };

        let delegate = if delegate == CompressedPubKey::empty() {
            None
        } else {
            Some(delegate)
        };

        Account { delegate, ..*a }
    };

    let matching_verification_key_hashes = !(account_update.is_proved())
        || account_verification_key_hash(&a) == account_update.verification_key_hash();

    let local_state = local_state.add_check(
        TransactionFailure::UnexpectedVerificationKeyHash,
        matching_verification_key_hashes,
    );

    let PerformResult::LocalState(local_state) = Env::perform(Eff::CheckAccountPrecondition(
        account_update.clone(),
        a.clone(),
        account_is_new,
        local_state,
    )) else {
        unreachable!()
    };

    let protocol_state_predicate_satisfied =
        if let PerformResult::Bool(protocol_state_predicate_satisfied) =
            Env::perform(Eff::CheckProtocolStatePrecondition(
                account_update.protocol_state_precondition(),
                global_state.clone(),
            ))
        {
            protocol_state_predicate_satisfied
        } else {
            unreachable!()
        };

    let local_state = local_state.add_check(
        TransactionFailure::ProtocolStatePreconditionUnsatisfied,
        protocol_state_predicate_satisfied,
    );

    let local_state = {
        let valid_while_satisfied = Env::perform(Eff::CheckValidWhilePrecondition(
            account_update.valid_while_precondition(),
            global_state.clone(),
        ))
        .to_bool();

        local_state.add_check(
            TransactionFailure::ValidWhilePreconditionUnsatisfied,
            valid_while_satisfied,
        )
    };

    let CheckAuthorizationResult {
        proof_verifies,
        signature_verifies,
    } = {
        let commitment = if account_update.use_full_commitment() {
            local_state.full_transaction_commitment.clone()
        } else {
            local_state.transaction_commitment.clone()
        };
        account_update.check_authorization(
            local_state.will_succeed,
            commitment.0,
            account_update_forest,
        )
    };

    __assert!(proof_verifies == account_update.is_proved())?;
    __assert!(signature_verifies == account_update.is_signed())?;

    let local_state = local_state.add_check(
        TransactionFailure::FeePayerNonceMustIncrease,
        account_update.increment_nonce() || !is_start_,
    );
    let local_state = local_state.add_check(
        TransactionFailure::FeePayerMustBeSigned,
        signature_verifies || !is_start_,
    );

    let precondition_has_constant_nonce = match account_update.account_precondition().nonce() {
        OrIgnore::Check(x) => x.is_constant(),
        OrIgnore::Ignore => false,
    };
    let increments_nonce_and_constrains_its_old_value =
        account_update.increment_nonce() && precondition_has_constant_nonce;
    let depends_on_the_fee_payers_nonce_and_isnt_the_fee_payer =
        account_update.use_full_commitment() && !is_start_;
    let does_not_use_a_signature = !signature_verifies;

    let local_state = local_state.add_check(
        TransactionFailure::ZkappCommandReplayCheckFailed,
        increments_nonce_and_constrains_its_old_value
            || depends_on_the_fee_payers_nonce_and_isnt_the_fee_payer
            || does_not_use_a_signature,
    );
    let a = Account {
        token_id: account_update.token_id(),
        ..a
    };
    let account_update_token_is_default = account_update.token_id() == TokenId::default();
    let account_is_untimed = !is_timed(&a);

    let timing = account_update.timing();
    let has_permission =
        controller_check(proof_verifies, signature_verifies, a.permissions.set_timing)?;

    let local_state = local_state.add_check(
        TransactionFailure::UpdateNotPermittedTiming,
        timing.is_keep() || (account_is_untimed && has_permission),
    );
    let timing = timing
        .into_map(Some)
        .set_or_keep(zkapp_command::Timing::of_account_timing(a.timing.clone()));

    // https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/transaction_logic/mina_transaction_logic.ml#L1197
    let vesting_period = match &timing {
        Some(timing) => timing.vesting_period,
        None => Timing::Untimed.to_record().vesting_period,
    };

    __assert!(vesting_period > SlotSpan::zero())?;

    let a = Account {
        timing: timing
            .map(|timing| timing.to_account_timing())
            .unwrap_or(Timing::Untimed),
        ..a
    };

    let account_creation_fee = Amount::from_u64(constraint_constants.account_creation_fee);
    let implicit_account_creation_fee = account_update.implicit_account_creation_fee();

    // Check the token for implicit account creation fee payment.
    let local_state = local_state.add_check(
        TransactionFailure::CannotPayCreationFeeInToken,
        (!implicit_account_creation_fee) || account_update_token_is_default,
    );

    // Compute the change to the account balance.
    let (local_state, actual_balance_change) = {
        let balance_change = account_update.balance_change();
        let neg_creation_fee = Signed::of_unsigned(account_creation_fee).negate();

        let (balance_change_for_creation, creation_overflow) =
            balance_change.add_flagged(neg_creation_fee);

        let pay_creation_fee = account_is_new && implicit_account_creation_fee;
        let creation_overflow = pay_creation_fee && creation_overflow;

        let balance_change = if pay_creation_fee {
            balance_change_for_creation
        } else {
            balance_change
        };

        let local_state = local_state.add_check(
            TransactionFailure::AmountInsufficientToCreateAccount,
            !(pay_creation_fee && (creation_overflow || balance_change.is_neg())),
        );

        (local_state, balance_change)
    };

    let (a, local_state) = {
        let pay_creation_fee_from_excess = account_is_new && !implicit_account_creation_fee;
        let (balance, failed1) = a.balance.add_signed_amount_flagged(actual_balance_change);

        // println!("[rust] failed1 {}", failed1);
        let local_state = local_state.add_check(TransactionFailure::Overflow, !failed1);
        let local_state = {
            let (excess_minus_creation_fee, excess_update_failed) =
                local_state.excess.add_flagged(Signed::<Amount> {
                    magnitude: Amount::from_u64(constraint_constants.account_creation_fee),
                    sgn: Sgn::Neg,
                });
            let local_state = local_state.add_check(
                TransactionFailure::LocalExcessOverflow,
                !(pay_creation_fee_from_excess && excess_update_failed),
            );
            LocalStateEnv {
                excess: if pay_creation_fee_from_excess {
                    excess_minus_creation_fee
                } else {
                    local_state.excess
                },
                ..local_state
            }
        };
        let local_state = {
            // Conditionally subtract account creation fee from supply increase
            let (supply_increase_minus_creation_fee, supply_increase_update_failed) = local_state
                .supply_increase
                .add_flagged(Signed::of_unsigned(account_creation_fee).negate());
            let local_state = local_state.add_check(
                TransactionFailure::LocalSupplyIncreaseOverflow,
                !(account_is_new && supply_increase_update_failed),
            );
            LocalStateEnv {
                supply_increase: if account_is_new {
                    supply_increase_minus_creation_fee
                } else {
                    local_state.supply_increase
                },
                ..local_state
            }
        };
        let is_receiver = actual_balance_change.is_non_neg();
        let local_state = {
            let controller = if is_receiver {
                a.permissions.receive
            } else {
                a.permissions.send
            };

            let has_permission = controller_check(proof_verifies, signature_verifies, controller)?;

            local_state.add_check(
                TransactionFailure::UpdateNotPermittedBalance,
                has_permission || actual_balance_change.is_zero(),
            )
        };
        let a = Account { balance, ..a };
        (a, local_state)
    };
    let txn_global_slot = global_state.block_global_slot;
    let (a, local_state) = {
        let (invalid_timing, timing) = match account_check_timing(&txn_global_slot, &a) {
            (TimingValidation::InsufficientBalance(true), _) => {
                panic!("Did not propose a balance change at this timing check!")
            }
            (TimingValidation::InvalidTiming(true), timing) => (true, timing),
            (_, timing) => (false, timing),
        };
        let local_state = local_state.add_check(
            TransactionFailure::SourceMinimumBalanceViolation,
            !invalid_timing,
        );
        let a = Account { timing, ..a };
        (a, local_state)
    };
    let a = make_zkapp(a);

    // Check that the account can be accessed with the given authorization.
    let local_state = {
        let has_permission =
            controller_check(proof_verifies, signature_verifies, a.permissions.access)?;
        local_state.add_check(TransactionFailure::UpdateNotPermittedAccess, has_permission)
    };

    let app_state = account_update.app_state();
    let keeping_app_state = app_state.iter().all(|x| x.is_keep());
    let changing_entire_app_state = app_state.iter().all(|x| x.is_set());
    let zkapp = a.zkapp.unwrap();

    let proved_state = if keeping_app_state {
        zkapp.proved_state
    } else if proof_verifies {
        if changing_entire_app_state {
            true
        } else {
            zkapp.proved_state
        }
    } else {
        false
    };
    let zkapp = ZkAppAccount {
        proved_state,
        ..zkapp
    };
    let a = Account {
        zkapp: Some(zkapp.clone()),
        ..a
    };
    let has_permission =
        controller_check(proof_verifies, signature_verifies, a.permissions.edit_state)?;

    let local_state = local_state.add_check(
        TransactionFailure::UpdateNotPermittedAppState,
        keeping_app_state || has_permission,
    );
    let app_state: Vec<Fp> = app_state
        .iter()
        .zip(zkapp.app_state.iter())
        .map(|(x, y)| x.set_or_keep(*y))
        .collect();
    let app_state = [
        app_state[0],
        app_state[1],
        app_state[2],
        app_state[3],
        app_state[4],
        app_state[5],
        app_state[6],
        app_state[7],
    ];

    let zkapp = ZkAppAccount { app_state, ..zkapp };
    let a = Account {
        zkapp: Some(zkapp.clone()),
        ..a
    };

    let (a, local_state) = {
        let verification_key = account_update.verification_key();

        let SetVerificationKey { auth, txn_version } = &a.permissions.set_verification_key;

        let older_than_current_version = txn_version.lt(&TXN_VERSION_CURRENT);
        let original_auth = auth;

        let auth = if older_than_current_version {
            original_auth.verification_key_perm_fallback_to_signature_with_older_version()
        } else {
            *original_auth
        };

        let has_permission = controller_check(proof_verifies, signature_verifies, auth)?;

        let local_state = local_state.add_check(
            TransactionFailure::UpdateNotPermittedVerificationKey,
            verification_key.is_keep() || has_permission,
        );
        let verification_key = match zkapp.verification_key {
            Some(vk) => Some(verification_key.set_or_keep(vk)),
            None => {
                if let SetOrKeep::Set(vk) = verification_key {
                    Some(vk)
                } else {
                    None
                }
            }
        };

        let zkapp = ZkAppAccount {
            verification_key,
            ..zkapp
        };
        let a = Account {
            zkapp: Some(zkapp),
            ..a
        };
        (a, local_state)
    };

    let (a, local_state) = {
        let actions = account_update.actions();
        let zkapp = a.zkapp.unwrap();
        let last_action_slot = zkapp.last_action_slot;
        let (action_state, last_action_slot) = update_action_state(
            zkapp.action_state,
            actions.clone(),
            txn_global_slot,
            last_action_slot,
        );
        let is_empty = actions.is_empty();
        let has_permission = controller_check(
            proof_verifies,
            signature_verifies,
            a.permissions.edit_action_state,
        )?;

        let local_state = local_state.add_check(
            TransactionFailure::UpdateNotPermittedActionState,
            is_empty || has_permission,
        );
        let zkapp = ZkAppAccount {
            action_state,
            last_action_slot,
            ..zkapp
        };
        let a = Account {
            zkapp: Some(zkapp),
            ..a
        };
        (a, local_state)
    };

    let (a, local_state) = {
        let zkapp_uri = account_update.zkapp_uri();
        let has_permission = controller_check(
            proof_verifies,
            signature_verifies,
            a.permissions.set_zkapp_uri,
        )?;

        let local_state = local_state.add_check(
            TransactionFailure::UpdateNotPermittedZkappUri,
            zkapp_uri.is_keep() || has_permission,
        );
        let zkapp = a.zkapp.map(|x| ZkAppAccount {
            zkapp_uri: zkapp_uri.set_or_keep(x.zkapp_uri),
            ..x
        });
        let a = Account { zkapp, ..a };
        (a, local_state)
    };

    //  At this point, all possible changes have been made to the zkapp
    //  part of an account. Reset zkApp state to [None] if that part
    //  is unmodified.
    let a = unmake_zkapp(a);
    // Update token symbol.
    let (a, local_state) = {
        let token_symbol = account_update.token_symbol();
        let has_permission = controller_check(
            proof_verifies,
            signature_verifies,
            a.permissions.set_token_symbol,
        )?;

        let local_state = local_state.add_check(
            TransactionFailure::UpdateNotPermittedTokenSymbol,
            token_symbol.is_keep() || has_permission,
        );
        let token_symbol = token_symbol.set_or_keep(a.token_symbol);
        let a = Account { token_symbol, ..a };
        (a, local_state)
    };

    // Update delegate
    let (a, local_state) = {
        let delegate = account_update.delegate();

        // for new accounts using the default token, we've already
        // set the delegate to the public key
        let base_delegate = a.delegate.unwrap_or_else(CompressedPubKey::empty);

        let has_permission = controller_check(
            proof_verifies,
            signature_verifies,
            a.permissions.set_delegate,
        )?;

        let local_state = local_state.add_check(
            TransactionFailure::UpdateNotPermittedDelegate,
            delegate.is_keep() || (has_permission && account_update_token_is_default),
        );

        let delegate = delegate.set_or_keep(base_delegate);

        let delegate = if delegate == CompressedPubKey::empty() {
            None
        } else {
            Some(delegate)
        };
        let a = Account { delegate, ..a };
        (a, local_state)
    };

    let (a, local_state) = {
        let nonce = a.nonce;
        let increment_nonce = account_update.increment_nonce();
        let nonce = if increment_nonce { nonce.incr() } else { nonce };
        let has_permission = controller_check(
            proof_verifies,
            signature_verifies,
            a.permissions.increment_nonce,
        )?;

        let local_state = local_state.add_check(
            TransactionFailure::UpdateNotPermittedNonce,
            !increment_nonce || has_permission,
        );
        let a = Account { nonce, ..a };
        (a, local_state)
    };

    let (a, local_state) = {
        let voting_for = account_update.voting_for();
        let has_permission = controller_check(
            proof_verifies,
            signature_verifies,
            a.permissions.set_voting_for,
        )?;

        let local_state = local_state.add_check(
            TransactionFailure::UpdateNotPermittedVotingFor,
            voting_for.is_keep() || has_permission,
        );
        let voting_for = voting_for.set_or_keep(a.voting_for);
        let a = Account { voting_for, ..a };
        (a, local_state)
    };

    let a = {
        let new_hash = {
            let old_hash = a.receipt_chain_hash;
            if signature_verifies || proof_verifies {
                let elt = ZkAppCommandElt::ZkAppCommandCommitment(
                    local_state.full_transaction_commitment.clone(),
                );
                cons_zkapp_command_commitment(local_state.account_update_index, elt, &old_hash)
            } else {
                old_hash
            }
        };
        Account {
            receipt_chain_hash: new_hash,
            ..a
        }
    };

    let (a, local_state) = {
        let permissions = account_update.permissions();
        let has_permission = controller_check(
            proof_verifies,
            signature_verifies,
            a.permissions.set_permissions,
        )?;

        let local_state = local_state.add_check(
            TransactionFailure::UpdateNotPermittedPermissions,
            permissions.is_keep() || has_permission,
        );
        let permissions = permissions.set_or_keep(a.permissions);
        let a = Account { permissions, ..a };
        (a, local_state)
    };

    let a = match Env::perform(Eff::<L>::InitAccount(account_update.clone(), a)) {
        PerformResult::Account(a) => a,
        _ => unreachable!(),
    };

    let local_delta = account_update.balance_change().negate();
    let (new_local_fee_excess, overflowed) = {
        // We only allow the default token for fees.
        __assert!(!is_start_ || (account_update_token_is_default && local_delta.is_non_neg()))?;

        let (new_local_fee_excess, overflow) = local_state.excess.add_flagged(Signed::<Amount> {
            magnitude: Amount::from_u64(local_delta.magnitude.as_u64()),
            sgn: local_delta.sgn,
        });

        let new_local_fee_excess = if account_update_token_is_default {
            new_local_fee_excess
        } else {
            local_state.excess
        };
        (
            new_local_fee_excess,
            account_update_token_is_default && overflow,
        )
    };
    let local_state = LocalStateEnv {
        excess: new_local_fee_excess,
        ..local_state
    };
    let mut local_state =
        local_state.add_check(TransactionFailure::LocalExcessOverflow, !overflowed);

    let new_ledger = set_account(&mut local_state.ledger, (a, &inclusion_proof));
    let is_last_account_update = remaining.calls.is_empty();
    let local_state = LocalStateEnv {
        ledger: new_ledger.clone(),
        transaction_commitment: if is_last_account_update {
            ReceiptChainHash(Fp::zero())
        } else {
            local_state.transaction_commitment
        },
        full_transaction_commitment: if is_last_account_update {
            ReceiptChainHash(Fp::zero())
        } else {
            local_state.full_transaction_commitment
        },
        ..local_state
    };

    let valid_fee_excess = {
        let delta_settled = local_state.excess == Signed::<Amount>::zero();
        is_start_ || !is_last_account_update || delta_settled
    };
    let local_state = local_state.add_check(TransactionFailure::InvalidFeeExcess, valid_fee_excess);

    // let is_start_or_last = Bool.(is_start' ||| is_last_account_update) in
    // let update_local_excess = is_start_ || is_last_account_update;
    // let update_global_state = update_local_excess && local_state.success;

    let is_start_or_last = is_start_ || is_last_account_update;

    let update_global_state_fee_excess = is_start_or_last && local_state.success;
    // let update_global_state_fee_excess =
    //   Bool.(is_start_or_last &&& local_state.success)
    // in

    let (global_state, global_excess_update_failed) = {
        // let (global_state, global_excess_update_failed, update_global_state) = {
        let amt = global_state.fee_excess;
        let (res, overflow) = amt.add_flagged(local_state.excess);
        let global_excess_update_failed = update_global_state_fee_excess && overflow;
        // let update_global_state = update_global_state && !overflow;
        let new_amt = if update_global_state_fee_excess {
            res
        } else {
            amt
        };
        (
            GlobalState {
                fee_excess: new_amt,
                ..global_state
            },
            global_excess_update_failed,
        )
    };
    let local_state = LocalStateEnv {
        excess: if is_start_or_last {
            Signed::<Amount>::zero()
        } else {
            local_state.excess
        },
        ..local_state
    };
    let local_state = local_state.add_check(
        TransactionFailure::GlobalExcessOverflow,
        !global_excess_update_failed,
    );

    // add local supply increase in global state
    let (new_global_supply_increase, global_supply_increase_update_failed) = {
        global_state
            .supply_increase()
            .add_flagged(local_state.supply_increase)
    };
    let local_state = local_state.add_check(
        TransactionFailure::GlobalSupplyIncreaseOverflow,
        !global_supply_increase_update_failed,
    );

    // The first account_update must succeed.
    assert_with_failure_status_tbl(
        !is_start_ || local_state.success,
        local_state.failure_status_tbl.clone(),
    )?;

    // If we are the fee payer (is_start' = true), push the first pass ledger
    // and set the local ledger to be the second pass ledger in preparation for
    // the children.
    let (local_state, global_state) = {
        let is_fee_payer = is_start_;
        let global_state =
            global_state.set_first_pass_ledger(is_fee_payer, local_state.ledger.clone());

        let local_state = LocalStateEnv {
            ledger: if is_fee_payer {
                global_state.second_pass_ledger()
            } else {
                local_state.ledger
            },
            ..local_state
        };

        (local_state, global_state)
    };

    //  If this is the last account update, and [will_succeed] is false, then
    //  [success] must also be false.
    let any = [
        !is_last_account_update,
        local_state.will_succeed,
        !local_state.success,
    ]
    .iter()
    .any(|b| *b);
    // https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/transaction_logic/mina_transaction_logic.ml#L1207
    assert!(any);

    // If this is the last party and there were no failures, update the second
    // pass ledger and the supply increase.
    let global_state = {
        let is_successful_last_party = is_last_account_update && local_state.success;
        let global_state = global_state.set_supply_increase(if is_successful_last_party {
            new_global_supply_increase
        } else {
            global_state.supply_increase()
        });
        global_state.set_second_pass_ledger(is_successful_last_party, local_state.ledger.clone())
    };

    let local_state = LocalStateEnv {
        ledger: if is_last_account_update {
            L::empty(0)
        } else {
            local_state.ledger
        },
        success: if is_last_account_update {
            true
        } else {
            local_state.success
        },
        account_update_index: if is_last_account_update {
            Index::zero()
        } else {
            local_state.account_update_index.incr()
        },
        supply_increase: if is_last_account_update {
            Signed::zero()
        } else {
            local_state.supply_increase
        },
        will_succeed: if is_last_account_update {
            true
        } else {
            local_state.will_succeed
        },
        ..local_state
    };
    Ok((global_state, local_state))
}

pub fn step<L>(
    constraint_constants: &ConstraintConstants,
    h: &Handler<L>,
    state: (GlobalState<L>, LocalStateEnv<L>),
) -> Result<(GlobalState<L>, LocalStateEnv<L>), String>
where
    L: LedgerIntf + Clone,
{
    apply(constraint_constants, IsStart::No, h, state)
}

pub fn start<L>(
    constraint_constants: &ConstraintConstants,
    start_data: StartData,
    h: &Handler<L>,
    state: (GlobalState<L>, LocalStateEnv<L>),
) -> Result<(GlobalState<L>, LocalStateEnv<L>), String>
where
    L: LedgerIntf + Clone,
{
    apply(constraint_constants, IsStart::Yes(start_data), h, state)
}
