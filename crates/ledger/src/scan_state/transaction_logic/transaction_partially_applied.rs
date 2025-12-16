//! Two-phase transaction application
//!
//! This module implements the two-phase transaction application model used in Mina.
//! This approach enables efficient proof generation, particularly for zkApp commands.
//!
//! # Application Phases
//!
//! ## First Pass
//!
//! The first pass ([`apply_transaction_first_pass`]) performs:
//! - Transaction validation (nonces, balances, permissions)
//! - Fee payment
//! - For zkApp commands: applies fee payer and begins account update processing
//! - For other transactions: completes the entire application
//! - Records the previous ledger hash
//!
//! ## Second Pass
//!
//! The second pass ([`apply_transaction_second_pass`]) performs:
//! - For zkApp commands: completes account update processing and emits events/actions
//! - For other transactions: simply packages the results from first pass
//!
//! # Key Types
//!
//! - [`TransactionPartiallyApplied`]: Intermediate state between passes
//! - [`ZkappCommandPartiallyApplied`]: zkApp-specific intermediate state
//! - [`FullyApplied`]: Wrapper for non-zkApp transactions that complete in first pass
//!
//! # Fee Transfers and Coinbase
//!
//! Fee transfers and coinbase transactions also use helper functions in this module:
//! - [`apply_fee_transfer`]: Distributes fees to block producers
//! - [`apply_coinbase`]: Handles block rewards and optional fee transfers
//! - [`process_fee_transfer`]: Core logic for fee distribution with permission checks
//!
//! Both transactions have structured failure status to indicate which part failed:
//! - Single transfer: `[[failure]]`
//! - Two transfers both fail: `[[failure1]; [failure2]]`
//! - First succeeds, second fails: `[[]; [failure2]]`
//! - First fails, second succeeds: `[[failure1]; []]`

use super::{
    transaction_applied::{CoinbaseApplied, FeeTransferApplied},
    *,
};

#[derive(Clone, Debug)]
pub struct ZkappCommandPartiallyApplied<L: LedgerNonSnark> {
    pub command: ZkAppCommand,
    pub previous_hash: Fp,
    pub original_first_pass_account_states: Vec<(AccountId, Option<(L::Location, Box<Account>)>)>,
    pub constraint_constants: ConstraintConstants,
    pub state_view: ProtocolStateView,
    pub global_state: GlobalState<L>,
    pub local_state: LocalStateEnv<L>,
}

#[derive(Clone, Debug)]
pub struct FullyApplied<T> {
    pub previous_hash: Fp,
    pub applied: T,
}

#[derive(Clone, Debug)]
pub enum TransactionPartiallyApplied<L: LedgerNonSnark> {
    SignedCommand(FullyApplied<SignedCommandApplied>),
    ZkappCommand(Box<ZkappCommandPartiallyApplied<L>>),
    FeeTransfer(FullyApplied<FeeTransferApplied>),
    Coinbase(FullyApplied<CoinbaseApplied>),
}

impl<L> TransactionPartiallyApplied<L>
where
    L: LedgerNonSnark,
{
    pub fn command(self) -> Transaction {
        use Transaction as T;

        match self {
            Self::SignedCommand(s) => T::Command(UserCommand::SignedCommand(Box::new(
                s.applied.common.user_command.data,
            ))),
            Self::ZkappCommand(z) => T::Command(UserCommand::ZkAppCommand(Box::new(z.command))),
            Self::FeeTransfer(ft) => T::FeeTransfer(ft.applied.fee_transfer.data),
            Self::Coinbase(cb) => T::Coinbase(cb.applied.coinbase.data),
        }
    }
}

