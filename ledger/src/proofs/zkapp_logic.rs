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
    <<Z as ZkappApplication>::StackFrame as StackFrameInterface>::Calls, // account_updates
    Boolean,                                                             // will_succeed
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
        let _ = {
            match &is_start {
                IsStart::Compute(start_data) => {
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

                        w.exists_no_check(match is_start2 {
                            Boolean::True => on_true,
                            Boolean::False => local_state.stack_frame,
                        })
                    };
                }
                IsStart::Yes(_) => todo!(),
                IsStart::No => todo!(),
            }
        };
    };

    // let ( (account_update, remaining, call_stack)
    //     , account_update_forest
    //     , local_state
    //     , (a, inclusion_proof) ) =
    //   let to_pop, call_stack =
    //     match is_start with
    //     | `Compute start_data ->
    //         ( Stack_frame.if_ is_start'
    //             ~then_:
    //               (Stack_frame.make ~calls:start_data.account_updates
    //                  ~caller:default_caller ~caller_caller:default_caller )
    //             ~else_:local_state.stack_frame
    //         , Call_stack.if_ is_start' ~then_:(Call_stack.empty ())
    //             ~else_:local_state.call_stack )
    //     | `Yes start_data ->
    //         ( Stack_frame.make ~calls:start_data.account_updates
    //             ~caller:default_caller ~caller_caller:default_caller
    //         , Call_stack.empty () )
    //     | `No ->
    //         (local_state.stack_frame, local_state.call_stack)
    //   in
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

    todo!()
}
