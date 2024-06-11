use std::{cell::RefCell, rc::Rc};

use ark_ff::{BigInteger256, Zero};
use kimchi::proof::PointEvaluations;
use mina_curves::pasta::Fq;
use mina_hasher::Fp;
use mina_p2p_messages::v2;

use crate::{
    hash_with_kimchi,
    proofs::{
        constants::{
            make_step_zkapp_data, StepMergeProof, StepZkappOptSignedOptSignedProof,
            WrapZkappOptSignedProof, WrapZkappProof,
        },
        field::{Boolean, CircuitVar, FieldWitness, ToBoolean},
        merge::{generate_merge_proof, MergeParams},
        public_input::{messages::MessagesForNextWrapProof, prepared_statement::DeferredValues},
        step::{
            extract_recursion_challenges, step, InductiveRule, OptFlag, PreviousProofStatement,
            StepParams, StepProof,
        },
        transaction::ReducedMessagesForNextStepProof,
        unfinalized::{AllEvals, EvalsWithPublicInput},
        verification::prev_evals_to_p2p,
        verifier_index::make_zkapp_verifier_index,
        wrap::{self, WrapParams, WrapProofState, WrapStatement},
        zkapp::group::{State, ZkappCommandIntermediateState},
    },
    scan_state::{
        currency::{Amount, Index, Signed, Slot},
        fee_excess::FeeExcess,
        pending_coinbase::{self, Stack, StackState},
        scan_state::transaction_snark::{Registers, SokDigest, SokMessage, Statement},
        transaction_logic::{
            local_state::{
                LocalState, LocalStateEnv, LocalStateSkeleton, StackFrame, StackFrameChecked,
            },
            protocol_state::{protocol_state_body_view, GlobalState, GlobalStateSkeleton},
            zkapp_command::{
                AccountUpdate, CallForest, Control, WithHash, ZkAppCommand,
                ACCOUNT_UPDATE_CONS_HASH_PARAM,
            },
            zkapp_statement::{TransactionCommitment, ZkappStatement},
            TransactionFailure,
        },
    },
    sparse_ledger::SparseLedger,
    zkapps::{
        snark::ZkappSnark,
        zkapp_logic::{self, ApplyZkappParams},
    },
    AccountId, ControlTag, ToInputs, TokenId,
};

use self::group::SegmentBasic;

use super::{
    constants::{
        ForWrapData, ProofConstants, StepZkappOptSignedProof, StepZkappProvedProof,
        WrapZkappProvedProof,
    },
    field::GroupAffine,
    gates::CIRCUIT_DIRECTORY,
    numbers::{
        currency::{CheckedAmount, CheckedSigned},
        nat::{CheckedIndex, CheckedSlot},
    },
    to_field_elements::ToFieldElements,
    transaction::{dummy_constraints, Check, ProofError, Prover},
    witness::Witness,
    wrap::WrapProof,
};

pub struct ZkappParams<'a> {
    pub statement: &'a v2::MinaStateBlockchainStateValueStableV2LedgerProofStatement,
    pub tx_witness: &'a v2::TransactionWitnessStableV2,
    pub message: &'a SokMessage,
    pub step_opt_signed_opt_signed_prover: &'a Prover<Fp>,
    pub step_opt_signed_prover: &'a Prover<Fp>,
    pub step_proof_prover: &'a Prover<Fp>,
    pub merge_step_prover: &'a Prover<Fp>,
    pub tx_wrap_prover: &'a Prover<Fq>,

    /// For debugging only
    pub opt_signed_path: Option<&'a str>,
    /// For debugging only
    pub proved_path: Option<&'a str>,
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
            connecting_ledger: *connecting_ledger,
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
                ([] | [[]], [ _ ]) => {
                    // eprintln!("GROUP 1");
                },
                ([[ AccountUpdate { authorization: a1, .. } ]], [[ before, after ]]) => {
                    // eprintln!("GROUP 2");
                    acc.push(intermediate_state(Same, to_spec(a1), before, after));
                }
                ([[], [AccountUpdate { authorization: a1, .. }]], [[ _ ], [ before, after ]]) => {
                    // eprintln!("GROUP 3");
                    acc.push(intermediate_state(New, to_spec(a1), before, after));
                }
                ([[AccountUpdate { authorization: a1 @ Proof(_), .. }, zkapp_command @ ..], zkapp_commands @ ..],
                 [stmts @ [ before, after, ..], stmtss @ .. ]
                ) => {
                    // eprintln!("GROUP 4");
                    let stmts = &stmts[1..];
                    let zkapp_commands = prepend(zkapp_command, zkapp_commands);
                    let stmtss = prepend(stmts, stmtss);

                    acc.push(intermediate_state(Same, to_spec(a1), before, after));
                    group_by_zkapp_command_rev_impl(zkapp_commands.as_slice(), stmtss.as_slice(), acc);
                }
                ([[], [AccountUpdate { authorization: a1 @ Proof(_), .. }, zkapp_command @ .. ], zkapp_commands @ ..],
                 [[ _ ], stmts @ [ before, after, ..], stmtss @ ..]
                ) => {
                    // eprintln!("GROUP 5");
                    let stmts = &stmts[1..];
                    let zkapp_commands = prepend(zkapp_command, zkapp_commands);
                    let stmtss = prepend(stmts, stmtss);

                    acc.push(intermediate_state(New, to_spec(a1), before, after));
                    group_by_zkapp_command_rev_impl(zkapp_commands.as_slice(), stmtss.as_slice(), acc);
                }
                ([zkapp_command @ [AccountUpdate { authorization: a1, .. }, AccountUpdate { authorization: Proof(_), .. }, ..], zkapp_commands @ ..],
                 [stmts @ [before, after, ..], stmtss @ ..]
                ) => {
                    // eprintln!("GROUP 6");
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
                    // eprintln!("GROUP 7");
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
                    // eprintln!("GROUP 8");
                    let stmts = &stmts[2..];
                    let zkapp_commands = prepend(zkapp_command, zkapp_commands);
                    let stmtss = prepend(stmts, stmtss);

                    acc.push(intermediate_state(Same, to_specs(a1, a2), before, after));
                    group_by_zkapp_command_rev_impl(zkapp_commands.as_slice(), stmtss.as_slice(), acc);
                }
                ([[], zkapp_command @ [AccountUpdate { authorization: a1, .. }, AccountUpdate { authorization: Proof(_), .. }, ..], zkapp_commands @ ..],
                 [[ _ ], stmts @ [before, after, ..], stmtss @ ..]
                ) => {
                    // eprintln!("GROUP 9");
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
                    // eprintln!("GROUP 10");
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
                    // eprintln!("GROUP 11");
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
                    // eprintln!("GROUP 12");
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
                    // eprintln!("GROUP 13");
                    let stmts = &stmts[1..];
                    let zkapp_commands = prepend(zkapp_command, zkapp_commands);
                    let stmtss = prepend(stmts, stmtss);

                    acc.push(intermediate_state(TwoNew, to_specs(a1, a2), before, after));
                    group_by_zkapp_command_rev_impl(zkapp_commands.as_slice(), stmtss.as_slice(), acc);
                }
                ([[AccountUpdate { authorization: a1, .. }]], [[before, after, ..], ..]) => {
                    // eprintln!("GROUP 14");
                    acc.push(intermediate_state(Same, to_spec(a1), before, after));
                }
                ([[], [AccountUpdate { authorization: a1, .. }], [], ..], [[ _ ], [before, after, ..], ..]) => {
                    // eprintln!("GROUP 15");
                    acc.push(intermediate_state(New, to_spec(a1), before, after));
                }
                _ => panic!("invalid")
            }
        }

        group_by_zkapp_command_rev_impl(&zkapp_account_updatess, &stmtss, &mut acc);
        acc
    }
}

