use ledger::{
    scan_state::transaction_logic::{valid, GenericCommand, UserCommand},
    transaction_pool::{
        diff::{self, DiffVerified},
        transaction_hash, ApplyDecision, TransactionPoolErrors,
    },
    Account, AccountId,
};
use openmina_core::{
    bug_condition,
    constants::constraint_constants,
    requests::RpcId,
    transaction::{Transaction, TransactionWithHash},
};
use p2p::channels::transaction::P2pChannelsTransactionAction;
use redux::callback;
use snark::user_command_verify::{SnarkUserCommandVerifyAction, SnarkUserCommandVerifyId};
use std::collections::{BTreeMap, BTreeSet};

use crate::{BlockProducerAction, RpcAction};

use super::{
    PendingId, TransactionPoolAction, TransactionPoolActionWithMetaRef,
    TransactionPoolEffectfulAction, TransactionPoolState, TransactionState,
};

impl TransactionPoolState {
    pub fn reducer(mut state: crate::Substate<Self>, action: TransactionPoolActionWithMetaRef<'_>) {
        // Uncoment following line to save actions to `/tmp/pool.bin`
        // Self::save_actions(&mut state);

        let substate = state.get_substate_mut().unwrap();
        if let Some(file) = substate.file.as_mut() {
            postcard::to_io(&action, file).unwrap();
        };

        Self::handle_action(state, action)
    }