/// Applies the first pass of transaction application.
///
/// This function performs the initial phase of transaction processing, which includes
/// validation, fee payment, and partial application. The behavior differs based on
/// transaction type:
///
/// # Transaction Type Handling
///
/// - **Signed Commands** (payments, stake delegations): Fully applied in the first pass.
///   The result is wrapped in [`FullyApplied`] since no second pass work is needed.
///
/// - **zkApp Commands**: Partially applied. The first pass:
///   - Validates the zkApp command structure and permissions
///   - Applies the fee payer account update
///   - Begins processing the first phase of account updates
///   - Records intermediate state in [`ZkappCommandPartiallyApplied`]
///
/// - **Fee Transfers**: Fully applied in the first pass, distributing fees to block
///   producers according to the protocol rules.
///
/// - **Coinbase**: Fully applied in the first pass, crediting block rewards and any
///   associated fee transfers to the designated accounts.
///
/// # Ledger State Changes
///
/// The ledger is mutated during the first pass as follows:
///
/// - **Signed Commands**:
///   - Fee payer balance decreased by fee amount
///   - Fee payer nonce incremented
///   - Fee payer receipt chain hash updated
///   - Fee payer timing updated based on vesting schedule
///   - For payments: sender balance decreased, receiver balance increased
///   - For payments: new account created if receiver doesn't exist
///   - For stake delegations: delegate field updated
///
/// - **zkApp Commands**:
///   - Fee payer account fully updated (balance, nonce, receipt chain, timing)
///   - First phase account updates applied to ledger
///   - New accounts may be created
///
/// - **Fee Transfers**:
///   - Receiver account balances increased by fee amounts
///   - Timing updated when balances increase
///   - New accounts created if receivers don't exist
///
/// - **Coinbase**:
///   - Block producer balance increased by reward amount
///   - Fee transfer recipient balance increased (if applicable)
///   - Timing updated when balances increase
///   - New accounts created if recipients don't exist
///
/// # Parameters
///
/// - `constraint_constants`: Protocol constants including account creation fees and limits
/// - `global_slot`: Current global slot number for timing validation
/// - `txn_state_view`: View of the protocol state for validating transaction preconditions
/// - `ledger`: Mutable reference to the ledger being updated
/// - `transaction`: The transaction to apply
///
/// # Returns
///
/// Returns a [`TransactionPartiallyApplied`] containing either:
/// - [`FullyApplied`] result for transactions that complete in first pass
/// - [`ZkappCommandPartiallyApplied`] for zkApp commands needing second pass
///
/// # Errors
///
/// Returns an error if:
/// - Transaction validation fails (invalid nonce, insufficient balance, etc.)
/// - Fee payment fails
/// - Account permissions are insufficient
/// - Timing constraints are violated
///
/// ## Error Side Effects
///
/// When an error occurs, the ledger state depends on where the error occurred:
///
/// - **Errors during fee payment** (invalid nonce, nonexistent fee payer): Ledger
///   remains completely unchanged.
///
/// - **Errors after fee payment** (insufficient balance for payment, permission
///   errors): The fee has already been charged to ensure network compensation. The
///   fee payer's account will have: balance decreased by fee, nonce incremented,
///   receipt chain hash updated. However, the actual payment/operation is NOT
///   performed.
///
/// # Tests
///
/// Test coverage (in `ledger/tests/test_transaction_logic_first_pass.rs`):
///
/// - [`test_apply_payment_success`]: successful payment with ledger state verification
/// - [`test_apply_payment_insufficient_balance`]: payment exceeding sender balance
/// - [`test_apply_payment_invalid_nonce`]: payment with incorrect nonce
/// - [`test_apply_payment_nonexistent_fee_payer`]: payment from nonexistent account
///
/// [`test_apply_payment_success`]: ../../tests/test_transaction_logic_first_pass.rs
/// [`test_apply_payment_insufficient_balance`]: ../../tests/test_transaction_logic_first_pass.rs
/// [`test_apply_payment_invalid_nonce`]: ../../tests/test_transaction_logic_first_pass.rs
/// [`test_apply_payment_nonexistent_fee_payer`]: ../../tests/test_transaction_logic_first_pass.rs
pub fn apply_transaction_first_pass<L>(
    constraint_constants: &ConstraintConstants,
    global_slot: Slot,
    txn_state_view: &ProtocolStateView,
    ledger: &mut L,
    transaction: &Transaction,
) -> Result<TransactionPartiallyApplied<L>, String>
where
    L: LedgerNonSnark,
{
    use Transaction::*;
    use UserCommand::*;

    let previous_hash = ledger.merkle_root();
    let txn_global_slot = &global_slot;

    match transaction {
        Command(SignedCommand(cmd)) => apply_user_command(
            constraint_constants,
            txn_state_view,
            txn_global_slot,
            ledger,
            cmd,
        )
        .map(|applied| {
            TransactionPartiallyApplied::SignedCommand(FullyApplied {
                previous_hash,
                applied,
            })
        }),
        Command(ZkAppCommand(txn)) => apply_zkapp_command_first_pass(
            constraint_constants,
            global_slot,
            txn_state_view,
            None,
            None,
            ledger,
            txn,
        )
        .map(Box::new)
        .map(TransactionPartiallyApplied::ZkappCommand),
        FeeTransfer(fee_transfer) => {
            apply_fee_transfer(constraint_constants, txn_global_slot, ledger, fee_transfer).map(
                |applied| {
                    TransactionPartiallyApplied::FeeTransfer(FullyApplied {
                        previous_hash,
                        applied,
                    })
                },
            )
        }
        Coinbase(coinbase) => {
            apply_coinbase(constraint_constants, txn_global_slot, ledger, coinbase).map(|applied| {
                TransactionPartiallyApplied::Coinbase(FullyApplied {
                    previous_hash,
                    applied,
                })
            })
        }
    }
}