pub type StartData = StartDataSkeleton<
    ZkAppCommand, // account_updates
    bool,         // will_succeed
>;

#[derive(Clone, Debug)]
pub struct StartDataSkeleton<AccountUpdates, WillSucceed> {
    pub account_updates: AccountUpdates,
    pub memo_hash: Fp,
    pub will_succeed: WillSucceed,
}

// TODO: De-duplicate with the one in `transaction_logic.rs`
#[derive(Debug, Clone, PartialEq)]
pub struct WithStackHash<T> {
    pub elt: T,
    pub stack_hash: Fp,
}

fn accumulate_call_stack_hashes(
    hash_frame: impl Fn(&WithHash<StackFrame>) -> Fp,
    frames: &[WithHash<StackFrame>],
) -> Vec<WithStackHash<WithHash<StackFrame>>> {
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
                    elt: f.clone(),
                    stack_hash: hash_with_kimchi(ACCOUNT_UPDATE_CONS_HASH_PARAM, &[h_f, h_tl]),
                },
            );

            tl
        }
    }
}

pub type LocalStateForWitness = LocalStateSkeleton<
    SparseLedger,                                       // ledger
    StackFrame,                                         // stack_frame
    WithHash<Vec<WithStackHash<WithHash<StackFrame>>>>, // call_stack
    TransactionCommitment,                              // commitments
    Signed<Amount>,                                     // excess & supply_increase
    Vec<Vec<TransactionFailure>>,                       // failure_status_tbl
    bool,                                               // success & will_succeed
    Index,                                              // account_update_index
>;

#[derive(Debug)]
pub struct ZkappCommandSegmentWitness<'a> {
    pub global_first_pass_ledger: SparseLedger,
    pub global_second_pass_ledger: SparseLedger,
    pub local_state_init: LocalStateForWitness,
    pub start_zkapp_command: Vec<StartData>,
    pub state_body: &'a v2::MinaStateProtocolStateBodyValueStableV2,
    pub init_stack: Stack,
    pub block_global_slot: Slot,
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

