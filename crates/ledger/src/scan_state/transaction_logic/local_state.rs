use super::{
    protocol_state::{GlobalState, ProtocolStateView},
    transaction_applied::ZkappCommandApplied,
    transaction_partially_applied::ZkappCommandPartiallyApplied,
    zkapp_command::{AccountUpdate, CallForest, WithHash, ZkAppCommand},
    TransactionFailure, TransactionStatus, WithStatus,
};
use crate::{
    proofs::{
        field::{field, Boolean, ToBoolean},
        numbers::nat::CheckedNat,
        to_field_elements::ToFieldElements,
        witness::Witness,
    },
    scan_state::currency::{Amount, Index, Magnitude, Signed, Slot},
    sparse_ledger::LedgerIntf,
    zkapps::{
        self,
        interfaces::{
            CallStackInterface, IndexInterface, SignedAmountInterface, StackFrameInterface,
        },
        non_snark::{LedgerNonSnark, ZkappNonSnark},
    },
    AccountId, AccountIdOrderable, AppendToInputs, ToInputs, TokenId,
};
use ark_ff::Zero;
use itertools::{FoldWhile, Itertools};
use mina_core::constants::ConstraintConstants;
use mina_curves::pasta::Fp;
use poseidon::hash::{hash_with_kimchi, params::MINA_ACCOUNT_UPDATE_STACK_FRAME, Inputs};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

#[derive(Debug, Clone, Default)]
pub struct StackFrame {
    pub caller: TokenId,
    pub caller_caller: TokenId,
    pub calls: CallForest<AccountUpdate>, // TODO
}

// <https://github.com/MinaProtocol/mina/blob/78535ae3a73e0e90c5f66155365a934a15535779/src/lib/transaction_snark/transaction_snark.ml#L1081>
#[derive(Debug, Clone)]
pub struct StackFrameCheckedFrame {
    pub caller: TokenId,
    pub caller_caller: TokenId,
    pub calls: WithHash<CallForest<AccountUpdate>>,
    /// Hack until we have proper cvar
    pub is_default: bool,
}

impl ToFieldElements<Fp> for StackFrameCheckedFrame {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self {
            caller,
            caller_caller,
            calls,
            is_default: _,
        } = self;

        // calls.hash().to_field_elements(fields);
        calls.hash.to_field_elements(fields);
        caller_caller.to_field_elements(fields);
        caller.to_field_elements(fields);
    }
}

#[derive(Default)]
enum LazyValueInner<T, D> {
    Value(T),
    Fun(Box<dyn FnOnce(&mut D) -> T>),
    #[default]
    None,
}

pub struct LazyValue<T, D> {
    value: Rc<RefCell<LazyValueInner<T, D>>>,
}

impl<T, D> Clone for LazyValue<T, D> {
    fn clone(&self) -> Self {
        Self {
            value: Rc::clone(&self.value),
        }
    }
}

impl<T: std::fmt::Debug, D> std::fmt::Debug for LazyValue<T, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.try_get();
        f.debug_struct("LazyValue").field("value", &v).finish()
    }
}

impl<T, D> LazyValue<T, D> {
    pub fn make<F>(fun: F) -> Self
    where
        F: FnOnce(&mut D) -> T + 'static,
    {
        Self {
            value: Rc::new(RefCell::new(LazyValueInner::Fun(Box::new(fun)))),
        }
    }

    fn get_impl(&self) -> std::cell::Ref<'_, T> {
        use std::cell::Ref;

        let inner = self.value.borrow();
        Ref::map(inner, |inner| {
            let LazyValueInner::Value(value) = inner else {
                panic!("invalid state");
            };
            value
        })
    }

    /// Returns the value when it already has been "computed"
    pub fn try_get(&self) -> Option<std::cell::Ref<'_, T>> {
        let inner = self.value.borrow();

        match &*inner {
            LazyValueInner::Value(_) => {}
            LazyValueInner::Fun(_) => return None,
            LazyValueInner::None => panic!("invalid state"),
        }

        Some(self.get_impl())
    }

    pub fn get(&self, data: &mut D) -> std::cell::Ref<'_, T> {
        let v = self.value.borrow();

        if let LazyValueInner::Fun(_) = &*v {
            std::mem::drop(v);

            let LazyValueInner::Fun(fun) = self.value.take() else {
                panic!("invalid state");
            };

            let data = fun(data);
            self.value.replace(LazyValueInner::Value(data));
        };

        self.get_impl()
    }
}