/// Completes the second pass of transaction application.
///
/// This function finalizes transaction processing by completing any remaining work
/// from the first pass. The behavior differs based on transaction type:
///
/// # Transaction Type Handling
///
/// - **Signed Commands**: No additional work needed. Simply unwraps the [`FullyApplied`]
///   result from the first pass and packages it into a [`TransactionApplied`].
///
/// - **zkApp Commands**: Completes the second phase of application:
///   - Processes the second phase of account updates
///   - Emits events and actions from the zkApp execution
///   - Updates the zkApp's on-chain state
///   - Validates all preconditions are satisfied
///
/// - **Fee Transfers**: No additional work needed. Simply packages the first pass result.
///
/// - **Coinbase**: No additional work needed. Simply packages the first pass result.
///
/// # Ledger State Changes
///
/// The ledger is mutated during the second pass only for zkApp commands:
///
/// - **Signed Commands**: No ledger changes (all modifications completed in first pass)
///
/// - **zkApp Commands**:
///   - Second phase account updates applied
///   - Account balances modified based on zkApp logic
///   - Account app state fields updated
///   - Account permissions may be modified
///   - Action state and event sequence numbers updated
///   - New accounts may be created
///
/// - **Fee Transfers**: No ledger changes (all modifications completed in first pass)
///
/// - **Coinbase**: No ledger changes (all modifications completed in first pass)
///
/// # Parameters
///
/// - `constraint_constants`: Protocol constants including account creation fees and limits
/// - `ledger`: Mutable reference to the ledger being updated
/// - `partial_transaction`: The partially applied transaction from the first pass
///
/// # Returns
///
/// Returns a [`TransactionApplied`] containing the complete application result with:
/// - Previous ledger hash (recorded during first pass)
/// - Full transaction status (Applied or Failed with specific error codes)
/// - Account updates and new account information
/// - Events and actions (for zkApp commands)
///
/// # Errors
///
/// Returns an error if:
/// - Second phase zkApp account updates fail
/// - zkApp preconditions fail during second pass
/// - Account permissions are insufficient
///
/// # Notes
///
/// For non-zkApp transactions, this function performs minimal work since the first
/// pass already completed the application. The two-phase model exists primarily to
/// enable efficient zkApp proof generation where different account updates can be
/// processed in separate circuit phases.
pub fn apply_transaction_second_pass<L>(
    constraint_constants: &ConstraintConstants,
    ledger: &mut L,
    partial_transaction: TransactionPartiallyApplied<L>,
) -> Result<TransactionApplied, String>
where
    L: LedgerNonSnark,
{
    use TransactionPartiallyApplied as P;

    match partial_transaction {
        P::SignedCommand(FullyApplied {
            previous_hash,
            applied,
        }) => Ok(TransactionApplied {
            previous_hash,
            varying: Varying::Command(CommandApplied::SignedCommand(Box::new(applied))),
        }),
        P::ZkappCommand(partially_applied) => {
            // TODO(OCaml): either here or in second phase of apply, need to update the
            // prior global state statement for the fee payer segment to add the
            // second phase ledger at the end

            let previous_hash = partially_applied.previous_hash;
            let applied =
                apply_zkapp_command_second_pass(constraint_constants, ledger, *partially_applied)?;

            Ok(TransactionApplied {
                previous_hash,
                varying: Varying::Command(CommandApplied::ZkappCommand(Box::new(applied))),
            })
        }
        P::FeeTransfer(FullyApplied {
            previous_hash,
            applied,
        }) => Ok(TransactionApplied {
            previous_hash,
            varying: Varying::FeeTransfer(applied),
        }),
        P::Coinbase(FullyApplied {
            previous_hash,
            applied,
        }) => Ok(TransactionApplied {
            previous_hash,
            varying: Varying::Coinbase(applied),
        }),
    }
}

pub fn apply_transactions<L>(
    constraint_constants: &ConstraintConstants,
    global_slot: Slot,
    txn_state_view: &ProtocolStateView,
    ledger: &mut L,
    txns: &[Transaction],
) -> Result<Vec<TransactionApplied>, String>
where
    L: LedgerNonSnark,
{
    let first_pass: Vec<_> = txns
        .iter()
        .map(|txn| {
            apply_transaction_first_pass(
                constraint_constants,
                global_slot,
                txn_state_view,
                ledger,
                txn,
            )
        })
        .collect::<Result<Vec<TransactionPartiallyApplied<_>>, _>>()?;

    first_pass
        .into_iter()
        .map(|partial_transaction| {
            apply_transaction_second_pass(constraint_constants, ledger, partial_transaction)
        })
        .collect()
}

pub struct FailureCollection {
    inner: Vec<Vec<TransactionFailure>>,
}

/// <https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/transaction_logic/mina_transaction_logic.ml#L2197C1-L2210C53>
impl FailureCollection {
    fn empty() -> Self {
        Self {
            inner: Vec::default(),
        }
    }

    fn no_failure() -> Vec<TransactionFailure> {
        vec![]
    }

    /// <https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/transaction_logic/mina_transaction_logic.ml#L2204>
    fn single_failure() -> Self {
        Self {
            inner: vec![vec![TransactionFailure::UpdateNotPermittedBalance]],
        }
    }

    fn update_failed() -> Vec<TransactionFailure> {
        vec![TransactionFailure::UpdateNotPermittedBalance]
    }

    /// <https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/transaction_logic/mina_transaction_logic.ml#L2208>
    fn append_entry(list: Vec<TransactionFailure>, mut s: Self) -> Self {
        if s.inner.is_empty() {
            Self { inner: vec![list] }
        } else {
            s.inner.insert(1, list);
            s
        }
    }