pub fn zkapp_command_witnesses_exn(
    params: ZkappCommandWitnessesParams,
) -> Vec<(
    ZkappCommandSegmentWitness<'_>,
    group::SegmentBasic,
    Statement<SokDigest>,
)> {
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
                        second_pass_ledger,
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

    let w = Vec::with_capacity(32);
    states.into_iter().fold(w, |mut witnesses, s| {
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
                            vec![zkapp_command.clone()],
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
                            _pending_coinbase_init_stack2,
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
                            vec![zkapp_command1.clone(), zkapp_command2.clone()],
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
            let call_stack = local
                .call_stack
                .iter()
                .map(|v| WithHash::of_data(v.clone(), |v| v.digest()))
                .collect::<Vec<_>>();
            let call_stack = accumulate_call_stack_hashes(|x| x.hash, &call_stack);

            let LocalStateEnv {
                stack_frame,
                call_stack: _,
                transaction_commitment,
                full_transaction_commitment,
                excess,
                supply_increase,
                ledger,
                success,
                account_update_index,
                failure_status_tbl,
                will_succeed,
            } = local.clone();

            LocalStateForWitness {
                stack_frame,
                call_stack: WithHash {
                    data: call_stack,
                    hash: Fp::zero(), // TODO
                },
                transaction_commitment: TransactionCommitment(transaction_commitment.0),
                full_transaction_commitment: TransactionCommitment(full_transaction_commitment.0),
                excess,
                supply_increase,
                ledger,
                success,
                account_update_index,
                failure_status_tbl,
                will_succeed,
            }
        };

        let source_local = LocalStateForWitness {
            transaction_commitment: current_commitment,
            full_transaction_commitment: current_full_commitment,
            ..hash_local_state(&source_local)
        };

        let target_local = LocalStateForWitness {
            transaction_commitment: next_commitment,
            full_transaction_commitment: next_full_commitment,
            ..hash_local_state(&target_local)
        };

        let w = ZkappCommandSegmentWitness {
            global_first_pass_ledger: source_global.first_pass_ledger.clone(),
            global_second_pass_ledger: source_global.second_pass_ledger.clone(),
            local_state_init: source_local.clone(),
            start_zkapp_command,
            state_body,
            init_stack: pending_coinbase_init_stack,
            block_global_slot: global_slot,
        };

        let fee_excess = {
            let fee_excess = target_global
                .fee_excess
                .add(&source_global.fee_excess.negate())
                .expect("unexpected fee excess");
            FeeExcess {
                fee_token_l: TokenId::default(),
                fee_excess_l: fee_excess.to_fee(),
                fee_token_r: TokenId::default(),
                fee_excess_r: Signed::zero(),
            }
        };

        let supply_increase = target_global
            .supply_increase
            .add(&source_global.supply_increase.negate())
            .expect("unexpected supply increase");

        let call_stack_hash = |s: &Vec<WithStackHash<WithHash<StackFrame>>>| {
            s.first().map(|v| v.stack_hash).unwrap_or_else(Fp::zero)
        };

        let statement = {
            let target_first_pass_ledger_root =
                target_global.first_pass_ledger.clone().merkle_root();

            let (source_local_ledger, target_local_ledger) = (
                source_local.ledger.clone().merkle_root(),
                target_local.ledger.clone().merkle_root(),
            );

            Statement::<SokDigest> {
                source: Registers {
                    first_pass_ledger: source_global.first_pass_ledger.clone().merkle_root(),
                    second_pass_ledger: source_global.second_pass_ledger.clone().merkle_root(),
                    pending_coinbase_stack: pending_coinbase_stack_state.source.clone(),
                    local_state: {
                        let LocalStateForWitness {
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
                        } = source_local;

                        LocalState {
                            stack_frame: stack_frame.digest(),
                            call_stack: call_stack_hash(&call_stack),
                            transaction_commitment: transaction_commitment.0,
                            full_transaction_commitment: full_transaction_commitment.0,
                            ledger: source_local_ledger,
                            excess,
                            supply_increase,
                            success,
                            account_update_index,
                            failure_status_tbl,
                            will_succeed,
                        }
                    },
                },
                target: Registers {
                    first_pass_ledger: target_first_pass_ledger_root,
                    second_pass_ledger: target_global.second_pass_ledger.clone().merkle_root(),
                    pending_coinbase_stack: pending_coinbase_stack_state.target.clone(),
                    local_state: {
                        let LocalStateForWitness {
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
                        } = target_local;

                        LocalState {
                            stack_frame: stack_frame.digest(),
                            call_stack: call_stack_hash(&call_stack),
                            transaction_commitment: transaction_commitment.0,
                            full_transaction_commitment: full_transaction_commitment.0,
                            ledger: target_local_ledger,
                            excess,
                            supply_increase,
                            success,
                            account_update_index,
                            failure_status_tbl,
                            will_succeed,
                        }
                    },
                },
                connecting_ledger_left: connecting_ledger,
                connecting_ledger_right: connecting_ledger,
                supply_increase,
                fee_excess,
                sok_digest: SokDigest::default(),
            }
        };

        witnesses.insert(0, (w, spec, statement));
        witnesses
    })
}

#[derive(Clone, Debug)]
pub enum IsStart {
    Yes,
    No,
    ComputeInCircuit,
}

#[derive(Clone, Debug)]
pub struct Spec {
    pub auth_type: ControlTag,
    pub is_start: IsStart,
}

fn basic_spec(s: &SegmentBasic) -> Box<[Spec]> {
    let opt_signed = || Spec {
        auth_type: ControlTag::Signature,
        is_start: IsStart::ComputeInCircuit,
    };

    match s {
        SegmentBasic::OptSignedOptSigned => Box::new([opt_signed(), opt_signed()]),
        SegmentBasic::OptSigned => Box::new([opt_signed()]),
        SegmentBasic::Proved => Box::new([Spec {
            auth_type: ControlTag::Proof,
            is_start: IsStart::No,
        }]),
    }
}

fn read_witnesses<F: FieldWitness>(path: &str) -> Vec<F> {
    let f = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(CIRCUIT_DIRECTORY)
            .join("witnesses")
            .join(path),
        // .join("zkapp_fps.txt"),
    )
    .unwrap();

    let fps = f
        .lines()
        .filter(|s| !s.is_empty())
        .map(|s| F::from_str(s).unwrap_or_else(|_| panic!()))
        .collect::<Vec<_>>();

    fps
}

struct CheckProtocolStateParams<'a> {
    pending_coinbase_stack_init: Stack,
    pending_coinbase_stack_before: Stack,
    pending_coinbase_stack_after: Stack,
    block_global_slot: CheckedSlot<Fp>,
    state_body: &'a v2::MinaStateProtocolStateBodyValueStableV2,
}

fn check_protocol_state(params: CheckProtocolStateParams, w: &mut Witness<Fp>) {
    let CheckProtocolStateParams {
        pending_coinbase_stack_init,
        pending_coinbase_stack_before,
        pending_coinbase_stack_after,
        block_global_slot,
        state_body,
    } = params;

    let state_body_hash = state_body.checked_hash_with_param("MinaProtoStateBody", w);
    let global_slot = block_global_slot;
    let computed_pending_coinbase_stack_after =
        pending_coinbase_stack_init.checked_push_state(state_body_hash, global_slot, w);

    let _correct_coinbase_target_stack =
        computed_pending_coinbase_stack_after.equal_var(&pending_coinbase_stack_after, w);

    let _valid_init_state = {
        let equal_source = pending_coinbase_stack_init.equal_var(&pending_coinbase_stack_before, w);

        let equal_source_with_state =
            computed_pending_coinbase_stack_after.equal_var(&pending_coinbase_stack_before, w);

        equal_source.or(&equal_source_with_state, w)
    };
}