#[derive(Clone, Debug)]
pub struct WithLazyHash<T> {
    pub data: T,
    hash: LazyValue<Fp, Witness<Fp>>,
}

impl<T> WithLazyHash<T> {
    pub fn new<F>(data: T, fun: F) -> Self
    where
        F: FnOnce(&mut Witness<Fp>) -> Fp + 'static,
    {
        Self {
            data,
            hash: LazyValue::make(fun),
        }
    }

    pub fn hash(&self, w: &mut Witness<Fp>) -> Fp {
        *self.hash.get(w)
    }
}

impl<T> std::ops::Deref for WithLazyHash<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> ToFieldElements<Fp> for WithLazyHash<T> {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let hash = self.hash.try_get().expect("hash hasn't been computed yet");
        hash.to_field_elements(fields)
    }
}

// <https://github.com/MinaProtocol/mina/blob/78535ae3a73e0e90c5f66155365a934a15535779/src/lib/transaction_snark/transaction_snark.ml#L1083>
pub type StackFrameChecked = WithLazyHash<StackFrameCheckedFrame>;

impl StackFrame {
    pub fn empty() -> Self {
        Self {
            caller: TokenId::default(),
            caller_caller: TokenId::default(),
            calls: CallForest(Vec::new()),
        }
    }

    /// TODO: this needs to be tested
    ///
    /// <https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/mina_base/stack_frame.ml#L90>
    pub fn hash(&self) -> Fp {
        let mut inputs = Inputs::new();

        inputs.append_field(self.caller.0);
        inputs.append_field(self.caller_caller.0);

        self.calls.ensure_hashed();
        let field = match self.calls.0.first() {
            None => Fp::zero(),
            Some(calls) => calls.stack_hash.get().unwrap(), // Never fail, we called `ensure_hashed`
        };
        inputs.append_field(field);

        hash_with_kimchi(&MINA_ACCOUNT_UPDATE_STACK_FRAME, &inputs.to_fields())
    }

    pub fn digest(&self) -> Fp {
        self.hash()
    }

    pub fn unhash(&self, _h: Fp, w: &mut Witness<Fp>) -> StackFrameChecked {
        let v = self.exists_elt(w);
        v.hash(w);
        v
    }

    pub fn exists_elt(&self, w: &mut Witness<Fp>) -> StackFrameChecked {
        // We decompose this way because of OCaml evaluation order
        let calls = WithHash {
            data: self.calls.clone(),
            hash: w.exists(self.calls.hash()),
        };
        let caller_caller = w.exists(self.caller_caller.clone());
        let caller = w.exists(self.caller.clone());

        let frame = StackFrameCheckedFrame {
            caller,
            caller_caller,
            calls,
            is_default: false,
        };

        StackFrameChecked::of_frame(frame)
    }
}

impl StackFrameCheckedFrame {
    pub fn hash(&self, w: &mut Witness<Fp>) -> Fp {
        let mut inputs = Inputs::new();

        inputs.append(&self.caller);
        inputs.append(&self.caller_caller.0);
        inputs.append(&self.calls.hash);

        let fields = inputs.to_fields();

        if self.is_default {
            use crate::proofs::transaction::transaction_snark::checked_hash3;
            checked_hash3(&MINA_ACCOUNT_UPDATE_STACK_FRAME, &fields, w)
        } else {
            use crate::proofs::transaction::transaction_snark::checked_hash;
            checked_hash(&MINA_ACCOUNT_UPDATE_STACK_FRAME, &fields, w)
        }
    }
}

impl StackFrameChecked {
    pub fn of_frame(frame: StackFrameCheckedFrame) -> Self {
        // TODO: Don't clone here
        let frame2 = frame.clone();
        let hash = LazyValue::make(move |w: &mut Witness<Fp>| frame2.hash(w));

        Self { data: frame, hash }
    }
}