    fn is_empty(&self) -> bool {
        self.inner.iter().all(Vec::is_empty)
    }

    fn take(self) -> Vec<Vec<TransactionFailure>> {
        self.inner
    }
}

/// Structure of the failure status:
///  I. No fee transfer and coinbase transfer fails: `[[failure]]`
///  II. With fee transfer-
///   Both fee transfer and coinbase fails:
///     `[[failure-of-fee-transfer]; [failure-of-coinbase]]`
///   Fee transfer succeeds and coinbase fails:
///     `[[];[failure-of-coinbase]]`
///   Fee transfer fails and coinbase succeeds:
///     `[[failure-of-fee-transfer];[]]`
///
/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L2022>
pub fn apply_coinbase<L>(
    constraint_constants: &ConstraintConstants,
    txn_global_slot: &Slot,
    ledger: &mut L,
    coinbase: &Coinbase,
) -> Result<transaction_applied::CoinbaseApplied, String>
where
    L: LedgerIntf,
{
    let Coinbase {
        receiver,
        amount: coinbase_amount,
        fee_transfer,
    } = &coinbase;

    let (
        receiver_reward,
        new_accounts1,
        transferee_update,
        transferee_timing_prev,
        failures1,
        burned_tokens1,
    ) = match fee_transfer {
        None => (
            *coinbase_amount,
            None,
            None,
            None,
            FailureCollection::empty(),
            Amount::zero(),
        ),
        Some(
            ft @ CoinbaseFeeTransfer {
                receiver_pk: transferee,
                fee,
            },
        ) => {
            assert_ne!(transferee, receiver);

            let transferee_id = ft.receiver();
            let fee = Amount::of_fee(fee);

            let receiver_reward = coinbase_amount
                .checked_sub(&fee)
                .ok_or_else(|| "Coinbase fee transfer too large".to_string())?;

            let (transferee_account, action, can_receive) =
                has_permission_to_receive(ledger, &transferee_id);
            let new_accounts = get_new_accounts(action, transferee_id.clone());

            let timing = update_timing_when_no_deduction(txn_global_slot, &transferee_account)?;

            let balance = {
                let amount = sub_account_creation_fee(constraint_constants, action, fee)?;
                add_amount(transferee_account.balance, amount)?
            };

            if can_receive.0 {
                let (_, mut transferee_account, transferee_location) =
                    ledger.get_or_create(&transferee_id)?;

                transferee_account.balance = balance;
                transferee_account.timing = timing;

                let timing = transferee_account.timing.clone();

                (
                    receiver_reward,
                    new_accounts,
                    Some((transferee_location, transferee_account)),
                    Some(timing),
                    FailureCollection::append_entry(
                        FailureCollection::no_failure(),
                        FailureCollection::empty(),
                    ),
                    Amount::zero(),
                )
            } else {
                (
                    receiver_reward,
                    None,
                    None,
                    None,
                    FailureCollection::single_failure(),
                    fee,
                )
            }
        }
    };

    let receiver_id = AccountId::new(receiver.clone(), TokenId::default());
    let (receiver_account, action2, can_receive) = has_permission_to_receive(ledger, &receiver_id);
    let new_accounts2 = get_new_accounts(action2, receiver_id.clone());

    // Note: Updating coinbase receiver timing only if there is no fee transfer.
    // This is so as to not add any extra constraints in transaction snark for checking
    // "receiver" timings. This is OK because timing rules will not be violated when
    // balance increases and will be checked whenever an amount is deducted from the
    // account (#5973)

    let coinbase_receiver_timing = match transferee_timing_prev {
        None => update_timing_when_no_deduction(txn_global_slot, &receiver_account)?,
        Some(_) => receiver_account.timing.clone(),
    };

    let receiver_balance = {
        let amount = sub_account_creation_fee(constraint_constants, action2, receiver_reward)?;
        add_amount(receiver_account.balance, amount)?
    };

    let (failures, burned_tokens2) = if can_receive.0 {
        let (_action2, mut receiver_account, receiver_location) =
            ledger.get_or_create(&receiver_id)?;

        receiver_account.balance = receiver_balance;
        receiver_account.timing = coinbase_receiver_timing;

        ledger.set(&receiver_location, receiver_account);

        (
            FailureCollection::append_entry(FailureCollection::no_failure(), failures1),
            Amount::zero(),
        )
    } else {
        (
            FailureCollection::append_entry(FailureCollection::update_failed(), failures1),
            receiver_reward,
        )
    };

    if let Some((addr, account)) = transferee_update {
        ledger.set(&addr, account);
    };

    let burned_tokens = burned_tokens1
        .checked_add(&burned_tokens2)
        .ok_or_else(|| "burned tokens overflow".to_string())?;

    let status = if failures.is_empty() {
        TransactionStatus::Applied
    } else {
        TransactionStatus::Failed(failures.take())
    };

    let new_accounts: Vec<_> = [new_accounts1, new_accounts2]
        .into_iter()
        .flatten()
        .collect();

    Ok(transaction_applied::CoinbaseApplied {
        coinbase: WithStatus {
            data: coinbase.clone(),
            status,
        },
        new_accounts,
        burned_tokens,
    })
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L1991>
pub fn apply_fee_transfer<L>(
    constraint_constants: &ConstraintConstants,
    txn_global_slot: &Slot,
    ledger: &mut L,
    fee_transfer: &FeeTransfer,
) -> Result<transaction_applied::FeeTransferApplied, String>
where
    L: LedgerIntf,
{
    let (new_accounts, failures, burned_tokens) = process_fee_transfer(
        ledger,
        fee_transfer,
        |action, _, balance, fee| {
            let amount = {
                let amount = Amount::of_fee(fee);
                sub_account_creation_fee(constraint_constants, action, amount)?
            };
            add_amount(balance, amount)
        },
        |account| update_timing_when_no_deduction(txn_global_slot, account),
    )?;

    let status = if failures.is_empty() {
        TransactionStatus::Applied
    } else {
        TransactionStatus::Failed(failures.take())
    };

    Ok(transaction_applied::FeeTransferApplied {
        fee_transfer: WithStatus {
            data: fee_transfer.clone(),
            status,
        },
        new_accounts,
        burned_tokens,
    })
}

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L607>
fn sub_account_creation_fee(
    constraint_constants: &ConstraintConstants,
    action: AccountState,
    amount: Amount,
) -> Result<Amount, String> {
    let account_creation_fee = Amount::from_u64(constraint_constants.account_creation_fee);

    match action {
        AccountState::Added => {
            if let Some(amount) = amount.checked_sub(&account_creation_fee) {
                return Ok(amount);
            }
            Err(format!(
                "Error subtracting account creation fee {:?}; transaction amount {:?} insufficient",
                account_creation_fee, amount
            ))
        }
        AccountState::Existed => Ok(amount),
    }
}

fn update_timing_when_no_deduction(
    txn_global_slot: &Slot,
    account: &Account,
) -> Result<Timing, String> {
    validate_timing(account, Amount::zero(), txn_global_slot)
}

fn get_new_accounts<T>(action: AccountState, data: T) -> Option<T> {
    match action {
        AccountState::Added => Some(data),
        AccountState::Existed => None,
    }
}

/// Structure of the failure status:
///  I. Only one fee transfer in the transaction (`One) and it fails:
///     [[failure]]
///  II. Two fee transfers in the transaction (`Two)-
///   Both fee transfers fail:
///     [[failure-of-first-fee-transfer]; [failure-of-second-fee-transfer]]
///   First succeeds and second one fails:
///     [[];[failure-of-second-fee-transfer]]
///   First fails and second succeeds:
///     [[failure-of-first-fee-transfer];[]]
pub fn process_fee_transfer<L, FunBalance, FunTiming>(
    ledger: &mut L,
    fee_transfer: &FeeTransfer,
    modify_balance: FunBalance,
    modify_timing: FunTiming,
) -> Result<(Vec<AccountId>, FailureCollection, Amount), String>
where
    L: LedgerIntf,
    FunTiming: Fn(&Account) -> Result<Timing, String>,
    FunBalance: Fn(AccountState, &AccountId, Balance, &Fee) -> Result<Balance, String>,
{
    if !fee_transfer.fee_tokens().all(TokenId::is_default) {
        return Err("Cannot pay fees in non-default tokens.".to_string());
    }

    match &**fee_transfer {
        OneOrTwo::One(fee_transfer) => {
            let account_id = fee_transfer.receiver();
            let (a, action, can_receive) = has_permission_to_receive(ledger, &account_id);

            let timing = modify_timing(&a)?;
            let balance = modify_balance(action, &account_id, a.balance, &fee_transfer.fee)?;

            if can_receive.0 {
                let (_, mut account, loc) = ledger.get_or_create(&account_id)?;
                let new_accounts = get_new_accounts(action, account_id.clone());

                account.balance = balance;
                account.timing = timing;

                ledger.set(&loc, account);

                let new_accounts: Vec<_> = new_accounts.into_iter().collect();
                Ok((new_accounts, FailureCollection::empty(), Amount::zero()))
            } else {
                Ok((
                    vec![],
                    FailureCollection::single_failure(),
                    Amount::of_fee(&fee_transfer.fee),
                ))
            }
        }
        OneOrTwo::Two((fee_transfer1, fee_transfer2)) => {
            let account_id1 = fee_transfer1.receiver();
            let (a1, action1, can_receive1) = has_permission_to_receive(ledger, &account_id1);

            let account_id2 = fee_transfer2.receiver();

            if account_id1 == account_id2 {
                let fee = fee_transfer1
                    .fee
                    .checked_add(&fee_transfer2.fee)
                    .ok_or_else(|| "Overflow".to_string())?;

                let timing = modify_timing(&a1)?;
                let balance = modify_balance(action1, &account_id1, a1.balance, &fee)?;

                if can_receive1.0 {
                    let (_, mut a1, l1) = ledger.get_or_create(&account_id1)?;
                    let new_accounts1 = get_new_accounts(action1, account_id1);

                    a1.balance = balance;
                    a1.timing = timing;

                    ledger.set(&l1, a1);

                    let new_accounts: Vec<_> = new_accounts1.into_iter().collect();
                    Ok((new_accounts, FailureCollection::empty(), Amount::zero()))
                } else {
                    // failure for each fee transfer single

                    Ok((
                        vec![],
                        FailureCollection::append_entry(
                            FailureCollection::update_failed(),
                            FailureCollection::single_failure(),
                        ),
                        Amount::of_fee(&fee),
                    ))
                }
            } else {
                let (a2, action2, can_receive2) = has_permission_to_receive(ledger, &account_id2);

                let balance1 =
                    modify_balance(action1, &account_id1, a1.balance, &fee_transfer1.fee)?;

                // Note: Not updating the timing field of a1 to avoid additional check
                // in transactions snark (check_timing for "receiver"). This is OK
                // because timing rules will not be violated when balance increases
                // and will be checked whenever an amount is deducted from the account. (#5973)*)

                let timing2 = modify_timing(&a2)?;
                let balance2 =
                    modify_balance(action2, &account_id2, a2.balance, &fee_transfer2.fee)?;

                let (new_accounts1, failures, burned_tokens1) = if can_receive1.0 {
                    let (_, mut a1, l1) = ledger.get_or_create(&account_id1)?;
                    let new_accounts1 = get_new_accounts(action1, account_id1);

                    a1.balance = balance1;
                    ledger.set(&l1, a1);

                    (
                        new_accounts1,
                        FailureCollection::append_entry(
                            FailureCollection::no_failure(),
                            FailureCollection::empty(),
                        ),
                        Amount::zero(),
                    )
                } else {
                    (
                        None,
                        FailureCollection::single_failure(),
                        Amount::of_fee(&fee_transfer1.fee),
                    )
                };

                let (new_accounts2, failures, burned_tokens2) = if can_receive2.0 {
                    let (_, mut a2, l2) = ledger.get_or_create(&account_id2)?;
                    let new_accounts2 = get_new_accounts(action2, account_id2);

                    a2.balance = balance2;
                    a2.timing = timing2;

                    ledger.set(&l2, a2);

                    (
                        new_accounts2,
                        FailureCollection::append_entry(FailureCollection::no_failure(), failures),
                        Amount::zero(),
                    )
                } else {
                    (
                        None,
                        FailureCollection::append_entry(
                            FailureCollection::update_failed(),
                            failures,
                        ),
                        Amount::of_fee(&fee_transfer2.fee),
                    )
                };

                let burned_tokens = burned_tokens1
                    .checked_add(&burned_tokens2)
                    .ok_or_else(|| "burned tokens overflow".to_string())?;

                let new_accounts: Vec<_> = [new_accounts1, new_accounts2]
                    .into_iter()
                    .flatten()
                    .collect();

                Ok((new_accounts, failures, burned_tokens))
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AccountState {
    Added,
    Existed,
}

#[derive(Debug)]
struct HasPermissionToReceive(bool);

/// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/transaction_logic/mina_transaction_logic.ml#L1852>
fn has_permission_to_receive<L>(
    ledger: &mut L,
    receiver_account_id: &AccountId,
) -> (Box<Account>, AccountState, HasPermissionToReceive)
where
    L: LedgerIntf,
{
    use crate::PermissionTo::*;
    use AccountState::*;

    let init_account = Account::initialize(receiver_account_id);

    match ledger.location_of_account(receiver_account_id) {
        None => {
            // new account, check that default permissions allow receiving
            let perm = init_account.has_permission_to(ControlTag::NoneGiven, Receive);
            (Box::new(init_account), Added, HasPermissionToReceive(perm))
        }
        Some(location) => match ledger.get(&location) {
            None => panic!("Ledger location with no account"),
            Some(receiver_account) => {
                let perm = receiver_account.has_permission_to(ControlTag::NoneGiven, Receive);
                (receiver_account, Existed, HasPermissionToReceive(perm))
            }
        },
    }
}

pub fn validate_time(valid_until: &Slot, current_global_slot: &Slot) -> Result<(), String> {
    if current_global_slot <= valid_until {
        return Ok(());
    }

    Err(format!(
        "Current global slot {:?} greater than transaction expiry slot {:?}",
        current_global_slot, valid_until
    ))
}

pub fn is_timed(a: &Account) -> bool {
    matches!(&a.timing, Timing::Timed { .. })
}

pub fn set_with_location<L>(
    ledger: &mut L,
    location: &ExistingOrNew<L::Location>,
    account: Box<Account>,
) -> Result<(), String>
where
    L: LedgerIntf,
{
    match location {
        ExistingOrNew::Existing(location) => {
            ledger.set(location, account);
            Ok(())
        }
        ExistingOrNew::New => ledger
            .create_new_account(account.id(), *account)
            .map_err(|_| "set_with_location".to_string()),
    }
}

pub struct Updates<Location> {
    pub located_accounts: Vec<(ExistingOrNew<Location>, Box<Account>)>,
    pub applied_body: signed_command_applied::Body,
}

pub fn compute_updates<L>(
    constraint_constants: &ConstraintConstants,
    receiver: AccountId,
    ledger: &mut L,
    current_global_slot: &Slot,
    user_command: &SignedCommand,
    fee_payer: &AccountId,
    fee_payer_account: &Account,
    fee_payer_location: &ExistingOrNew<L::Location>,
    reject_command: &mut bool,
) -> Result<Updates<L::Location>, TransactionFailure>
where
    L: LedgerIntf,
{
    match &user_command.payload.body {
        signed_command::Body::StakeDelegation(_) => {
            let (receiver_location, _) = get_with_location(ledger, &receiver).unwrap();

            if let ExistingOrNew::New = receiver_location {
                return Err(TransactionFailure::ReceiverNotPresent);
            }
            if !fee_payer_account.has_permission_to_set_delegate() {
                return Err(TransactionFailure::UpdateNotPermittedDelegate);
            }

            let previous_delegate = fee_payer_account.delegate.clone();

            // Timing is always valid, but we need to record any switch from
            // timed to untimed here to stay in sync with the snark.
            let fee_payer_account = {
                let timing = timing_error_to_user_command_status(validate_timing(
                    fee_payer_account,
                    Amount::zero(),
                    current_global_slot,
                ))?;

                Box::new(Account {
                    delegate: Some(receiver.public_key.clone()),
                    timing,
                    ..fee_payer_account.clone()
                })
            };

            Ok(Updates {
                located_accounts: vec![(fee_payer_location.clone(), fee_payer_account)],
                applied_body: signed_command_applied::Body::StakeDelegation { previous_delegate },
            })
        }
        signed_command::Body::Payment(payment) => {
            let get_fee_payer_account = || {
                let balance = fee_payer_account
                    .balance
                    .sub_amount(payment.amount)
                    .ok_or(TransactionFailure::SourceInsufficientBalance)?;

                let timing = timing_error_to_user_command_status(validate_timing(
                    fee_payer_account,
                    payment.amount,
                    current_global_slot,
                ))?;

                Ok(Box::new(Account {
                    balance,
                    timing,
                    ..fee_payer_account.clone()
                }))
            };

            let fee_payer_account = match get_fee_payer_account() {
                Ok(fee_payer_account) => fee_payer_account,
                Err(e) => {
                    // OCaml throw an exception when an error occurs here
                    // Here in Rust we set `reject_command` to differentiate the 3 cases (Ok, Err, exception)
                    //
                    // <https://github.com/MinaProtocol/mina/blob/bfd1009abdbee78979ff0343cc73a3480e862f58/src/lib/transaction_logic/mina_transaction_logic.ml#L962>

                    // Don't accept transactions with insufficient balance from the fee-payer.
                    // TODO(OCaml): eliminate this condition and accept transaction with failed status
                    *reject_command = true;
                    return Err(e);
                }
            };

            let (receiver_location, mut receiver_account) = if fee_payer == &receiver {
                (fee_payer_location.clone(), fee_payer_account.clone())
            } else {
                get_with_location(ledger, &receiver).unwrap()
            };

            if !fee_payer_account.has_permission_to_send() {
                return Err(TransactionFailure::UpdateNotPermittedBalance);
            }

            if !receiver_account.has_permission_to_receive() {
                return Err(TransactionFailure::UpdateNotPermittedBalance);
            }

            let receiver_amount = match &receiver_location {
                ExistingOrNew::Existing(_) => payment.amount,
                ExistingOrNew::New => {
                    match payment
                        .amount
                        .checked_sub(&Amount::from_u64(constraint_constants.account_creation_fee))
                    {
                        Some(amount) => amount,
                        None => return Err(TransactionFailure::AmountInsufficientToCreateAccount),
                    }
                }
            };

            let balance = match receiver_account.balance.add_amount(receiver_amount) {
                Some(balance) => balance,
                None => return Err(TransactionFailure::Overflow),
            };

            let new_accounts = match receiver_location {
                ExistingOrNew::New => vec![receiver.clone()],
                ExistingOrNew::Existing(_) => vec![],
            };

            receiver_account.balance = balance;

            let updated_accounts = if fee_payer == &receiver {
                // [receiver_account] at this point has all the updates
                vec![(receiver_location, receiver_account)]
            } else {
                vec![
                    (receiver_location, receiver_account),
                    (fee_payer_location.clone(), fee_payer_account),
                ]
            };

            Ok(Updates {
                located_accounts: updated_accounts,
                applied_body: signed_command_applied::Body::Payments { new_accounts },
            })
        }
    }
}

pub fn apply_user_command_unchecked<L>(
    constraint_constants: &ConstraintConstants,
    _txn_state_view: &ProtocolStateView,
    txn_global_slot: &Slot,
    ledger: &mut L,
    user_command: &SignedCommand,
) -> Result<SignedCommandApplied, String>
where
    L: LedgerIntf,
{
    let SignedCommand {
        payload: _,
        signer: signer_pk,
        signature: _,
    } = &user_command;
    let current_global_slot = txn_global_slot;

    let valid_until = user_command.valid_until();
    validate_time(&valid_until, current_global_slot)?;

    // Fee-payer information
    let fee_payer = user_command.fee_payer();
    let (fee_payer_location, fee_payer_account) =
        pay_fee(user_command, signer_pk, ledger, current_global_slot)?;

    if !fee_payer_account.has_permission_to_send() {
        return Err(TransactionFailure::UpdateNotPermittedBalance.to_string());
    }
    if !fee_payer_account.has_permission_to_increment_nonce() {
        return Err(TransactionFailure::UpdateNotPermittedNonce.to_string());
    }

    // Charge the fee. This must happen, whether or not the command itself
    // succeeds, to ensure that the network is compensated for processing this
    // command.
    set_with_location(ledger, &fee_payer_location, fee_payer_account.clone())?;

    let receiver = user_command.receiver();

    let mut reject_command = false;

    match compute_updates(
        constraint_constants,
        receiver,
        ledger,
        current_global_slot,
        user_command,
        &fee_payer,
        &fee_payer_account,
        &fee_payer_location,
        &mut reject_command,
    ) {
        Ok(Updates {
            located_accounts,
            applied_body,
        }) => {
            for (location, account) in located_accounts {
                set_with_location(ledger, &location, account)?;
            }

            Ok(SignedCommandApplied {
                common: signed_command_applied::Common {
                    user_command: WithStatus::<SignedCommand> {
                        data: user_command.clone(),
                        status: TransactionStatus::Applied,
                    },
                },
                body: applied_body,
            })
        }
        Err(failure) if !reject_command => Ok(SignedCommandApplied {
            common: signed_command_applied::Common {
                user_command: WithStatus::<SignedCommand> {
                    data: user_command.clone(),
                    status: TransactionStatus::Failed(vec![vec![failure]]),
                },
            },
            body: signed_command_applied::Body::Failed,
        }),
        Err(failure) => {
            // This case occurs when an exception is throwned in OCaml
            // <https://github.com/MinaProtocol/mina/blob/3753a8593cc1577bcf4da16620daf9946d88e8e5/src/lib/transaction_logic/mina_transaction_logic.ml#L964>
            assert!(reject_command);
            Err(failure.to_string())
        }
    }
}

pub fn apply_user_command<L>(
    constraint_constants: &ConstraintConstants,
    txn_state_view: &ProtocolStateView,
    txn_global_slot: &Slot,
    ledger: &mut L,
    user_command: &SignedCommand,
) -> Result<SignedCommandApplied, String>
where
    L: LedgerIntf,
{
    apply_user_command_unchecked(
        constraint_constants,
        txn_state_view,
        txn_global_slot,
        ledger,
        user_command,
    )
}

pub fn pay_fee<L, Loc>(
    user_command: &SignedCommand,
    signer_pk: &CompressedPubKey,
    ledger: &mut L,
    current_global_slot: &Slot,
) -> Result<(ExistingOrNew<Loc>, Box<Account>), String>
where
    L: LedgerIntf<Location = Loc>,
{
    let nonce = user_command.nonce();
    let fee_payer = user_command.fee_payer();
    let fee_token = user_command.fee_token();

    if &fee_payer.public_key != signer_pk {
        return Err("Cannot pay fees from a public key that did not sign the transaction".into());
    }

    if fee_token != TokenId::default() {
        return Err("Cannot create transactions with fee_token different from the default".into());
    }

    pay_fee_impl(
        &user_command.payload,
        nonce,
        fee_payer,
        user_command.fee(),
        ledger,
        current_global_slot,
    )
}

pub fn pay_fee_impl<L>(
    command: &SignedCommandPayload,
    nonce: Nonce,
    fee_payer: AccountId,
    fee: Fee,
    ledger: &mut L,
    current_global_slot: &Slot,
) -> Result<(ExistingOrNew<L::Location>, Box<Account>), String>
where
    L: LedgerIntf,
{
    // Fee-payer information
    let (location, mut account) = get_with_location(ledger, &fee_payer)?;

    if let ExistingOrNew::New = location {
        return Err("The fee-payer account does not exist".to_string());
    };

    let fee = Amount::of_fee(&fee);
    let balance = sub_amount(account.balance, fee)?;

    validate_nonces(nonce, account.nonce)?;
    let timing = validate_timing(&account, fee, current_global_slot)?;

    account.balance = balance;
    account.nonce = account.nonce.incr(); // TODO: Not sure if OCaml wraps
    account.receipt_chain_hash = cons_signed_command_payload(command, account.receipt_chain_hash);
    account.timing = timing;

    Ok((location, account))
}