/// With root hash
#[derive(Clone)]
pub struct LedgerWithHash {
    pub ledger: SparseLedger,
    pub hash: Fp,
}

impl ToFieldElements<Fp> for LedgerWithHash {
    fn to_field_elements(&self, fields: &mut Vec<Fp>) {
        let Self { ledger: _, hash } = self;
        hash.to_field_elements(fields);
    }
}

impl Check<Fp> for LedgerWithHash {
    fn check(&self, _w: &mut Witness<Fp>) {
        // Nothing
    }
}

pub struct ZkappSingleData {
    spec: Spec,
    zkapp_input: Rc<RefCell<Option<ZkappStatement>>>,
    must_verify: Rc<RefCell<Boolean>>,
}

impl ZkappSingleData {
    pub fn spec(&self) -> &Spec {
        &self.spec
    }
    pub fn set_zkapp_input(&self, x: ZkappStatement) {
        let mut zkapp_input = self.zkapp_input.borrow_mut();
        zkapp_input.replace(x);
    }
    pub fn set_must_verify(&self, x: Boolean) {
        let mut must_verify = self.must_verify.borrow_mut();
        *must_verify = x;
    }
}

pub type LocalStateForProof = LocalStateSkeleton<
    LedgerWithHash,                                     // ledger
    StackFrameChecked,                                  // stack_frame
    WithHash<Vec<WithStackHash<WithHash<StackFrame>>>>, // call_stack
    Fp,                                                 // commitments
    CheckedSigned<Fp, CheckedAmount<Fp>>,               // fee_excess & supply_increase
    (),                                                 // failure_status_tbl
    Boolean,                                            // success & will_succeed
    CheckedIndex<Fp>,                                   // account_update_index
>;

pub type GlobalStateForProof = GlobalStateSkeleton<
    LedgerWithHash,                       // ledger
    CheckedSigned<Fp, CheckedAmount<Fp>>, // fee_excess & supply_increase
    CheckedSlot<Fp>,                      // block_global_slot
>;

pub type StartDataForProof = StartDataSkeleton<
    WithHash<CallForest<AccountUpdate>>, // account_updates
    CircuitVar<Boolean>,                 // will_succeed
>;