#[derive(Debug, Clone)]
pub struct CallStack(pub Vec<StackFrame>);

impl Default for CallStack {
    fn default() -> Self {
        Self::new()
    }
}

impl CallStack {
    pub fn new() -> Self {
        CallStack(Vec::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &StackFrame> {
        self.0.iter().rev()
    }

    pub fn push(&self, stack_frame: &StackFrame) -> Self {
        let mut ret = self.0.clone();
        ret.push(stack_frame.clone());
        Self(ret)
    }

    pub fn pop(&self) -> Option<(StackFrame, CallStack)> {
        let mut ret = self.0.clone();
        ret.pop().map(|frame| (frame, Self(ret)))
    }

    pub fn pop_exn(&self) -> (StackFrame, CallStack) {
        let mut ret = self.0.clone();
        if let Some(frame) = ret.pop() {
            (frame, Self(ret))
        } else {
            panic!()
        }
    }
}

// NOTE: It looks like there are different instances of the polymorphic LocalEnv type
// One with concrete types for the stack frame, call stack, and ledger. Created from the Env
// And the other with their hashes. To differentiate them I renamed the first LocalStateEnv
// Maybe a better solution is to keep the LocalState name and put it under a different module
// pub type LocalStateEnv<L> = LocalStateSkeleton<
//     L,                            // ledger
//     StackFrame,                   // stack_frame
//     CallStack,                    // call_stack
//     ReceiptChainHash,             // commitments
//     Signed<Amount>,               // excess & supply_increase
//     Vec<Vec<TransactionFailure>>, // failure_status_tbl
//     bool,                         // success & will_succeed
//     Index,                        // account_update_index
// >;

pub type LocalStateEnv<L> = crate::zkapps::zkapp_logic::LocalState<ZkappNonSnark<L>>;

// TODO: Dedub this with `crate::zkapps::zkapp_logic::LocalState`
#[derive(Debug, Clone)]
pub struct LocalStateSkeleton<
    L: LedgerIntf + Clone,
    StackFrame: StackFrameInterface,
    CallStack: CallStackInterface,
    TC,
    SignedAmount: SignedAmountInterface,
    FailuresTable,
    Bool,
    Index: IndexInterface,
> {
    pub stack_frame: StackFrame,
    pub call_stack: CallStack,
    pub transaction_commitment: TC,
    pub full_transaction_commitment: TC,
    pub excess: SignedAmount,
    pub supply_increase: SignedAmount,
    pub ledger: L,
    pub success: Bool,
    pub account_update_index: Index,
    // TODO: optimize by reversing the insertion order
    pub failure_status_tbl: FailuresTable,
    pub will_succeed: Bool,
}

// impl<L> LocalStateEnv<L>
// where
//     L: LedgerNonSnark,
// {
//     pub fn add_new_failure_status_bucket(&self) -> Self {
//         let mut failure_status_tbl = self.failure_status_tbl.clone();
//         failure_status_tbl.insert(0, Vec::new());
//         Self {
//             failure_status_tbl,
//             ..self.clone()
//         }
//     }

//     pub fn add_check(&self, failure: TransactionFailure, b: bool) -> Self {
//         let failure_status_tbl = if !b {
//             let mut failure_status_tbl = self.failure_status_tbl.clone();
//             failure_status_tbl[0].insert(0, failure);
//             failure_status_tbl
//         } else {
//             self.failure_status_tbl.clone()
//         };

//         Self {
//             failure_status_tbl,
//             success: self.success && b,
//             ..self.clone()
//         }
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalState {
    pub stack_frame: Fp,
    pub call_stack: Fp,
    pub transaction_commitment: Fp,
    pub full_transaction_commitment: Fp,
    pub excess: Signed<Amount>,
    pub supply_increase: Signed<Amount>,
    pub ledger: Fp,
    pub success: bool,
    pub account_update_index: Index,
    pub failure_status_tbl: Vec<Vec<TransactionFailure>>,
    pub will_succeed: bool,
}

impl ToInputs for LocalState {
    /// <https://github.com/MinaProtocol/mina/blob/4e0b324912017c3ff576704ee397ade3d9bda412/src/lib/mina_state/local_state.ml#L116>
    fn to_inputs(&self, inputs: &mut Inputs) {
        let Self {
            stack_frame,
            call_stack,
            transaction_commitment,
            full_transaction_commitment,
            excess,
            supply_increase,
            ledger,
            success,
            account_update_index,
            failure_status_tbl: _,
            will_succeed,
        } = self;

        inputs.append(stack_frame);
        inputs.append(call_stack);
        inputs.append(transaction_commitment);
        inputs.append(full_transaction_commitment);
        inputs.append(excess);
        inputs.append(supply_increase);
        inputs.append(ledger);
        inputs.append(account_update_index);
        inputs.append(success);
        inputs.append(will_succeed);
    }
}

impl LocalState {
    /// <https://github.com/MinaProtocol/mina/blob/436023ba41c43a50458a551b7ef7a9ae61670b25/src/lib/mina_state/local_state.ml#L65>
    pub fn dummy() -> Self {
        Self {
            stack_frame: StackFrame::empty().hash(),
            call_stack: Fp::zero(),
            transaction_commitment: Fp::zero(),
            full_transaction_commitment: Fp::zero(),
            excess: Signed::<Amount>::zero(),
            supply_increase: Signed::<Amount>::zero(),
            ledger: Fp::zero(),
            success: true,
            account_update_index: <Index as Magnitude>::zero(),
            failure_status_tbl: Vec::new(),
            will_succeed: true,
        }
    }

    pub fn empty() -> Self {
        Self::dummy()
    }

    pub fn equal_without_ledger(&self, other: &Self) -> bool {
        let Self {
            stack_frame,
            call_stack,
            transaction_commitment,
            full_transaction_commitment,
            excess,
            supply_increase,
            ledger: _,
            success,
            account_update_index,
            failure_status_tbl,
            will_succeed,
        } = self;

        stack_frame == &other.stack_frame
            && call_stack == &other.call_stack
            && transaction_commitment == &other.transaction_commitment
            && full_transaction_commitment == &other.full_transaction_commitment
            && excess == &other.excess
            && supply_increase == &other.supply_increase
            && success == &other.success
            && account_update_index == &other.account_update_index
            && failure_status_tbl == &other.failure_status_tbl
            && will_succeed == &other.will_succeed
    }

    pub fn checked_equal_prime(&self, other: &Self, w: &mut Witness<Fp>) -> [Boolean; 11] {
        let Self {
            stack_frame,
            call_stack,
            transaction_commitment,
            full_transaction_commitment,
            excess,
            supply_increase,
            ledger,
            success,
            account_update_index,
            failure_status_tbl: _,
            will_succeed,
        } = self;

        // { stack_frame : 'stack_frame
        // ; call_stack : 'call_stack
        // ; transaction_commitment : 'comm
        // ; full_transaction_commitment : 'comm
        // ; excess : 'signed_amount
        // ; supply_increase : 'signed_amount
        // ; ledger : 'ledger
        // ; success : 'bool
        // ; account_update_index : 'length
        // ; failure_status_tbl : 'failure_status_tbl
        // ; will_succeed : 'bool
        // }

        let mut alls = [
            field::equal(*stack_frame, other.stack_frame, w),
            field::equal(*call_stack, other.call_stack, w),
            field::equal(*transaction_commitment, other.transaction_commitment, w),
            field::equal(
                *full_transaction_commitment,
                other.full_transaction_commitment,
                w,
            ),
            excess
                .to_checked::<Fp>()
                .equal(&other.excess.to_checked(), w),
            supply_increase
                .to_checked::<Fp>()
                .equal(&other.supply_increase.to_checked(), w),
            field::equal(*ledger, other.ledger, w),
            success.to_boolean().equal(&other.success.to_boolean(), w),
            account_update_index
                .to_checked::<Fp>()
                .equal(&other.account_update_index.to_checked(), w),
            Boolean::True,
            will_succeed
                .to_boolean()
                .equal(&other.will_succeed.to_boolean(), w),
        ];
        alls.reverse();
        alls
    }
}

fn step_all<A, L>(
    _constraint_constants: &ConstraintConstants,
    f: &impl Fn(&mut A, &GlobalState<L>, &LocalStateEnv<L>),
    user_acc: &mut A,
    (g_state, l_state): (&mut GlobalState<L>, &mut LocalStateEnv<L>),
) -> Result<Vec<Vec<TransactionFailure>>, String>
where
    L: LedgerNonSnark,
{
    while !l_state.stack_frame.calls.is_empty() {
        zkapps::non_snark::step(g_state, l_state)?;
        f(user_acc, g_state, l_state);
    }
    Ok(l_state.failure_status_tbl.clone())
}

/// apply zkapp command fee payer's while stubbing out the second pass ledger
/// CAUTION: If you use the intermediate local states, you MUST update the
/// [`LocalStateEnv::will_succeed`] field to `false` if the `status` is [`TransactionStatus::Failed`].*)
pub fn apply_zkapp_command_first_pass_aux<A, F, L>(
    constraint_constants: &ConstraintConstants,
    global_slot: Slot,
    state_view: &ProtocolStateView,
    init: &mut A,
    f: F,
    fee_excess: Option<Signed<Amount>>,
    supply_increase: Option<Signed<Amount>>,
    ledger: &mut L,
    command: &ZkAppCommand,
) -> Result<ZkappCommandPartiallyApplied<L>, String>
where
    L: LedgerNonSnark,
    F: Fn(&mut A, &GlobalState<L>, &LocalStateEnv<L>),
{
    let fee_excess = fee_excess.unwrap_or_else(Signed::zero);
    let supply_increase = supply_increase.unwrap_or_else(Signed::zero);

    let previous_hash = ledger.merkle_root();
    let original_first_pass_account_states = {
        let id = command.fee_payer();
        let location = {
            let loc = ledger.location_of_account(&id);
            let account = loc.as_ref().and_then(|loc| ledger.get(loc));
            loc.zip(account)
        };

        vec![(id, location)]
    };
    // let perform = |eff: Eff<L>| Env::perform(eff);

    let (mut global_state, mut local_state) = (
        GlobalState {
            protocol_state: state_view.clone(),
            first_pass_ledger: ledger.clone(),
            second_pass_ledger: {
                // We stub out the second_pass_ledger initially, and then poke the
                // correct value in place after the first pass is finished.
                <L as LedgerIntf>::empty(0)
            },
            fee_excess,
            supply_increase,
            block_global_slot: global_slot,
        },
        LocalStateEnv {
            stack_frame: StackFrame::default(),
            call_stack: CallStack::new(),
            transaction_commitment: Fp::zero(),
            full_transaction_commitment: Fp::zero(),
            excess: Signed::<Amount>::zero(),
            supply_increase,
            ledger: <L as LedgerIntf>::empty(0),
            success: true,
            account_update_index: IndexInterface::zero(),
            failure_status_tbl: Vec::new(),
            will_succeed: true,
        },
    );

    f(init, &global_state, &local_state);
    let account_updates = command.all_account_updates();

    zkapps::non_snark::start(
        &mut global_state,
        &mut local_state,
        zkapps::non_snark::StartData {
            account_updates,
            memo_hash: command.memo.hash(),
            // It's always valid to set this value to true, and it will
            // have no effect outside of the snark.
            will_succeed: true,
        },
    )?;

    let command = command.clone();
    let constraint_constants = constraint_constants.clone();
    let state_view = state_view.clone();

    let res = ZkappCommandPartiallyApplied {
        command,
        previous_hash,
        original_first_pass_account_states,
        constraint_constants,
        state_view,
        global_state,
        local_state,
    };

    Ok(res)
}

pub fn apply_zkapp_command_first_pass<L>(
    constraint_constants: &ConstraintConstants,
    global_slot: Slot,
    state_view: &ProtocolStateView,
    fee_excess: Option<Signed<Amount>>,
    supply_increase: Option<Signed<Amount>>,
    ledger: &mut L,
    command: &ZkAppCommand,
) -> Result<ZkappCommandPartiallyApplied<L>, String>
where
    L: LedgerNonSnark,
{
    let mut acc = ();
    let partial_stmt = apply_zkapp_command_first_pass_aux(
        constraint_constants,
        global_slot,
        state_view,
        &mut acc,
        |_acc, _g, _l| {},
        fee_excess,
        supply_increase,
        ledger,
        command,
    )?;

    Ok(partial_stmt)
}

pub fn apply_zkapp_command_second_pass_aux<A, F, L>(
    constraint_constants: &ConstraintConstants,
    init: &mut A,
    f: F,
    ledger: &mut L,
    c: ZkappCommandPartiallyApplied<L>,
) -> Result<ZkappCommandApplied, String>
where
    L: LedgerNonSnark,
    F: Fn(&mut A, &GlobalState<L>, &LocalStateEnv<L>),
{
    // let perform = |eff: Eff<L>| Env::perform(eff);

    let original_account_states: Vec<(AccountId, Option<_>)> = {
        // get the original states of all the accounts in each pass.
        // If an account updated in the first pass is referenced in account
        // updates, then retain the value before first pass application*)

        let accounts_referenced = c.command.accounts_referenced();

        let mut account_states = BTreeMap::<AccountIdOrderable, Option<_>>::new();

        let referenced = accounts_referenced.into_iter().map(|id| {
            let location = {
                let loc = ledger.location_of_account(&id);
                let account = loc.as_ref().and_then(|loc| ledger.get(loc));
                loc.zip(account)
            };
            (id, location)
        });

        c.original_first_pass_account_states
            .into_iter()
            .chain(referenced)
            .for_each(|(id, acc_opt)| {
                use std::collections::btree_map::Entry::Vacant;

                let id_with_order: AccountIdOrderable = id.into();
                if let Vacant(entry) = account_states.entry(id_with_order) {
                    entry.insert(acc_opt);
                };
            });

        account_states
            .into_iter()
            // Convert back the `AccountIdOrder` into `AccountId`, now that they are sorted
            .map(|(id, account): (AccountIdOrderable, Option<_>)| (id.into(), account))
            .collect()
    };

    let mut account_states_after_fee_payer = {
        // To check if the accounts remain unchanged in the event the transaction
        // fails. First pass updates will remain even if the transaction fails to
        // apply zkapp account updates*)

        c.command.accounts_referenced().into_iter().map(|id| {
            let loc = ledger.location_of_account(&id);
            let a = loc.as_ref().and_then(|loc| ledger.get(loc));

            match a {
                Some(a) => (id, Some((loc.unwrap(), a))),
                None => (id, None),
            }
        })
    };

    let accounts = || {
        original_account_states
            .iter()
            .map(|(id, account)| (id.clone(), account.as_ref().map(|(_loc, acc)| acc.clone())))
            .collect::<Vec<_>>()
    };

    // Warning(OCaml): This is an abstraction leak / hack.
    // Here, we update global second pass ledger to be the input ledger, and
    // then update the local ledger to be the input ledger *IF AND ONLY IF*
    // there are more transaction segments to be processed in this pass.

    // TODO(OCaml): Remove this, and uplift the logic into the call in staged ledger.

    let mut global_state = GlobalState {
        second_pass_ledger: ledger.clone(),
        ..c.global_state
    };

    let mut local_state = {
        if c.local_state.stack_frame.calls.is_empty() {
            // Don't mess with the local state; we've already finished the
            // transaction after the fee payer.
            c.local_state
        } else {
            // Install the ledger that should already be in the local state, but
            // may not be in some situations depending on who the caller is.
            LocalStateEnv {
                ledger: global_state.second_pass_ledger(),
                ..c.local_state
            }
        }
    };

    f(init, &global_state, &local_state);
    let start = (&mut global_state, &mut local_state);

    let reversed_failure_status_tbl = step_all(constraint_constants, &f, init, start)?;

    let failure_status_tbl = reversed_failure_status_tbl
        .into_iter()
        .rev()
        .collect::<Vec<_>>();

    let account_ids_originally_not_in_ledger =
        original_account_states
            .iter()
            .filter_map(|(acct_id, loc_and_acct)| {
                if loc_and_acct.is_none() {
                    Some(acct_id)
                } else {
                    None
                }
            });

    let successfully_applied = failure_status_tbl.concat().is_empty();

    // if the zkapp command fails in at least 1 account update,
    // then all the account updates would be cancelled except
    // the fee payer one
    let failure_status_tbl = if successfully_applied {
        failure_status_tbl
    } else {
        failure_status_tbl
            .into_iter()
            .enumerate()
            .map(|(idx, fs)| {
                if idx > 0 && fs.is_empty() {
                    vec![TransactionFailure::Cancelled]
                } else {
                    fs
                }
            })
            .collect()
    };

    // accounts not originally in ledger, now present in ledger
    let new_accounts = account_ids_originally_not_in_ledger
        .filter(|acct_id| ledger.location_of_account(acct_id).is_some())
        .cloned()
        .collect::<Vec<_>>();

    let new_accounts_is_empty = new_accounts.is_empty();

    let valid_result = Ok(ZkappCommandApplied {
        accounts: accounts(),
        command: WithStatus {
            data: c.command,
            status: if successfully_applied {
                TransactionStatus::Applied
            } else {
                TransactionStatus::Failed(failure_status_tbl)
            },
        },
        new_accounts,
    });

    if successfully_applied {
        valid_result
    } else {
        let other_account_update_accounts_unchanged = account_states_after_fee_payer
            .fold_while(true, |acc, (_, loc_opt)| match loc_opt {
                Some((loc, a)) => match ledger.get(&loc) {
                    Some(a_) if !(a == a_) => FoldWhile::Done(false),
                    _ => FoldWhile::Continue(acc),
                },
                _ => FoldWhile::Continue(acc),
            })
            .into_inner();

        // Other zkapp_command failed, therefore, updates in those should not get applied
        if new_accounts_is_empty && other_account_update_accounts_unchanged {
            valid_result
        } else {
            Err("Zkapp_command application failed but new accounts created or some of the other account_update updates applied".to_string())
        }
    }
}

pub fn apply_zkapp_command_second_pass<L>(
    constraint_constants: &ConstraintConstants,
    ledger: &mut L,
    c: ZkappCommandPartiallyApplied<L>,
) -> Result<ZkappCommandApplied, String>
where
    L: LedgerNonSnark,
{
    let x = apply_zkapp_command_second_pass_aux(
        constraint_constants,
        &mut (),
        |_, _, _| {},
        ledger,
        c,
    )?;
    Ok(x)
}

fn apply_zkapp_command_unchecked_aux<A, F, L>(
    constraint_constants: &ConstraintConstants,
    global_slot: Slot,
    state_view: &ProtocolStateView,
    init: &mut A,
    f: F,
    fee_excess: Option<Signed<Amount>>,
    supply_increase: Option<Signed<Amount>>,
    ledger: &mut L,
    command: &ZkAppCommand,
) -> Result<ZkappCommandApplied, String>
where
    L: LedgerNonSnark,
    F: Fn(&mut A, &GlobalState<L>, &LocalStateEnv<L>),
{
    let partial_stmt = apply_zkapp_command_first_pass_aux(
        constraint_constants,
        global_slot,
        state_view,
        init,
        &f,
        fee_excess,
        supply_increase,
        ledger,
        command,
    )?;

    apply_zkapp_command_second_pass_aux(constraint_constants, init, &f, ledger, partial_stmt)
}

fn apply_zkapp_command_unchecked<L>(
    constraint_constants: &ConstraintConstants,
    global_slot: Slot,
    state_view: &ProtocolStateView,
    ledger: &mut L,
    command: &ZkAppCommand,
) -> Result<(ZkappCommandApplied, (LocalStateEnv<L>, Signed<Amount>)), String>
where
    L: LedgerNonSnark,
{
    let zkapp_partially_applied: ZkappCommandPartiallyApplied<L> = apply_zkapp_command_first_pass(
        constraint_constants,
        global_slot,
        state_view,
        None,
        None,
        ledger,
        command,
    )?;

    let mut state_res = None;
    let account_update_applied = apply_zkapp_command_second_pass_aux(
        constraint_constants,
        &mut state_res,
        |acc, global_state, local_state| {
            *acc = Some((local_state.clone(), global_state.fee_excess))
        },
        ledger,
        zkapp_partially_applied,
    )?;
    let (state, amount) = state_res.unwrap();

    Ok((account_update_applied, (state.clone(), amount)))
}