    pub(super) fn handle_action(
        mut state: crate::Substate<Self>,
        action: TransactionPoolActionWithMetaRef<'_>,
    ) {
        let (action, meta) = action.split();
        let Some((global_slot, global_slot_from_genesis)) =
            // TODO: remove usage of `unsafe_get_state`
            Self::global_slots(state.unsafe_get_state())
        else {
            return;
        };
        let substate = state.get_substate_mut().unwrap();

        match action {
            TransactionPoolAction::Candidate(a) => {
                super::candidate::TransactionPoolCandidatesState::reducer(
                    openmina_core::Substate::from_compatible_substate(state),
                    meta.with_action(a),
                );
            }
            TransactionPoolAction::StartVerify { commands, from_rpc } => {
                let Ok(commands) = commands
                    .iter()
                    .map(TransactionWithHash::body)
                    .map(UserCommand::try_from)
                    .collect::<Result<Vec<_>, _>>()
                else {
                    // ignore all commands if one is invalid
                    return;
                };

                let account_ids = commands
                    .iter()
                    .flat_map(UserCommand::accounts_referenced)
                    .collect::<BTreeSet<_>>();
                let best_tip_hash = substate.best_tip_hash.clone().unwrap();
                let pending_id = substate.make_action_pending(action);

                let dispatcher = state.into_dispatcher();
                dispatcher.push(TransactionPoolEffectfulAction::FetchAccounts {
                    account_ids,
                    ledger_hash: best_tip_hash.clone(),
                    on_result: callback!(fetch_to_verify((accounts: BTreeMap<AccountId, Account>, id: Option<PendingId>, from_rpc: Option<RpcId>)) -> crate::Action {
                        TransactionPoolAction::StartVerifyWithAccounts { accounts, pending_id: id.unwrap(), from_rpc }
                    }),
                    pending_id: Some(pending_id),
                    from_rpc: *from_rpc,
                });
            }
            TransactionPoolAction::StartVerifyWithAccounts {
                accounts,
                pending_id,
                from_rpc,
            } => {
                let TransactionPoolAction::StartVerify { commands, .. } =
                    substate.pending_actions.remove(pending_id).unwrap()
                else {
                    panic!()
                };

                // TODO: Convert those commands only once
                let Ok(commands) = commands
                    .iter()
                    .map(TransactionWithHash::body)
                    .map(UserCommand::try_from)
                    .collect::<Result<Vec<_>, _>>()
                else {
                    return;
                };
                let diff = diff::Diff { list: commands };

                match substate
                    .pool
                    .prevalidate(diff)
                    .and_then(|diff| substate.pool.convert_diff_to_verifiable(diff, accounts))
                {
                    Ok(verifiable) => {
                        let (dispatcher, global_state) = state.into_dispatcher_and_state();
                        let req_id = global_state.snark.user_command_verify.next_req_id();

                        dispatcher.push(SnarkUserCommandVerifyAction::Init {
                            req_id,
                            commands: verifiable,
                            from_rpc: *from_rpc,
                            on_success: callback!(
                                on_snark_user_command_verify_success(
                                    (req_id: SnarkUserCommandVerifyId, valids: Vec<valid::UserCommand>, from_rpc: Option<RpcId>)
                                ) -> crate::Action {
                                    TransactionPoolAction::VerifySuccess {
                                        valids,
                                        from_rpc,
                                    }
                                }
                            ),
                            on_error: callback!(
                                on_snark_user_command_verify_error(
                                    (req_id: SnarkUserCommandVerifyId, errors: Vec<String>)
                                ) -> crate::Action {
                                    TransactionPoolAction::VerifyError {
                                        errors
                                    }
                                }
                            )
                        });
                    }
                    Err(e) => {
                        let dispatch_errors = |errors: Vec<String>| {
                            let dispatcher = state.into_dispatcher();
                            dispatcher.push(TransactionPoolAction::VerifyError {
                                errors: errors.clone(),
                            });
                            if let Some(rpc_id) = from_rpc {
                                dispatcher.push(RpcAction::TransactionInjectFailure {
                                    rpc_id: *rpc_id,
                                    errors,
                                })
                            }
                        };
                        match e {
                            TransactionPoolErrors::BatchedErrors(errors) => {
                                let errors: Vec<_> =
                                    errors.into_iter().map(|e| e.to_string()).collect();
                                dispatch_errors(errors);
                            }
                            TransactionPoolErrors::LoadingVK(error) => dispatch_errors(vec![error]),
                            TransactionPoolErrors::Unexpected(es) => {
                                panic!("{es}")
                            }
                        }
                    }
                }
            }
            TransactionPoolAction::VerifySuccess { valids, from_rpc } => {
                let valids = valids
                    .iter()
                    .cloned()
                    .map(transaction_hash::hash_command)
                    .collect::<Vec<_>>();
                let best_tip_hash = substate.best_tip_hash.clone().unwrap();
                let diff = DiffVerified { list: valids };

                let dispatcher = state.into_dispatcher();
                dispatcher.push(TransactionPoolAction::ApplyVerifiedDiff {
                    best_tip_hash,
                    diff,
                    is_sender_local: from_rpc.is_some(),
                    from_rpc: *from_rpc,
                });
            }
            TransactionPoolAction::VerifyError { .. } => {
                // just logging the errors
            }
            TransactionPoolAction::BestTipChanged { best_tip_hash } => {
                let account_ids = substate.pool.get_accounts_to_revalidate_on_new_best_tip();
                substate.best_tip_hash = Some(best_tip_hash.clone());

                let dispatcher = state.into_dispatcher();
                dispatcher.push(TransactionPoolEffectfulAction::FetchAccounts {
                    account_ids,
                    ledger_hash: best_tip_hash.clone(),
                    on_result: callback!(fetch_for_best_tip((accounts: BTreeMap<AccountId, Account>, id: Option<PendingId>, from_rpc: Option<RpcId>)) -> crate::Action {
                        TransactionPoolAction::BestTipChangedWithAccounts { accounts }
                    }),
                    pending_id: None,
                    from_rpc: None,
                });
            }
            TransactionPoolAction::BestTipChangedWithAccounts { accounts } => {
                match substate
                    .pool
                    .on_new_best_tip(global_slot_from_genesis, accounts)
                {
                    Err(e) => bug_condition!("transaction pool::on_new_best_tip failed: {:?}", e),
                    Ok(dropped) => {
                        for tx in dropped {
                            substate.dpool.remove(&tx.hash);
                        }
                    }
                }
            }
            TransactionPoolAction::ApplyVerifiedDiff {
                best_tip_hash,
                diff,
                is_sender_local: _,
                from_rpc,
            } => {
                let account_ids = substate.pool.get_accounts_to_apply_diff(diff);
                let pending_id = substate.make_action_pending(action);

                let dispatcher = state.into_dispatcher();
                dispatcher.push(TransactionPoolEffectfulAction::FetchAccounts {
                    account_ids,
                    ledger_hash: best_tip_hash.clone(),
                    on_result: callback!(fetch_for_apply((accounts: BTreeMap<AccountId, Account>, id: Option<PendingId>, from_rpc: Option<RpcId>)) -> crate::Action {
                        TransactionPoolAction::ApplyVerifiedDiffWithAccounts {
                            accounts,
                            pending_id: id.unwrap(),
                        }
                    }),
                    pending_id: Some(pending_id),
                    from_rpc: *from_rpc,
                });
            }
            TransactionPoolAction::ApplyVerifiedDiffWithAccounts {
                accounts,
                pending_id,
            } => {
                let TransactionPoolAction::ApplyVerifiedDiff {
                    best_tip_hash: _,
                    diff,
                    is_sender_local,
                    from_rpc,
                } = substate.pending_actions.remove(pending_id).unwrap()
                else {
                    panic!()
                };

                // Note(adonagy): Action for rebroadcast, in his action we can use forget_check
                let (rpc_action, was_accepted, accepted, rejected) = match substate
                    .pool
                    .unsafe_apply(
                        meta.time(),
                        global_slot_from_genesis,
                        global_slot,
                        &diff,
                        accounts,
                        is_sender_local,
                    ) {
                    Ok((ApplyDecision::Accept, accepted, rejected, dropped)) => {
                        for hash in dropped {
                            substate.dpool.remove(&hash);
                        }
                        for tx in &accepted {
                            substate.dpool.insert(TransactionState {
                                time: meta.time(),
                                hash: tx.hash.clone(),
                            });
                        }
                        let rpc_action =
                            from_rpc.map(|rpc_id| RpcAction::TransactionInjectSuccess {
                                rpc_id,
                                response: accepted.clone(),
                            });
                        (rpc_action, true, accepted, rejected)
                    }
                    Ok((ApplyDecision::Reject, accepted, rejected, _)) => {
                        let rpc_action =
                            from_rpc.map(|rpc_id| RpcAction::TransactionInjectRejected {
                                rpc_id,
                                response: rejected.clone(),
                            });
                        (rpc_action, false, accepted, rejected)
                    }
                    Err(e) => {
                        crate::core::warn!(meta.time(); kind = "TransactionPoolUnsafeApplyError", summary = e);
                        return;
                    }
                };

                let dispatcher = state.into_dispatcher();
                if let Some(rpc_action) = rpc_action {
                    dispatcher.push(rpc_action);
                }
                // TODO: libp2p logic already broadcasts everything right now and doesn't
                // wait for validation, thad needs to be fixed. See #952
                if was_accepted {
                    dispatcher.push(TransactionPoolAction::Rebroadcast {
                        accepted,
                        rejected,
                        is_local: is_sender_local,
                    });
                }
            }
            TransactionPoolAction::ApplyTransitionFrontierDiff {
                best_tip_hash,
                diff,
            } => {
                assert_eq!(substate.best_tip_hash.as_ref().unwrap(), best_tip_hash);

                let (account_ids, uncommitted) =
                    substate.pool.get_accounts_to_handle_transition_diff(diff);
                let pending_id = substate.make_action_pending(action);

                let dispatcher = state.into_dispatcher();
                dispatcher.push(TransactionPoolEffectfulAction::FetchAccounts {
                    account_ids: account_ids.union(&uncommitted).cloned().collect(),
                    ledger_hash: best_tip_hash.clone(),
                    on_result: callback!(fetch_for_diff((accounts: BTreeMap<AccountId, Account>, id: Option<PendingId>, from_rpc: Option<RpcId>)) -> crate::Action {
                        TransactionPoolAction::ApplyTransitionFrontierDiffWithAccounts {
                            accounts,
                            pending_id: id.unwrap(),
                        }
                    }),
                    pending_id: Some(pending_id),
                    from_rpc: None,
                });
            }
            TransactionPoolAction::ApplyTransitionFrontierDiffWithAccounts {
                accounts,
                pending_id,
            } => {
                let TransactionPoolAction::ApplyTransitionFrontierDiff {
                    best_tip_hash: _,
                    diff,
                } = substate.pending_actions.remove(pending_id).unwrap()
                else {
                    panic!()
                };

                let collect = |set: &BTreeSet<AccountId>| {
                    set.iter()
                        .filter_map(|id| {
                            let account = accounts.get(id).cloned()?;
                            Some((id.clone(), account))
                        })
                        .collect::<BTreeMap<_, _>>()
                };

                let (account_ids, uncommitted) =
                    substate.pool.get_accounts_to_handle_transition_diff(&diff);

                let in_cmds = collect(&account_ids);
                let uncommitted = collect(&uncommitted);

                if let Err(e) = substate.pool.handle_transition_frontier_diff(
                    global_slot_from_genesis,
                    global_slot,
                    &diff,
                    &account_ids,
                    &in_cmds,
                    &uncommitted,
                ) {
                    bug_condition!(
                        "transaction pool::handle_transition_frontier_diff failed: {:?}",
                        e
                    );
                }
            }
            TransactionPoolAction::Rebroadcast {
                accepted,
                rejected,
                is_local,
            } => {
                let rejected = rejected.iter().map(|(cmd, _)| cmd.data.forget_check());

                let all_commands = accepted
                    .iter()
                    .map(|cmd| cmd.data.forget_check())
                    .chain(rejected)
                    .collect::<Vec<_>>();

                let dispatcher = state.into_dispatcher();

                for cmd in all_commands {
                    dispatcher.push(P2pChannelsTransactionAction::Libp2pBroadcast {
                        transaction: Box::new((&cmd).into()),
                        nonce: 0,
                        is_local: *is_local,
                    });
                }
            }
            TransactionPoolAction::CollectTransactionsByFee => {
                let transaction_capacity =
                    2u64.pow(constraint_constants().transaction_capacity_log_2 as u32);
                let transactions_by_fee = substate
                    .pool
                    .list_includable_transactions(transaction_capacity as usize)
                    .into_iter()
                    .map(|cmd| cmd.data)
                    .collect::<Vec<_>>();

                let dispatcher = state.into_dispatcher();

                dispatcher.push(BlockProducerAction::WonSlotTransactionsSuccess {
                    transactions_by_fee,
                });
            }
            TransactionPoolAction::P2pSendAll => {
                let (dispatcher, global_state) = state.into_dispatcher_and_state();
                for peer_id in global_state.p2p.ready_peers() {
                    dispatcher.push(TransactionPoolAction::P2pSend { peer_id });
                }
            }
            TransactionPoolAction::P2pSend { peer_id } => {
                let peer_id = *peer_id;
                let (dispatcher, global_state) = state.into_dispatcher_and_state();
                let Some(peer) = global_state.p2p.get_ready_peer(&peer_id) else {
                    return;
                };

                // Send commitments.
                let index_and_limit = peer.channels.transaction.next_send_index_and_limit();
                let (transactions, first_index, last_index) = global_state
                    .transaction_pool
                    .dpool
                    .next_messages_to_send(index_and_limit, |state| {
                        let tx = global_state.transaction_pool.get(&state.hash)?;
                        let tx = tx.clone().forget();
                        // TODO(binier): avoid conversion
                        Some((&Transaction::from(&tx)).into())
                    });

                dispatcher.push(P2pChannelsTransactionAction::ResponseSend {
                    peer_id,
                    transactions,
                    first_index,
                    last_index,
                });
            }
        }
    }
}