fn zkapp_main(
    statement: Statement<SokDigest>,
    witness: &ZkappCommandSegmentWitness,
    spec: &[Spec],
    w: &mut Witness<Fp>,
) -> (Option<ZkappStatement>, Boolean) {
    w.exists(&statement);

    dummy_constraints(w);
    let state_body = w.exists(witness.state_body);
    let block_global_slot = w.exists(witness.block_global_slot).to_checked();
    let pending_coinbase_stack_init = w.exists(witness.init_stack.clone());

    check_protocol_state(
        CheckProtocolStateParams {
            pending_coinbase_stack_init,
            pending_coinbase_stack_before: statement.source.pending_coinbase_stack.clone(),
            pending_coinbase_stack_after: statement.target.pending_coinbase_stack.clone(),
            block_global_slot,
            state_body,
        },
        w,
    );

    let (mut global, mut local) = {
        let g = GlobalStateForProof {
            first_pass_ledger: LedgerWithHash {
                ledger: witness.global_first_pass_ledger.clone(),
                hash: statement.source.first_pass_ledger,
            },
            second_pass_ledger: LedgerWithHash {
                ledger: witness.global_second_pass_ledger.clone(),
                hash: statement.source.second_pass_ledger,
            },
            fee_excess: CheckedSigned::zero(),
            supply_increase: CheckedSigned::zero(),
            protocol_state: protocol_state_body_view(state_body),
            block_global_slot,
        };

        let l = zkapp_logic::LocalState::<ZkappSnark> {
            stack_frame: witness
                .local_state_init
                .stack_frame
                .unhash(statement.source.local_state.stack_frame, w),
            call_stack: WithHash {
                hash: statement.source.local_state.call_stack,
                data: witness.local_state_init.call_stack.data.clone(),
            },
            transaction_commitment: statement.source.local_state.transaction_commitment,
            full_transaction_commitment: statement.source.local_state.full_transaction_commitment,
            excess: statement.source.local_state.excess.to_checked(),
            supply_increase: statement.source.local_state.supply_increase.to_checked(),
            ledger: LedgerWithHash {
                ledger: witness.local_state_init.ledger.copy_content(),
                hash: statement.source.local_state.ledger,
            },
            success: statement.source.local_state.success.to_boolean().var(),
            account_update_index: statement
                .source
                .local_state
                .account_update_index
                .to_checked(),
            failure_status_tbl: (),
            will_succeed: statement.source.local_state.will_succeed.to_boolean().var(),
        };

        (g, l)
    };

    let init_fee_excess = global.fee_excess.clone();

    let mut start_zkapp_command = witness.start_zkapp_command.as_slice();
    let zkapp_input = Rc::new(RefCell::new(None));
    let must_verify = Rc::new(RefCell::new(Boolean::True));

    spec.iter().rev().for_each(|account_update_spec| {
        enum StartOrSkip<T> {
            Start(T),
            Skip,
        }

        let mut finish = |v: StartOrSkip<&StartData>, (global_state, local_state)| {
            let ps = match v {
                StartOrSkip::Skip => CallForest::empty(),
                StartOrSkip::Start(p) => p.account_updates.all_account_updates(),
            };

            let h = w.exists(ps.hash());

            // We decompose this way because of OCaml evaluation order
            let will_succeed = w.exists(match v {
                StartOrSkip::Start(p) => p.will_succeed.to_boolean().constant(),
                StartOrSkip::Skip => Boolean::False.constant(),
            });
            let memo_hash = w.exists(match v {
                StartOrSkip::Skip => Fp::zero(),
                StartOrSkip::Start(p) => p.memo_hash,
            });

            let start_data = StartDataForProof {
                account_updates: WithHash { data: ps, hash: h },
                memo_hash,
                will_succeed,
            };

            {
                let is_start = match account_update_spec.is_start {
                    IsStart::Yes => zkapp_logic::IsStart::Yes(start_data),
                    IsStart::No => zkapp_logic::IsStart::No,
                    IsStart::ComputeInCircuit => zkapp_logic::IsStart::Compute(start_data),
                };

                let single_data = ZkappSingleData {
                    spec: account_update_spec.clone(),
                    zkapp_input: Rc::clone(&zkapp_input),
                    must_verify: Rc::clone(&must_verify),
                };

                zkapp_logic::apply::<ZkappSnark>(
                    ApplyZkappParams {
                        is_start,
                        global_state,
                        local_state,
                        single_data,
                    },
                    w,
                )
            }
        };

        let new_acc = match account_update_spec.is_start {
            IsStart::No => {
                let is_start = zkapp_logic::IsStart::No;

                let single_data = ZkappSingleData {
                    spec: account_update_spec.clone(),
                    zkapp_input: Rc::clone(&zkapp_input),
                    must_verify: Rc::clone(&must_verify),
                };

                zkapp_logic::apply::<ZkappSnark>(
                    ApplyZkappParams {
                        is_start,
                        global_state: &mut global,
                        local_state: &mut local,
                        single_data,
                    },
                    w,
                )
            }
            IsStart::ComputeInCircuit => {
                let v = match start_zkapp_command {
                    [] => StartOrSkip::Skip,
                    [p, ps @ ..] => {
                        let should_pop = local.stack_frame.data.calls.data.is_empty();

                        if should_pop {
                            start_zkapp_command = ps;
                            StartOrSkip::Start(p)
                        } else {
                            StartOrSkip::Skip
                        }
                    }
                };
                finish(v, (&mut global, &mut local))
            }
            IsStart::Yes => {
                assert!(local.stack_frame.data.calls.data.is_empty());

                let v = match start_zkapp_command {
                    [] => unreachable!(),
                    [p, ps @ ..] => {
                        start_zkapp_command = ps;
                        StartOrSkip::Start(p)
                    }
                };
                finish(v, (&mut global, &mut local))
            }
        };

        new_acc.unwrap() // TODO: Remove unwrap
    });

    let on_true = local.stack_frame.hash(w);
    let _local_state_ledger = w.exists_no_check(match local.success.as_boolean() {
        Boolean::True => on_true,
        Boolean::False => statement.target.local_state.stack_frame,
    });

    // Call to `Local_state.Checked.assert_equal` only pushes those
    // 2 values
    w.exists_no_check(local.excess.force_value());
    w.exists_no_check(local.supply_increase.force_value());

    // Call to `Amount.Signed.Checked.assert_equal` only pushes this value
    w.exists_no_check(global.supply_increase.force_value());

    // Call to `Fee_excess.assert_equal_checked` only pushes those 2 values
    w.exists_no_check(global.fee_excess.force_value());
    w.exists_no_check(init_fee_excess.force_value());

    let zkapp_input = Rc::into_inner(zkapp_input).unwrap().into_inner();
    let must_verify = Rc::into_inner(must_verify).unwrap().into_inner();

    (zkapp_input, must_verify)
}

pub struct LedgerProof {
    pub statement: Statement<SokDigest>,
    pub proof: WrapProof,
}

impl From<&LedgerProof> for v2::LedgerProofProdStableV2 {
    fn from(value: &LedgerProof) -> Self {
        let LedgerProof { statement, proof } = value;
        Self(v2::TransactionSnarkStableV2 {
            statement: statement.into(),
            proof: v2::TransactionSnarkProofStableV2(proof.into()),
        })
    }
}

fn first_account_update<'a>(witness: &'a ZkappCommandSegmentWitness) -> Option<&'a AccountUpdate> {
    match witness.local_state_init.stack_frame.calls.0.as_slice() {
        [] => witness
            .start_zkapp_command
            .iter()
            .find_map(|s| s.account_updates.account_updates.first())
            .map(|v| &v.elt.account_update),
        [first, ..] => Some(&first.elt.account_update),
    }
}

fn account_update_proof(p: &AccountUpdate) -> Option<&v2::PicklesProofProofsVerifiedMaxStableV2> {
    match &p.authorization {
        Control::Proof(proof) => Some(proof),
        Control::Signature(_) | Control::NoneGiven => None,
    }
}

fn snapp_proof_data<'a>(
    witness: &'a ZkappCommandSegmentWitness,
) -> Option<(
    &'a v2::PicklesProofProofsVerifiedMaxStableV2,
    crate::VerificationKey,
)> {
    let p = first_account_update(witness)?;
    let pi = account_update_proof(p)?;
    let vk = {
        let account_id = AccountId::create(p.body.public_key.clone(), p.body.token_id.clone());
        let addr = witness.local_state_init.ledger.find_index_exn(account_id);
        let account = witness.local_state_init.ledger.get_exn(&addr);
        account
            .zkapp
            .and_then(|z| z.verification_key)
            .expect("No verification key found in the account")
    };
    Some((pi, vk))
}

