#![allow(unused)]

use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_p2p_messages::v2;

use crate::{
    scan_state::{
        currency::{Amount, Signed, Slot},
        pending_coinbase,
        scan_state::transaction_snark::SokMessage,
        transaction_logic::{
            protocol_state::protocol_state_body_view, zkapp_command::ZkAppCommand,
        },
    },
    sparse_ledger::SparseLedger,
};

use super::witness::{Prover, Witness};

pub struct ZkappParams<'a> {
    pub statement: &'a v2::MinaStateBlockchainStateValueStableV2LedgerProofStatement,
    pub tx_witness: &'a v2::TransactionWitnessStableV2,
    pub message: &'a SokMessage,
    pub step_prover: &'a Prover<Fp>,
    // pub tx_wrap_prover: &'a Prover<Fq>,
    /// For debugging only
    pub expected_step_proof: Option<&'static str>,
    /// For debugging only
    pub ocaml_wrap_witness: Option<Vec<Fq>>,
}

struct ZkappCommandWitnessesParams<'a> {
    global_slot: Slot,
    state_body: &'a v2::MinaStateProtocolStateBodyValueStableV2,
    fee_excess: Signed<Amount>,
    pending_coinbase_init_stack: pending_coinbase::Stack,
    pending_coinbase_of_statement: pending_coinbase::StackState,
    first_pass_ledger: SparseLedger,
    second_pass_ledger: SparseLedger,
    connecting_ledger_hash: Fp,
    zkapp_command: &'a ZkAppCommand,
}

fn zkapp_command_witnesses_exn(params: ZkappCommandWitnessesParams) {
    let ZkappCommandWitnessesParams {
        global_slot,
        state_body,
        fee_excess,
        pending_coinbase_init_stack,
        pending_coinbase_of_statement,
        mut first_pass_ledger,
        mut second_pass_ledger,
        connecting_ledger_hash,
        zkapp_command,
    } = params;

    let supply_increase = Signed::<Amount>::zero();
    let state_view = protocol_state_body_view(state_body);

    let (will_succeed, states) = {
        let (txn_applied, states) = {
            let (partial_txn, states) = first_pass_ledger
                .apply_zkapp_first_pass_unchecked_with_states(
                    global_slot,
                    &state_view,
                    fee_excess,
                    supply_increase,
                    &second_pass_ledger,
                    zkapp_command,
                )
                .unwrap();

            second_pass_ledger
                .apply_zkapp_second_pass_unchecked_with_states(states, partial_txn)
                .unwrap()
        };

        let will_succeed = txn_applied.command.status.is_applied();

        let states_with_connecting_ledger = states
            .iter()
            .cloned()
            .map(|(global, local)| (global, local, connecting_ledger_hash))
            .collect::<Vec<_>>();

        (will_succeed, states_with_connecting_ledger)
    };

    dbg!(states.len());

    // let will_succeeds = List.rev will_succeeds_rev in
    // let states = List.rev states_rev in
    // let states_rev =
    //   Account_update_group.group_by_zkapp_command_rev
    //     (List.map
    //        ~f:(fun (_, _, _, _, _, zkapp_command) -> zkapp_command)
    //        zkapp_commands_with_context )
    //     ([ List.hd_exn (List.hd_exn states) ] :: states)
    // in
    // let commitment = ref (Local_state.dummy ()).transaction_commitment in
    // let full_commitment =
    //   ref (Local_state.dummy ()).full_transaction_commitment
    // in
    // let remaining_zkapp_command =
    //   let zkapp_commands =
    //     List.map2_exn zkapp_commands_with_context will_succeeds
    //       ~f:(fun
    //            ( pending_coinbase_init_stack
    //            , pending_coinbase_stack_state
    //            , _
    //            , _
    //            , _
    //            , account_updates )
    //            will_succeed
    //          ->
    //         ( pending_coinbase_init_stack
    //         , pending_coinbase_stack_state
    //         , { Mina_transaction_logic.Zkapp_command_logic.Start_data
    //             .account_updates
    //           ; memo_hash = Signed_command_memo.hash account_updates.memo
    //           ; will_succeed
    //           } ) )
    //   in
    //   ref zkapp_commands
    // in
    // let pending_coinbase_init_stack = ref Pending_coinbase.Stack.empty in
    // let pending_coinbase_stack_state =
    //   ref
    //     { Pending_coinbase_stack_state.source = Pending_coinbase.Stack.empty
    //     ; target = Pending_coinbase.Stack.empty
    //     }
    // in
}

pub fn generate_zkapp_proof(params: ZkappParams, w: &mut Witness<Fp>) {
    let ZkappParams {
        statement,
        tx_witness,
        message,
        step_prover,
        expected_step_proof,
        ocaml_wrap_witness,
    } = params;

    let zkapp = match &tx_witness.transaction {
        v2::MinaTransactionTransactionStableV2::Command(cmd) => {
            let v2::MinaBaseUserCommandStableV2::ZkappCommand(zkapp_command) = &**cmd else {
                unreachable!();
            };
            zkapp_command
        }
        _ => unreachable!(),
    };

    let zkapp_command: ZkAppCommand = zkapp.into();

    let params = ZkappCommandWitnessesParams {
        global_slot: Slot::from_u32(tx_witness.block_global_slot.as_u32()),
        state_body: &tx_witness.protocol_state_body,
        fee_excess: Signed::zero(),
        pending_coinbase_init_stack: (&tx_witness.init_stack).into(),
        pending_coinbase_of_statement: pending_coinbase::StackState {
            source: (&statement.source.pending_coinbase_stack).into(),
            target: (&statement.target.pending_coinbase_stack).into(),
        },
        first_pass_ledger: (&tx_witness.first_pass_ledger).into(),
        second_pass_ledger: (&tx_witness.second_pass_ledger).into(),
        connecting_ledger_hash: statement.connecting_ledger_left.to_field(),
        zkapp_command: &zkapp_command,
    };

    zkapp_command_witnesses_exn(params);
}
