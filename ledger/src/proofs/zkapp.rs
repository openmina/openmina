#![allow(unused)]

use ark_ff::Zero;
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_p2p_messages::v2;

use crate::{
    hash_with_kimchi,
    proofs::zkapp::group::{State, ZkappCommandIntermediateState},
    scan_state::{
        currency::{Amount, Signed, Slot},
        pending_coinbase::{self, Stack, StackState},
        scan_state::transaction_snark::SokMessage,
        transaction_logic::{
            local_state::{LocalState, LocalStateEnv},
            protocol_state::{protocol_state_body_view, GlobalState},
            zkapp_command::{AccountUpdate, ZkAppCommand},
            zkapp_statement::TransactionCommitment,
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

    #[derive(Debug)]
    pub enum SegmentBasic {
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

    #[derive(Debug)]
    pub enum Kind {
        New,
        Same,
        TwoNew,
    }

    #[derive(Debug)]
    pub struct State {
        pub global: GlobalState<SparseLedger>,
        pub local: LocalStateEnv<SparseLedger>,
    }

    #[derive(Debug)]
    pub struct ZkappCommandIntermediateState {
        pub kind: Kind,
        pub spec: SegmentBasic,
        pub state_before: State,
        pub state_after: State,
        pub connecting_ledger: Fp,
    }

    fn intermediate_state(
        kind: Kind,
        spec: SegmentBasic,
        before: &(GlobalState<SparseLedger>, LocalStateEnv<SparseLedger>, Fp),
        after: &(GlobalState<SparseLedger>, LocalStateEnv<SparseLedger>, Fp),
    ) -> ZkappCommandIntermediateState {
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
    }

    // Note: Unlike OCaml, the returned value (the list) is not reversed, but we keep the same method name
    pub fn group_by_zkapp_command_rev(
        zkapp_command: Vec<&ZkAppCommand>,
        stmtss: Vec<Vec<(GlobalState<SparseLedger>, LocalStateEnv<SparseLedger>, Fp)>>,
    ) -> Vec<ZkappCommandIntermediateState> {
        let mut zkapp_account_updatess = zkapp_command
            .iter()
            .map(|zkapp_command| zkapp_command.all_account_updates_list())
            .collect::<Vec<_>>();
        zkapp_account_updatess.insert(0, vec![]);

        let mut acc = Vec::<ZkappCommandIntermediateState>::with_capacity(32);

        // Convert to slices, to allow matching below
        let zkapp_account_updatess = zkapp_account_updatess
            .iter()
            .map(|v| v.as_slice())
            .collect::<Vec<_>>();
        let stmtss = stmtss.iter().map(|v| v.as_slice()).collect::<Vec<_>>();

        #[rustfmt::skip]
        fn group_by_zkapp_command_rev_impl(
            zkapp_commands: &[&[AccountUpdate]],
            stmtss: &[&[(GlobalState<SparseLedger>, LocalStateEnv<SparseLedger>, Fp)]],
            acc: &mut Vec<ZkappCommandIntermediateState>,
        ) {
            use Kind::{New, Same, TwoNew};
            use Control::{Proof, Signature, NoneGiven};

            let to_spec = |c: &Control| SegmentBasic::of_controls(&[c]);
            let to_specs = |c1: &Control, c2: &Control| SegmentBasic::of_controls(&[c1, c2]);

            fn prepend<'a, T: ?Sized>(value: &'a T, list: &[&'a T]) -> Vec<&'a T> {
                // `T` here is actually a slice
                let mut new_list = Vec::with_capacity(list.len() + 1);
                new_list.push(value);
                new_list.extend(list);
                new_list
            }

            // I don't take responsability for this code, see OCaml comments
            // https://github.com/MinaProtocol/mina/blob/78535ae3a73e0e90c5f66155365a934a15535779/src/lib/mina_base/zkapp_command.ml#L1590
            match (zkapp_commands, stmtss) {
                (([] | [[]]), [ _ ]) => {
                    eprintln!("GROUP 1");
                    return;
                },
                ([[ AccountUpdate { authorization: a1, .. } ]], [[ before, after ]]) => {
                    eprintln!("GROUP 2");
                    acc.push(intermediate_state(Same, to_spec(a1), before, after));
                }
                ([[], [AccountUpdate { authorization: a1, .. }]], [[ _ ], [ before, after ]]) => {
                    eprintln!("GROUP 3");
                    acc.push(intermediate_state(New, to_spec(a1), before, after));
                }
                ([[AccountUpdate { authorization: a1 @ Proof(_), .. }, zkapp_command @ ..], zkapp_commands @ ..],
                 [stmts @ [ before, after, ..], stmtss @ .. ]
                ) => {
                    eprintln!("GROUP 4");
                    let stmts = &stmts[1..];
                    let zkapp_commands = prepend(zkapp_command, zkapp_commands);
                    let stmtss = prepend(stmts, stmtss);

                    acc.push(intermediate_state(Same, to_spec(a1), before, after));
                    group_by_zkapp_command_rev_impl(zkapp_commands.as_slice(), stmtss.as_slice(), acc);
                }
                ([[], [AccountUpdate { authorization: a1 @ Proof(_), .. }, zkapp_command @ .. ], zkapp_commands @ ..],
                 [[ _ ], stmts @ [ before, after, ..], stmtss @ ..]
                ) => {
                    eprintln!("GROUP 5");
                    let stmts = &stmts[1..];
                    let zkapp_commands = prepend(zkapp_command, zkapp_commands);
                    let stmtss = prepend(stmts, stmtss);

                    acc.push(intermediate_state(New, to_spec(a1), before, after));
                    group_by_zkapp_command_rev_impl(zkapp_commands.as_slice(), stmtss.as_slice(), acc);
                }
                ([zkapp_command @ [AccountUpdate { authorization: a1, .. }, AccountUpdate { authorization: Proof(_), .. }, ..], zkapp_commands @ ..],
                 [stmts @ [before, after, ..], stmtss @ ..]
                ) => {
                    eprintln!("GROUP 6");
                    let stmts = &stmts[1..];
                    let zkapp_command = &zkapp_command[1..];
                    let zkapp_commands = prepend(zkapp_command, zkapp_commands);
                    let stmtss = prepend(stmts, stmtss);

                    acc.push(intermediate_state(Same, to_spec(a1), before, after));
                    group_by_zkapp_command_rev_impl(zkapp_commands.as_slice(), stmtss.as_slice(), acc);
                }
                (zkapp_commands @ [[AccountUpdate { authorization: a1, .. }], zkapp_command @ [], [AccountUpdate { authorization: Proof(_), .. }, ..], ..],
                 [stmts @ [before, after, ..], stmtss @ ..]
                ) => {
                    eprintln!("GROUP 7");
                    let stmts = &stmts[1..];
                    let zkapp_commands = &zkapp_commands[2..];
                    let zkapp_commands = prepend(*zkapp_command, zkapp_commands);
                    let stmtss = prepend(stmts, stmtss);

                    acc.push(intermediate_state(Same, to_spec(a1), before, after));
                    group_by_zkapp_command_rev_impl(zkapp_commands.as_slice(), stmtss.as_slice(), acc);
                }
                ([[AccountUpdate { authorization: a1 @ (Signature(_) | NoneGiven), .. },
                   AccountUpdate { authorization: a2 @ (Signature(_) | NoneGiven), .. },
                   zkapp_command @ ..], zkapp_commands @ ..],
                 [stmts @ [before, _, after, ..], stmtss @ ..]
                ) => {
                    eprintln!("GROUP 8");
                    let stmts = &stmts[2..];
                    let zkapp_commands = prepend(zkapp_command, zkapp_commands);
                    let stmtss = prepend(stmts, stmtss);

                    acc.push(intermediate_state(Same, to_specs(a1, a2), before, after));
                    group_by_zkapp_command_rev_impl(zkapp_commands.as_slice(), stmtss.as_slice(), acc);
                }
                ([[], zkapp_command @ [AccountUpdate { authorization: a1, .. }, AccountUpdate { authorization: Proof(_), .. }, ..], zkapp_commands @ ..],
                 ([[ _ ], stmts @ [before, after, ..], stmtss @ ..])
                ) => {
                    eprintln!("GROUP 9");
                    let stmts = &stmts[1..];
                    let zkapp_command = &zkapp_command[1..];
                    let zkapp_commands = prepend(zkapp_command, zkapp_commands);
                    let stmtss = prepend(stmts, stmtss);

                    acc.push(intermediate_state(New, to_spec(a1), before, after));
                    group_by_zkapp_command_rev_impl(zkapp_commands.as_slice(), stmtss.as_slice(), acc);
                }
                ([[], [AccountUpdate { authorization: a1 @ (Signature(_) | NoneGiven), .. },
                       AccountUpdate { authorization: a2 @ (Signature(_) | NoneGiven), .. },
                       zkapp_command @ ..], zkapp_commands @ ..],
                 [[ _ ], stmts @ [before, _, after, ..], stmtss @ ..] ) => {
                    eprintln!("GROUP 10");
                    let stmts = &stmts[2..];
                    let zkapp_commands = prepend(zkapp_command, zkapp_commands);
                    let stmtss = prepend(stmts, stmtss);

                    acc.push(intermediate_state(New, to_specs(a1, a2), before, after));
                    group_by_zkapp_command_rev_impl(zkapp_commands.as_slice(), stmtss.as_slice(), acc);
                }
                ([[AccountUpdate { authorization: a1 @ (Signature(_) | NoneGiven), .. }],
                  [AccountUpdate { authorization: a2 @ (Signature(_) | NoneGiven), .. }, zkapp_command @ ..],
                  zkapp_commands @ ..],
                 [[before, _after1], stmts @ [_before2, after, ..], stmtss @ .. ]
                ) => {
                    eprintln!("GROUP 11");
                    let stmts = &stmts[1..];
                    let zkapp_commands = prepend(zkapp_command, zkapp_commands);
                    let stmtss = prepend(stmts, stmtss);

                    acc.push(intermediate_state(New, to_specs(a1, a2), before, after));
                    group_by_zkapp_command_rev_impl(zkapp_commands.as_slice(), stmtss.as_slice(), acc);
                }
                (zkapp_commands @ [[], [AccountUpdate { authorization: a1, .. }, zkapp_command @ ..],
                  [AccountUpdate { authorization: Proof(_), .. }, ..], ..],
                 stmtss @ [[ _ ], [before], stmts @ [after], _, ..]
                ) => {
                    eprintln!("GROUP 12");
                    let stmtss = &stmtss[3..];
                    let zkapp_commands = &zkapp_commands[2..];
                    let zkapp_commands = prepend(zkapp_command, zkapp_commands);
                    let stmtss = prepend(*stmts, stmtss);

                    acc.push(intermediate_state(New, to_spec(a1), before, after));
                    group_by_zkapp_command_rev_impl(zkapp_commands.as_slice(), stmtss.as_slice(), acc);
                }
                ([[], [AccountUpdate { authorization: a1 @ (Signature(_) | NoneGiven), .. }],
                  [AccountUpdate { authorization: a2 @ (Signature(_) | NoneGiven), .. }, zkapp_command @ ..],
                  zkapp_commands @ ..],
                 [[ _ ], [before, _after1], stmts @ [_before2, after, ..], stmtss @ ..]
                ) => {
                    eprintln!("GROUP 13");
                    let stmts = &stmts[1..];
                    let zkapp_commands = prepend(zkapp_command, zkapp_commands);
                    let stmtss = prepend(stmts, stmtss);

                    acc.push(intermediate_state(TwoNew, to_specs(a1, a2), before, after));
                    group_by_zkapp_command_rev_impl(zkapp_commands.as_slice(), stmtss.as_slice(), acc);
                }
                ([[AccountUpdate { authorization: a1, .. }]], [[before, after, ..], ..]) => {
                    eprintln!("GROUP 14");
                    acc.push(intermediate_state(Same, to_spec(a1), before, after));
                }
                ([[], [AccountUpdate { authorization: a1, .. }], [], ..], [[ _ ], [before, after, ..], ..]) => {
                    eprintln!("GROUP 15");
                    acc.push(intermediate_state(New, to_spec(a1), before, after));
                }
                _ => panic!("invalid")
            }
        }

        group_by_zkapp_command_rev_impl(&zkapp_account_updatess, &stmtss, &mut acc);
        acc
    }
}

pub struct StartData {
    pub account_updates: ZkAppCommand,
    pub memo_hash: Fp,
    pub will_succeed: bool,
}

// TODO: De-duplicate with the one in `transaction_logic.rs`
#[derive(Debug, Clone, PartialEq)]
pub struct WithStackHash<T> {
    pub elt: T,
    pub stack_hash: Fp,
}

fn accumulate_call_stack_hashes<Frame>(
    hash_frame: impl Fn(&Frame) -> Fp,
    frames: &[Frame],
) -> Vec<WithStackHash<&Frame>> {
    match frames {
        [] => vec![],
        [f, fs @ ..] => {
            let h_f = hash_frame(f);
            let mut tl = accumulate_call_stack_hashes(hash_frame, fs);
            let h_tl = match tl.as_slice() {
                [] => Fp::zero(),
                [t, ..] => t.stack_hash,
            };

            tl.insert(
                0,
                WithStackHash {
                    elt: f,
                    stack_hash: hash_with_kimchi("MinaAcctUpdateCons", &[h_f, h_tl]),
                },
            );

            tl
        }
    }
}

pub struct ZkappCommandsWithContext<'a> {
    pub pending_coinbase_init_stack: pending_coinbase::Stack,
    pub pending_coinbase_of_statement: pending_coinbase::StackState,
    pub first_pass_ledger: SparseLedger,
    pub second_pass_ledger: SparseLedger,
    pub connecting_ledger_hash: Fp,
    pub zkapp_command: &'a ZkAppCommand,
}

pub struct ZkappCommandWitnessesParams<'a> {
    pub global_slot: Slot,
    pub state_body: &'a v2::MinaStateProtocolStateBodyValueStableV2,
    pub fee_excess: Signed<Amount>,
    pub zkapp_commands_with_context: Vec<ZkappCommandsWithContext<'a>>,
}

pub fn zkapp_command_witnesses_exn(params: ZkappCommandWitnessesParams) {
    let ZkappCommandWitnessesParams {
        global_slot,
        state_body,
        fee_excess,
        zkapp_commands_with_context,
    } = params;

    let supply_increase = Signed::<Amount>::zero();
    let state_view = protocol_state_body_view(state_body);

    let (_, _, will_succeeds, mut states) = zkapp_commands_with_context.iter().fold(
        (fee_excess, supply_increase, vec![], vec![]),
        |acc, v| {
            let (fee_excess, supply_increase, mut will_succeeds, mut statess) = acc;

            let ZkappCommandsWithContext {
                pending_coinbase_init_stack: _,
                pending_coinbase_of_statement: _,
                first_pass_ledger,
                second_pass_ledger,
                connecting_ledger_hash,
                zkapp_command,
            } = v;

            let (txn_applied, states) = {
                let (partial_txn, states) = first_pass_ledger
                    .clone()
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
                    .clone()
                    .apply_zkapp_second_pass_unchecked_with_states(states, partial_txn)
                    .unwrap()
            };

            let will_succeed = txn_applied.command.status.is_applied();

            let states_with_connecting_ledger = states
                .iter()
                .cloned()
                .map(|(global, local)| (global, local, *connecting_ledger_hash))
                .collect::<Vec<_>>();

            let final_state = {
                let (global_state, _local_state, _connecting_ledger) =
                    states_with_connecting_ledger.last().unwrap();
                global_state
            };

            let fee_excess = final_state.fee_excess;
            let supply_increase = final_state.supply_increase;
            will_succeeds.push(will_succeed);
            statess.push(states_with_connecting_ledger);

            (fee_excess, supply_increase, will_succeeds, statess)
        },
    );

    dbg!(
        states.len(),
        states.iter().map(|v| v.len()).collect::<Vec<_>>()
    );

    states.insert(0, vec![states[0][0].clone()]);
    let states = group::group_by_zkapp_command_rev(
        zkapp_commands_with_context
            .iter()
            .map(|v| v.zkapp_command)
            .collect(),
        states,
    );

    let (mut commitment, mut full_commitment) = {
        let LocalState {
            transaction_commitment,
            full_transaction_commitment,
            ..
        } = LocalState::dummy();
        (
            TransactionCommitment(transaction_commitment),
            TransactionCommitment(full_transaction_commitment),
        )
    };

    let remaining_zkapp_command = {
        let zkapp_commands = zkapp_commands_with_context
            .iter()
            .zip(will_succeeds)
            .map(|(v, will_succeed)| {
                let ZkappCommandsWithContext {
                    pending_coinbase_init_stack,
                    pending_coinbase_of_statement,
                    first_pass_ledger: _,
                    second_pass_ledger: _,
                    connecting_ledger_hash: _,
                    zkapp_command,
                } = v;

                (
                    pending_coinbase_init_stack,
                    pending_coinbase_of_statement,
                    StartData {
                        account_updates: (*zkapp_command).clone(),
                        memo_hash: zkapp_command.memo.hash(),
                        will_succeed,
                    },
                )
            })
            .collect::<Vec<_>>();

        zkapp_commands
    };
    let mut remaining_zkapp_command = remaining_zkapp_command.as_slice();

    let mut pending_coinbase_init_stack = Stack::empty();
    let mut pending_coinbase_stack_state = StackState {
        source: Stack::empty(),
        target: Stack::empty(),
    };

    states.into_iter().fold(vec![], |witnesses, s| {
        let ZkappCommandIntermediateState {
            kind,
            spec,
            state_before:
                State {
                    global: source_global,
                    local: mut source_local,
                },
            state_after:
                State {
                    global: target_global,
                    local: mut target_local,
                },
            connecting_ledger,
        } = s;

        source_local.failure_status_tbl = vec![];
        target_local.failure_status_tbl = vec![];

        let current_commitment = commitment;
        let current_full_commitment = full_commitment;

        let (
            start_zkapp_command,
            next_commitment,
            next_full_commitment,
            pending_coinbase_init_stack,
            pending_coinbase_stack_state,
        ) = {
            type TC = TransactionCommitment;

            let empty_if_last = |mk: Box<dyn Fn() -> (TC, TC) + '_>| -> (TC, TC) {
                let calls = target_local.stack_frame.calls.0.as_slice();
                let call_stack = target_local.call_stack.0.as_slice();

                match (calls, call_stack) {
                    ([], []) => (TC::empty(), TC::empty()),
                    _ => mk(),
                }
            };

            let mk_next_commitment = |zkapp_command: &ZkAppCommand| {
                empty_if_last(Box::new(|| {
                    let next_commitment = zkapp_command.commitment();
                    let memo_hash = zkapp_command.memo.hash();
                    let fee_payer_hash =
                        AccountUpdate::of_fee_payer(zkapp_command.fee_payer.clone()).digest();
                    let next_full_commitment =
                        next_commitment.create_complete(memo_hash, fee_payer_hash);

                    (next_commitment, next_full_commitment)
                }))
            };

            match kind {
                group::Kind::Same => {
                    let (next_commitment, next_full_commitment) =
                        empty_if_last(Box::new(|| (current_commitment, current_full_commitment)));
                    (
                        Vec::new(),
                        next_commitment,
                        next_full_commitment,
                        pending_coinbase_init_stack.clone(),
                        pending_coinbase_stack_state.clone(),
                    )
                }
                group::Kind::New => match remaining_zkapp_command {
                    [v, rest @ ..] => {
                        let (
                            pending_coinbase_init_stack1,
                            pending_coinbase_stack_state1,
                            zkapp_command,
                        ) = v;

                        let (commitment2, full_commitment2) =
                            mk_next_commitment(&zkapp_command.account_updates);

                        remaining_zkapp_command = rest;
                        commitment = commitment2;
                        full_commitment = full_commitment2;
                        pending_coinbase_init_stack = (*pending_coinbase_init_stack1).clone();
                        pending_coinbase_stack_state = (*pending_coinbase_stack_state1).clone();

                        (
                            vec![zkapp_command],
                            commitment2,
                            full_commitment2,
                            pending_coinbase_init_stack.clone(),
                            pending_coinbase_stack_state.clone(),
                        )
                    }
                    _ => panic!("Not enough remaining zkapp_command"),
                },
                group::Kind::TwoNew => match remaining_zkapp_command {
                    [v1, v2, rest @ ..] => {
                        let (
                            pending_coinbase_init_stack1,
                            pending_coinbase_stack_state1,
                            zkapp_command1,
                        ) = v1;
                        let (
                            pending_coinbase_init_stack2,
                            pending_coinbase_stack_state2,
                            zkapp_command2,
                        ) = v2;

                        let (commitment2, full_commitment2) =
                            mk_next_commitment(&zkapp_command2.account_updates);

                        remaining_zkapp_command = rest;
                        commitment = commitment2;
                        full_commitment = full_commitment2;
                        pending_coinbase_init_stack = (*pending_coinbase_init_stack1).clone();
                        pending_coinbase_stack_state = StackState {
                            target: pending_coinbase_stack_state2.target.clone(),
                            ..(*pending_coinbase_stack_state1).clone()
                        };

                        (
                            vec![zkapp_command1, zkapp_command2],
                            commitment2,
                            full_commitment2,
                            pending_coinbase_init_stack.clone(),
                            pending_coinbase_stack_state.clone(),
                        )
                    }
                    _ => panic!("Not enough remaining zkapp_command"),
                },
            }
        };

        let hash_local_state = |local: &LocalStateEnv<SparseLedger>| {
            local.call_stack.iter().map(|v| v.digest());
        };

        // let hash_local_state
        //         (local :
        //           ( Stack_frame.value
        //           , Stack_frame.value list
        //           , _
        //           , _
        //           , _
        //           , _
        //           , _
        //           , _ )
        //           Mina_transaction_logic.Zkapp_command_logic.Local_state.t ) =
        //       { local with
        //         stack_frame = local.stack_frame
        //       ; call_stack =
        //           List.map local.call_stack
        //             ~f:(With_hash.of_data ~hash_data:Stack_frame.Digest.create)
        //           |> accumulate_call_stack_hashes ~hash_frame:(fun x ->
        //                  x.With_hash.hash )
        //       }
        //     in
        //     let source_local =
        //       { (hash_local_state source_local) with
        //         transaction_commitment = current_commitment
        //       ; full_transaction_commitment = current_full_commitment
        //       }
        //     in
        //     let target_local =
        //       { (hash_local_state target_local) with
        //         transaction_commitment = next_commitment
        //       ; full_transaction_commitment = next_full_commitment
        //       }
        //     in

        Vec::<usize>::new()
    });
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
        zkapp_commands_with_context: vec![ZkappCommandsWithContext {
            pending_coinbase_init_stack: (&tx_witness.init_stack).into(),
            pending_coinbase_of_statement: pending_coinbase::StackState {
                source: (&statement.source.pending_coinbase_stack).into(),
                target: (&statement.target.pending_coinbase_stack).into(),
            },
            first_pass_ledger: (&tx_witness.first_pass_ledger).into(),
            second_pass_ledger: (&tx_witness.second_pass_ledger).into(),
            connecting_ledger_hash: statement.connecting_ledger_left.to_field(),
            zkapp_command: &zkapp_command,
        }],
    });
}
