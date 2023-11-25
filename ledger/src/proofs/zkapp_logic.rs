use mina_hasher::Fp;

use crate::{
    scan_state::{
        scan_state::ConstraintConstants,
        transaction_logic::{local_state::LocalStateSkeleton, protocol_state::GlobalStateSkeleton},
    },
    zkapps::intefaces::*,
    TokenId,
};

use super::{
    witness::{Boolean, Witness},
    zkapp::{StartDataForProof, StartDataSkeleton},
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

pub enum Eff {
    // CheckValidWhilePrecondition(Numeric<Slot>, GlobalStateForProof),
    // CheckAccountPrecondition(AccountUpdate, Account, bool, LocalStateForProof),
    // CheckProtocolStatePrecondition(ZkAppPreconditions, GlobalStateForProof),
    // InitAccount(AccountUpdate, Account),
}

pub struct Handler {
    pub perform: fn(&Eff) -> PerformResult,
}

fn assert_(_b: Boolean) -> Result<(), String> {
    // Used only for circuit generation (add constraints)
    // https://github.com/MinaProtocol/mina/blob/e44ddfe1ca54b3855e1ed336d89f6230d35aeb8c/src/lib/transaction_logic/zkapp_command_logic.ml#L929

    // TODO: In non-witness generation, we raise an exception
    Ok(())
}

fn stack_frame_default<Z: ZkappApplication>(w: &mut Z::WitnessGenerator) -> Z::StackFrame {
    Z::StackFrame::make(
        TokenId::default(),
        TokenId::default(),
        &Z::CallForest::empty(),
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
) {
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

    let a = current_forest.calls().pop_exn(w);
}

//   let (account_update, account_update_forest), remainder_of_current_forest =
//     Call_forest.pop_exn (Stack_frame.calls current_forest)
//   in
//   let may_use_parents_own_token =
//     Account_update.may_use_parents_own_token account_update
//   in
//   let may_use_token_inherited_from_parent =
//     Account_update.may_use_token_inherited_from_parent account_update
//   in
//   let caller_id =
//     Token_id.if_ may_use_token_inherited_from_parent
//       ~then_:(Stack_frame.caller_caller current_forest)
//       ~else_:
//         (Token_id.if_ may_use_parents_own_token
//            ~then_:(Stack_frame.caller current_forest)
//            ~else_:Token_id.default )
//   in
//   (* Cases:
//      - [account_update_forest] is empty, [remainder_of_current_forest] is empty.
//      Pop from the call stack to get another forest, which is guaranteed to be non-empty.
//      The result of popping becomes the "current forest".
//      - [account_update_forest] is empty, [remainder_of_current_forest] is non-empty.
//      Push nothing to the stack. [remainder_of_current_forest] becomes new "current forest"
//      - [account_update_forest] is non-empty, [remainder_of_current_forest] is empty.
//      Push nothing to the stack. [account_update_forest] becomes new "current forest"
//      - [account_update_forest] is non-empty, [remainder_of_current_forest] is non-empty:
//      Push [remainder_of_current_forest] to the stack. [account_update_forest] becomes new "current forest".
//   *)
//   let account_update_forest_empty =
//     Call_forest.is_empty account_update_forest
//   in
//   let remainder_of_current_forest_empty =
//     Call_forest.is_empty remainder_of_current_forest
//   in
//   let newly_popped_frame, popped_call_stack = pop_call_stack call_stack in
//   let remainder_of_current_forest_frame : Stack_frame.t =
//     Stack_frame.make
//       ~caller:(Stack_frame.caller current_forest)
//       ~caller_caller:(Stack_frame.caller_caller current_forest)
//       ~calls:remainder_of_current_forest
//   in
//   let new_call_stack =
//     Call_stack.if_ account_update_forest_empty
//       ~then_:
//         (Call_stack.if_ remainder_of_current_forest_empty
//            ~then_:
//              (* Don't actually need the or_default used in this case. *)
//              popped_call_stack ~else_:call_stack )
//       ~else_:
//         (Call_stack.if_ remainder_of_current_forest_empty ~then_:call_stack
//            ~else_:
//              (Call_stack.push remainder_of_current_forest_frame
//                 ~onto:call_stack ) )
//   in
//   let new_frame =
//     Stack_frame.if_ account_update_forest_empty
//       ~then_:
//         (Stack_frame.if_ remainder_of_current_forest_empty
//            ~then_:newly_popped_frame ~else_:remainder_of_current_forest_frame )
//       ~else_:
//         (let caller =
//            Account_id.derive_token_id
//              ~owner:(Account_update.account_id account_update)
//          and caller_caller = caller_id in
//          Stack_frame.make ~calls:account_update_forest ~caller ~caller_caller
//         )
//   in
//   { account_update
//   ; caller_id
//   ; account_update_forest
//   ; new_frame
//   ; new_call_stack
//   }

type LocalState<Z> = LocalStateSkeleton<
    <Z as ZkappApplication>::Ledger,       // ledger
    <Z as ZkappApplication>::StackFrame,   // stack_frame
    <Z as ZkappApplication>::CallStack,    // call_stack
    Fp,                                    // commitments
    <Z as ZkappApplication>::SignedAmount, // fee_excess & supply_increase
    (),                                    // failure_status_tbl
    Boolean,                               // success & will_succeed
    <Z as ZkappApplication>::Index,        // account_update_index
>;

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
    _h: &Handler,
    (global_state, mut local_state): (Z::GlobalState, LocalState<Z>),
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

    let _ = {
        let (to_pop, call_stack) = {
            match &is_start {
                IsStart::Compute(start_data) => {
                    // We decompose this way because of OCaml evaluation order
                    let right = w.exists_no_check(match is_start2 {
                        Boolean::True => Z::CallStack::empty(),
                        Boolean::False => local_state.call_stack,
                    });

                    let left = {
                        let on_true = Z::StackFrame::make(
                            TokenId::default(),
                            TokenId::default(),
                            &start_data.account_updates,
                            w,
                        );
                        match is_start2 {
                            Boolean::True => on_true,
                            Boolean::False => local_state.stack_frame,
                        }
                        .on_if(w)
                    };

                    (left, right)
                }
                IsStart::Yes(start_data) => {
                    // We decompose this way because of OCaml evaluation order
                    let right = Z::CallStack::empty();
                    let left = Z::StackFrame::make(
                        TokenId::default(),
                        TokenId::default(),
                        &start_data.account_updates,
                        w,
                    );
                    (left, right)
                }
                IsStart::No => (local_state.stack_frame, local_state.call_stack),
            }
        };

        get_next_account_update::<Z>(to_pop, call_stack, w);
    };

    //   let { account_update
    //       ; caller_id
    //       ; account_update_forest
    //       ; new_frame = remaining
    //       ; new_call_stack = call_stack
    //       } =
    //     with_label ~label:"get next account update" (fun () ->
    //         (* TODO: Make the stack frame hashed inside of the local state *)
    //         get_next_account_update to_pop call_stack )
    //   in
    //   let local_state =
    //     with_label ~label:"token owner not caller" (fun () ->
    //         let default_token_or_token_owner_was_caller =
    //           (* Check that the token owner was consulted if using a non-default
    //              token *)
    //           let account_update_token_id =
    //             Account_update.token_id account_update
    //           in
    //           Bool.( ||| )
    //             (Token_id.equal account_update_token_id Token_id.default)
    //             (Token_id.equal account_update_token_id caller_id)
    //         in
    //         Local_state.add_check local_state Token_owner_not_caller
    //           default_token_or_token_owner_was_caller )
    //   in

    eprintln!("DONE");
    std::process::exit(0);
}