fn of_zkapp_command_segment_exn<StepConstants, WrapConstants>(
    statement: Statement<SokDigest>,
    zkapp_witness: &ZkappCommandSegmentWitness,
    spec: &SegmentBasic,
    step_prover: &Prover<Fp>,
    tx_wrap_prover: &Prover<Fq>,
    fps_path: Option<String>,
    fqs_path: Option<String>,
) -> Result<LedgerProof, ProofError>
where
    StepConstants: ProofConstants,
    WrapConstants: ProofConstants + ForWrapData,
{
    use SegmentBasic::*;

    let s = basic_spec(spec);
    let mut w = Witness::new::<StepConstants>();

    if let Some(path) = fps_path.as_ref() {
        w.ocaml_aux = read_witnesses(path);
    };

    let (zkapp_input, must_verify) = zkapp_main(statement.clone(), zkapp_witness, &s, &mut w);

    let StepProof {
        statement: step_statement,
        prev_evals,
        proof,
    } = match spec {
        OptSignedOptSigned | OptSigned => step::<StepConstants, 0>(
            StepParams {
                app_state: Rc::new(statement.clone()),
                rule: InductiveRule::empty(),
                for_step_datas: [],
                indexes: [],
                prev_challenge_polynomial_commitments: vec![],
                hack_feature_flags: OptFlag::No,
                step_prover,
                wrap_prover: tx_wrap_prover,
                only_verify_constraints: false,
            },
            &mut w,
        )?,
        Proved => {
            let (proof, vk) = snapp_proof_data(zkapp_witness).unwrap();
            let proof = proof.into();

            let dlog_plonk_index_cvar = vk.wrap_index.to_cvar(CircuitVar::Var);
            let verifier_index = make_zkapp_verifier_index(&vk);

            let zkapp_data = make_step_zkapp_data(&dlog_plonk_index_cvar);
            let for_step_datas = [&zkapp_data];

            let indexes = [(&verifier_index, &dlog_plonk_index_cvar)];
            let prev_challenge_polynomial_commitments = extract_recursion_challenges(&[&proof]);

            step::<StepConstants, 1>(
                StepParams {
                    app_state: Rc::new(statement.clone()),
                    rule: InductiveRule {
                        previous_proof_statements: [PreviousProofStatement {
                            public_input: Rc::new(zkapp_input.unwrap()),
                            proof: &proof,
                            proof_must_verify: must_verify.var(),
                        }],
                        public_output: (),
                        auxiliary_output: (),
                    },
                    for_step_datas,
                    indexes,
                    prev_challenge_polynomial_commitments,
                    hack_feature_flags: OptFlag::Maybe,
                    step_prover,
                    wrap_prover: tx_wrap_prover,
                    only_verify_constraints: false,
                },
                &mut w,
            )?
        }
    };

    let dlog_plonk_index = super::merge::dlog_plonk_index(tx_wrap_prover);

    let mut w: Witness<Fq> = Witness::new::<WrapConstants>();

    if let Some(path) = fqs_path.as_ref() {
        w.ocaml_aux = read_witnesses(path);
    };

    let proof = wrap::wrap::<WrapConstants>(
        WrapParams {
            app_state: Rc::new(statement.clone()),
            proof: &proof,
            step_statement,
            prev_evals: &prev_evals,
            dlog_plonk_index: &dlog_plonk_index,
            step_prover_index: &step_prover.index,
            wrap_prover: tx_wrap_prover,
        },
        &mut w,
    )?;

    Ok(LedgerProof { statement, proof })
}

