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
            local_state::{LocalState, LocalStateEnv},
            protocol_state::{protocol_state_body_view, GlobalState},
            zkapp_command::ZkAppCommand,
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

mod group {
    use super::*;
    use crate::scan_state::transaction_logic::zkapp_command::{AccountUpdate, Control};

    enum SegmentBasic {
        OptSignedOptSigned,
        OptSigned,
        Proved,
    }

    impl SegmentBasic {
        pub fn of_controls(controls: &[&Control]) -> Self {
            use Control::{NoneGiven, Proof, Signature};

            match controls {
                [Proof(_)] => Self::Proved,
                [Signature(_) | NoneGiven] => Self::OptSigned,
                [Signature(_) | NoneGiven, Signature(_) | NoneGiven] => Self::OptSignedOptSigned,
                _ => panic!("Unsupported combination"),
            }
        }
    }

    enum Kind {
        New,
        Same,
        TwoNew,
    }

    struct State {
        global: GlobalState<SparseLedger>,
        local: LocalStateEnv<SparseLedger>,
    }

    struct ZkappCommandIntermediateState {
        kind: Kind,
        spec: SegmentBasic,
        state_before: State,
        state_after: State,
        connecting_ledger: Fp,
    }

    pub fn group_by_zkapp_command_rev(
        zkapp_command: &ZkAppCommand,
        stmtss: Vec<Vec<(GlobalState<SparseLedger>, LocalStateEnv<SparseLedger>, Fp)>>,
    ) {
        let intermediate_state =
            |kind: Kind,
             spec: SegmentBasic,
             before: &(GlobalState<SparseLedger>, LocalStateEnv<SparseLedger>, Fp),
             after: &(GlobalState<SparseLedger>, LocalStateEnv<SparseLedger>, Fp)| {
                let (global_before, local_before, _) = before;
                let (global_after, local_after, connecting_ledger) = after;
                ZkappCommandIntermediateState {
                    kind,
                    spec,
                    state_before: State {
                        global: global_before.clone(),
                        local: local_before.clone(),
                    },
                    state_after: State {
                        global: global_after.clone(),
                        local: local_after.clone(),
                    },
                    connecting_ledger: connecting_ledger.clone(),
                }
            };

        let zkapp_account_updatess = vec![vec![], zkapp_command.all_account_updates_list()];

        dbg!(zkapp_account_updatess
            .iter()
            .map(|v| v.len())
            .collect::<Vec<_>>());
        dbg!(stmtss.iter().map(|v| v.len()).collect::<Vec<_>>());

        let mut acc = Vec::<ZkappCommandIntermediateState>::with_capacity(32);

        let zkapp_account_updatess = zkapp_account_updatess
            .iter()
            .map(|v| v.as_slice())
            .collect::<Vec<_>>();
        let stmtss = stmtss.iter().map(|v| v.as_slice()).collect::<Vec<_>>();

        let mut zkapp_commands = zkapp_account_updatess.as_slice();
        let mut stmtss = stmtss.as_slice();

        #[rustfmt::skip]
        let res = loop {
            match (zkapp_commands, stmtss) {
                (([] | [[]]), [ _ ]) => {
                    break acc;
                },
                ([[ AccountUpdate { authorization: a1, .. } ]], [[ before, after]]) => {
                    let s = intermediate_state(Kind::Same, SegmentBasic::of_controls(&[a1]), before, after);
                    acc.push(s);
                }
                _ => todo!()
            }
        };

        // match (zkapp_commands, stmtss) with
        // | ([] | [ [] ]), [ _ ] ->
        //     (* We've associated statements with all given zkapp_command. *)
        //     acc
        // | [ [ { authorization = a1; _ } ] ], [ [ before; after ] ] ->
        //     (* There are no later zkapp_command to pair this one with. Prove it on its
        //        own.
        //     *)
        //     intermediate_state ~kind:`Same
        //       ~spec:(zkapp_segment_of_controls [ a1 ])
        //       ~before ~after
        //     :: acc
        // | [ []; [ { authorization = a1; _ } ] ], [ [ _ ]; [ before; after ] ] ->
        //     (* This account_update is part of a new transaction, and there are no later
        //        zkapp_command to pair it with. Prove it on its own.
        //     *)
        //     intermediate_state ~kind:`New
        //       ~spec:(zkapp_segment_of_controls [ a1 ])
        //       ~before ~after
        //     :: acc
    }
}

pub struct ZkappCommandWitnessesParams<'a> {
    pub global_slot: Slot,
    pub state_body: &'a v2::MinaStateProtocolStateBodyValueStableV2,
    pub fee_excess: Signed<Amount>,
    pub pending_coinbase_init_stack: pending_coinbase::Stack,
    pub pending_coinbase_of_statement: pending_coinbase::StackState,
    pub first_pass_ledger: SparseLedger,
    pub second_pass_ledger: SparseLedger,
    pub connecting_ledger_hash: Fp,
    pub zkapp_command: &'a ZkAppCommand,
}

pub fn zkapp_command_witnesses_exn(params: ZkappCommandWitnessesParams) {
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

    let states = vec![vec![states[0].clone()], states];

    group::group_by_zkapp_command_rev(zkapp_command, states);

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

    zkapp_command_witnesses_exn(ZkappCommandWitnessesParams {
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
    });
}
