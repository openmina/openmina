use mina_hasher::Fp;

use crate::{
    proofs::witness::ToBoolean,
    scan_state::{currency, scan_state::ConstraintConstants},
    sparse_ledger::LedgerIntf,
};

use super::{
    witness::{Boolean, Witness},
    zkapp::{GlobalStateForProof, LocalStateForProof, StartDataForProof},
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

pub fn apply(
    constraint_constants: &ConstraintConstants,
    is_start: IsStart<StartDataForProof>,
    h: &Handler,
    (global_state, local_state): (GlobalStateForProof, LocalStateForProof),
    w: &mut Witness<Fp>,
) -> Result<(GlobalStateForProof, LocalStateForProof), String> {
    let is_start2 = {
        let is_empty_call_forest = local_state.stack_frame.calls.is_empty().to_boolean();

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

    let will_succeed = match is_start {
        IsStart::Compute(start_data) => w.exists_no_check(match is_start2 {
            Boolean::True => start_data.will_succeed,
            Boolean::False => local_state.will_succeed,
        }),
        IsStart::Yes(start_data) => start_data.will_succeed,
        IsStart::No => local_state.will_succeed,
    };

    // local_state.ledger =

    // global_state.first_pass_ledger.1
    // local_state.led

    &global_state.first_pass_ledger;
    &local_state.ledger;

    // let local_state =
    //   { local_state with
    //     ledger =
    //       Inputs.Ledger.if_ is_start'
    //         ~then_:(Inputs.Global_state.first_pass_ledger global_state)
    //         ~else_:local_state.ledger
    //   ; will_succeed
    //   }
    // in

    todo!()
}