impl From<&WrapProof> for v2::PicklesProofProofsVerified2ReprStableV2 {
    fn from(value: &WrapProof) -> Self {
        let WrapProof {
            proof:
                kimchi::proof::ProverProof {
                    commitments:
                        kimchi::proof::ProverCommitments {
                            w_comm,
                            z_comm,
                            t_comm,
                            lookup,
                        },
                    proof:
                        poly_commitment::evaluation_proof::OpeningProof {
                            lr,
                            delta,
                            z1,
                            z2,
                            sg,
                        },
                    evals,
                    ft_eval1,
                    prev_challenges: _,
                },
            statement:
                WrapStatement {
                    proof_state,
                    messages_for_next_step_proof,
                },
            prev_evals,
        } = value;

        assert!(lookup.is_none());

        use mina_p2p_messages::bigint::BigInt;
        use mina_p2p_messages::pseq::PaddedSeq;
        use std::array;

        let to_tuple = |g: &GroupAffine<Fp>| -> (BigInt, BigInt) { (g.x.into(), g.y.into()) };

        v2::PicklesProofProofsVerified2ReprStableV2 {
            statement: v2::PicklesProofProofsVerified2ReprStableV2Statement {
                proof_state: {
                    let WrapProofState {
                        deferred_values:
                            DeferredValues {
                                plonk,
                                combined_inner_product: _,
                                b: _,
                                xi: _,
                                bulletproof_challenges,
                                branch_data,
                            },
                        sponge_digest_before_evaluations,
                        messages_for_next_wrap_proof:
                            MessagesForNextWrapProof {
                                challenge_polynomial_commitment,
                                old_bulletproof_challenges,
                            },
                    } = proof_state;

                    v2::PicklesProofProofsVerified2ReprStableV2StatementProofState {
                        deferred_values: {
                            let to_padded = |v: [u64; 2]| -> PaddedSeq<
                                v2::LimbVectorConstantHex64StableV1,
                                2,
                            > {
                                use v2::LimbVectorConstantHex64StableV1 as V;
                                PaddedSeq([V(v[0].into()), V(v[1].into())])
                            };

                            v2::PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValues {
                                plonk: v2::PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonk {
                                    alpha: v2::PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge {
                                        inner: to_padded(plonk.alpha),
                                    },
                                    beta: to_padded(plonk.beta),
                                    gamma: to_padded(plonk.gamma),
                                    zeta: v2::PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge {
                                        inner: to_padded(plonk.zeta),
                                    },
                                    joint_combiner: None,
                                    feature_flags: v2::PicklesProofProofsVerified2ReprStableV2StatementProofStateDeferredValuesPlonkFeatureFlags {
                                        range_check0: false,
                                        range_check1: false,
                                        foreign_field_add: false,
                                        foreign_field_mul: false,
                                        xor: false,
                                        rot: false,
                                        lookup: false,
                                        runtime_tables: false,
                                    },
                                },
                                bulletproof_challenges: PaddedSeq(array::from_fn(|i| {
                                    v2::PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A {
                                        prechallenge: v2::PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge {
                                            inner: {
                                                let BigInteger256(bigint) = bulletproof_challenges[i].into();
                                                PaddedSeq([v2::LimbVectorConstantHex64StableV1(bigint[0].into()), v2::LimbVectorConstantHex64StableV1(bigint[1].into())])
                                            },
                                        },
                                    }
                                })),
                                branch_data: branch_data.clone(),
                            }
                        },
                        sponge_digest_before_evaluations:
                            v2::CompositionTypesDigestConstantStableV1({
                                let BigInteger256(bigint) =
                                    (*sponge_digest_before_evaluations).into();
                                PaddedSeq(
                                    bigint
                                        .each_ref()
                                        .map(|v| v2::LimbVectorConstantHex64StableV1(v.into())),
                                )
                            }),
                        messages_for_next_wrap_proof:
                            v2::PicklesProofProofsVerified2ReprStableV2MessagesForNextWrapProof {
                                challenge_polynomial_commitment: {
                                    let GroupAffine::<Fq> { x, y, .. } =
                                        challenge_polynomial_commitment.to_affine();
                                    (x.into(), y.into())
                                },
                                old_bulletproof_challenges: PaddedSeq(array::from_fn(|i| {
                                    v2::PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2(
                                    PaddedSeq(array::from_fn(|j| v2::PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A {
                                        prechallenge: v2::PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge {
                                            inner: {
                                                let BigInteger256(bigint) = old_bulletproof_challenges[i][j].into();
                                                PaddedSeq([v2::LimbVectorConstantHex64StableV1(bigint[0].into()), v2::LimbVectorConstantHex64StableV1(bigint[1].into())])
                                            },
                                        },
                                    }))
                                )
                                })),
                            },
                    }
                },
                messages_for_next_step_proof: {
                    let ReducedMessagesForNextStepProof {
                        app_state: _,
                        challenge_polynomial_commitments,
                        old_bulletproof_challenges,
                    } = messages_for_next_step_proof;

                    v2::PicklesProofProofsVerified2ReprStableV2MessagesForNextStepProof {
                        app_state: (),
                        challenge_polynomial_commitments: challenge_polynomial_commitments.iter().map(|curve| {
                            let GroupAffine::<Fp> { x, y, .. } = curve.to_affine();
                            (x.into(), y.into())
                        }).collect(),
                        old_bulletproof_challenges: old_bulletproof_challenges.iter().map(|v| {
                            PaddedSeq(array::from_fn(|i| v2::PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2A {
                                prechallenge: v2::PicklesReducedMessagesForNextProofOverSameFieldWrapChallengesVectorStableV2AChallenge {
                                    inner: {
                                        let BigInteger256(bigint) = v[i].into();
                                        PaddedSeq([v2::LimbVectorConstantHex64StableV1(bigint[0].into()), v2::LimbVectorConstantHex64StableV1(bigint[1].into())])
                                    },
                                },
                            }))
                        }).collect(),
                    }
                },
            },
            prev_evals: {
                let AllEvals {
                    ft_eval1,
                    evals:
                        EvalsWithPublicInput {
                            evals,
                            public_input,
                        },
                } = prev_evals;

                v2::PicklesProofProofsVerified2ReprStableV2PrevEvals {
                    evals: v2::PicklesProofProofsVerified2ReprStableV2PrevEvalsEvals {
                        public_input: (public_input.0.into(), public_input.1.into()),
                        evals: prev_evals_to_p2p(evals),
                    },
                    ft_eval1: ft_eval1.into(),
                }
            },
            proof: v2::PicklesWrapWireProofStableV1 {
                commitments: v2::PicklesWrapWireProofCommitmentsStableV1 {
                    w_comm: PaddedSeq(w_comm.each_ref().map(|w| to_tuple(&w.unshifted[0]))),
                    z_comm: to_tuple(&z_comm.unshifted[0]),
                    t_comm: PaddedSeq(array::from_fn(|i| to_tuple(&t_comm.unshifted[i]))),
                },
                evaluations: {
                    let kimchi::proof::ProofEvaluations {
                        w,
                        z,
                        s,
                        coefficients,
                        generic_selector,
                        poseidon_selector,
                        complete_add_selector,
                        mul_selector,
                        emul_selector,
                        endomul_scalar_selector,
                        ..
                    } = evals;

                    let to_tuple = |point: &PointEvaluations<Vec<Fq>>| -> (BigInt, BigInt) {
                        (point.zeta[0].into(), point.zeta_omega[0].into())
                    };

                    v2::PicklesWrapWireProofEvaluationsStableV1 {
                        w: PaddedSeq(w.each_ref().map(to_tuple)),
                        coefficients: PaddedSeq(coefficients.each_ref().map(to_tuple)),
                        z: to_tuple(z),
                        s: PaddedSeq(s.each_ref().map(to_tuple)),
                        generic_selector: to_tuple(generic_selector),
                        poseidon_selector: to_tuple(poseidon_selector),
                        complete_add_selector: to_tuple(complete_add_selector),
                        mul_selector: to_tuple(mul_selector),
                        emul_selector: to_tuple(emul_selector),
                        endomul_scalar_selector: to_tuple(endomul_scalar_selector),
                    }
                },
                ft_eval1: ft_eval1.into(),
                bulletproof: v2::PicklesWrapWireProofStableV1Bulletproof {
                    lr: lr.iter().map(|(a, b)| (to_tuple(a), to_tuple(b))).collect(),
                    z_1: z1.into(),
                    z_2: z2.into(),
                    delta: to_tuple(delta),
                    challenge_polynomial_commitment: to_tuple(sg),
                },
            },
        }
    }
}

fn of_zkapp_command_segment(
    statement: Statement<SokDigest>,
    zkapp_witness: &ZkappCommandSegmentWitness,
    spec: &SegmentBasic,
    step_opt_signed_opt_signed_prover: &Prover<Fp>,
    step_opt_signed_prover: &Prover<Fp>,
    step_proof_prover: &Prover<Fp>,
    tx_wrap_prover: &Prover<Fq>,
    opt_signed_path: Option<&str>,
    proved_path: Option<&str>,
) -> Result<LedgerProof, ProofError> {
    let (step_prover, step_path, wrap_path) = match spec {
        SegmentBasic::OptSignedOptSigned => (step_opt_signed_opt_signed_prover, None, None),
        SegmentBasic::OptSigned => {
            let fps_path = opt_signed_path.map(|p| format!("{}_fps.txt", p));
            let fqs_path = opt_signed_path.map(|p| format!("{}_fqs.txt", p));
            (step_opt_signed_prover, fps_path, fqs_path)
        }
        SegmentBasic::Proved => {
            let fps_path = proved_path.map(|p| format!("{}_fps.txt", p));
            let fqs_path = proved_path.map(|p| format!("{}_fqs.txt", p));
            (step_proof_prover, fps_path, fqs_path)
        }
    };

    let of_zkapp_command_segment_exn = match spec {
        SegmentBasic::OptSignedOptSigned => {
            of_zkapp_command_segment_exn::<StepZkappOptSignedOptSignedProof, WrapZkappProof>
        }
        SegmentBasic::OptSigned => {
            of_zkapp_command_segment_exn::<StepZkappOptSignedProof, WrapZkappOptSignedProof>
        }
        SegmentBasic::Proved => {
            of_zkapp_command_segment_exn::<StepZkappProvedProof, WrapZkappProvedProof>
        }
    };

    of_zkapp_command_segment_exn(
        statement,
        zkapp_witness,
        spec,
        step_prover,
        tx_wrap_prover,
        step_path,
        wrap_path,
    )
}

pub fn generate_zkapp_proof(params: ZkappParams) -> Result<LedgerProof, ProofError> {
    let ZkappParams {
        statement,
        tx_witness,
        message,
        step_opt_signed_opt_signed_prover,
        step_opt_signed_prover,
        step_proof_prover,
        tx_wrap_prover,
        merge_step_prover,
        opt_signed_path,
        proved_path,
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

    let witnesses_specs_stmts = zkapp_command_witnesses_exn(ZkappCommandWitnessesParams {
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

    let of_zkapp_command_segment = |statement: Statement<SokDigest>,
                                    zkapp_witness: &ZkappCommandSegmentWitness<'_>,
                                    spec: &SegmentBasic| {
        of_zkapp_command_segment(
            statement,
            zkapp_witness,
            spec,
            step_opt_signed_opt_signed_prover,
            step_opt_signed_prover,
            step_proof_prover,
            tx_wrap_prover,
            opt_signed_path,
            proved_path,
        )
    };

    let sok_digest = message.digest();
    let mut witnesses_specs_stmts = witnesses_specs_stmts.into_iter().rev();
    let (zkapp_witness, spec, statement) = witnesses_specs_stmts.next().unwrap(); // last one

    let first_proof = of_zkapp_command_segment(
        statement.with_digest(sok_digest.clone()),
        &zkapp_witness,
        &spec,
    );

    witnesses_specs_stmts.fold(
        first_proof,
        |prev_proof, (zkapp_witness, spec, statement)| {
            let prev_proof = prev_proof?;
            let curr_proof = of_zkapp_command_segment(
                statement.with_digest(sok_digest.clone()),
                &zkapp_witness,
                &spec,
            )?;

            merge_zkapp_proofs(
                prev_proof,
                curr_proof,
                message,
                merge_step_prover,
                tx_wrap_prover,
            )
        },
    )
}

fn merge_zkapp_proofs(
    prev: LedgerProof,
    curr: LedgerProof,
    message: &SokMessage,
    merge_step_prover: &Prover<Fp>,
    tx_wrap_prover: &Prover<Fq>,
) -> Result<LedgerProof, ProofError> {
    let merged_statement = prev
        .statement
        .clone()
        .without_digest()
        .merge(&curr.statement.clone().without_digest())
        .unwrap();

    let prev: v2::LedgerProofProdStableV2 = (&prev).into();
    let curr: v2::LedgerProofProdStableV2 = (&curr).into();

    let mut w: Witness<Fp> = Witness::new::<StepMergeProof>();

    let sok_digest = message.digest();
    let statement_with_sok = merged_statement.clone().with_digest(sok_digest);

    let wrap_proof = generate_merge_proof(
        MergeParams {
            statement: merged_statement,
            proofs: &[prev, curr],
            message,
            step_prover: merge_step_prover,
            wrap_prover: tx_wrap_prover,
            only_verify_constraints: false,
            expected_step_proof: None,
            ocaml_wrap_witness: None,
        },
        &mut w,
    )?;

    Ok(LedgerProof {
        statement: statement_with_sok,
        proof: wrap_proof,
    })
}
