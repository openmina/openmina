use ledger::{
    scan_state::{
        currency::{Amount, Nonce, Slot},
        transaction_logic::{verifiable, UserCommand, WithStatus},
    },
    transaction_pool::{
        diff::{self, DiffVerified},
        transaction_hash, ApplyDecision, Config, TransactionPoolErrors, ValidCommandWithHash,
    },
    Account, AccountId,
};
use mina_p2p_messages::v2;
use openmina_core::{
    consensus::ConsensusConstants, constants::constraint_constants, requests::RpcId,
};
use p2p::channels::transaction::P2pChannelsTransactionAction;
use redux::callback;
use snark::{user_command_verify::SnarkUserCommandVerifyId, VerifierIndex, VerifierSRS};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::{Arc, Mutex},
};

pub mod transaction_pool_actions;

pub use transaction_pool_actions::{TransactionPoolAction, TransactionPoolEffectfulAction};

use crate::{BlockProducerAction, RpcAction};

type PendingId = u32;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TransactionPoolState {
    pool: ledger::transaction_pool::TransactionPool,
    pending_actions: BTreeMap<PendingId, TransactionPoolAction>,
    pending_id: PendingId,
    best_tip_hash: Option<v2::LedgerHash>,
    /// For debug only
    #[serde(skip)]
    file: Option<std::fs::File>,
}

impl Clone for TransactionPoolState {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            pending_actions: self.pending_actions.clone(),
            pending_id: self.pending_id,
            best_tip_hash: self.best_tip_hash.clone(),
            file: None,
        }
    }
}

impl TransactionPoolState {
    pub fn new(config: Config, consensus_constants: &ConsensusConstants) -> Self {
        Self {
            pool: ledger::transaction_pool::TransactionPool::new(config, consensus_constants),
            pending_actions: Default::default(),
            pending_id: 0,
            best_tip_hash: None,
            file: None,
        }
    }

    pub fn size(&self) -> usize {
        self.pool.size()
    }

    pub fn transactions(&mut self, limit: usize) -> Vec<ValidCommandWithHash> {
        self.pool.transactions(limit)
    }

    pub fn list_includable_transactions(&self, limit: usize) -> Vec<ValidCommandWithHash> {
        self.pool.list_includable_transactions(limit)
    }

    pub fn get_all_transactions(&self) -> Vec<ValidCommandWithHash> {
        self.pool.get_all_transactions()
    }

    pub fn get_pending_amount_and_nonce(&self) -> HashMap<AccountId, (Option<Nonce>, Amount)> {
        self.pool.get_pending_amount_and_nonce()
    }

    fn next_pending_id(&mut self) -> PendingId {
        let id = self.pending_id;
        self.pending_id = self.pending_id.wrapping_add(1);
        id
    }

    fn make_action_pending(&mut self, action: &TransactionPoolAction) -> PendingId {
        let id = self.next_pending_id();
        self.pending_actions.insert(id, action.clone());
        id
    }

    #[allow(dead_code)]
    fn save_actions(state: &mut crate::Substate<Self>) {
        let substate = state.get_substate_mut().unwrap();
        if substate.file.is_none() {
            let mut file = std::fs::File::create("/tmp/pool.bin").unwrap();
            postcard::to_io(&state.get_state(), &mut file).unwrap();
            let substate = state.get_substate_mut().unwrap();
            substate.file = Some(file);
        }
    }

    pub fn reducer(mut state: crate::Substate<Self>, action: &TransactionPoolAction) {
        // Uncoment following line to save actions to `/tmp/pool.bin`
        // Self::save_actions(&mut state);

        let substate = state.get_substate_mut().unwrap();
        if let Some(file) = substate.file.as_mut() {
            postcard::to_io(action, file).unwrap();
        };

        Self::handle_action(state, action)
    }

    fn global_slots(state: &crate::State) -> Option<(Slot, Slot)> {
        Some((
            Slot::from_u32(state.cur_global_slot()?),
            Slot::from_u32(state.cur_global_slot_since_genesis()?),
        ))
    }

    fn handle_action(mut state: crate::Substate<Self>, action: &TransactionPoolAction) {
        let Some((global_slot, global_slot_from_genesis)) = Self::global_slots(state.get_state())
        else {
            return;
        };
        let substate = state.get_substate_mut().unwrap();

        match action {
            TransactionPoolAction::StartVerify { commands, from_rpc } => {
                let commands = commands.iter().map(UserCommand::from).collect::<Vec<_>>();
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
                let commands = commands.iter().map(UserCommand::from).collect::<Vec<_>>();
                let diff = diff::Diff { list: commands };

                match substate.pool.verify(diff, accounts) {
                    Ok(valids) => {
                        let valids = valids
                            .into_iter()
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
                substate
                    .pool
                    .on_new_best_tip(global_slot_from_genesis, accounts);
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
                match substate.pool.unsafe_apply(
                    global_slot_from_genesis,
                    global_slot,
                    &diff,
                    accounts,
                    is_sender_local,
                ) {
                    Ok((ApplyDecision::Accept, accepted, rejected)) => {
                        if let Some(rpc_id) = from_rpc {
                            let dispatcher = state.into_dispatcher();

                            dispatcher.push(RpcAction::TransactionInjectSuccess {
                                rpc_id,
                                response: accepted.clone(),
                            });
                            dispatcher
                                .push(TransactionPoolAction::Rebroadcast { accepted, rejected });
                        }
                    }
                    Ok((ApplyDecision::Reject, accepted, rejected)) => {
                        if let Some(rpc_id) = from_rpc {
                            let dispatcher = state.into_dispatcher();

                            dispatcher.push(RpcAction::TransactionInjectRejected {
                                rpc_id,
                                response: rejected.clone(),
                            });
                            dispatcher
                                .push(TransactionPoolAction::Rebroadcast { accepted, rejected });
                        }
                    }
                    Err(e) => eprintln!("unsafe_apply error: {:?}", e),
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

                substate.pool.handle_transition_frontier_diff(
                    global_slot_from_genesis,
                    global_slot,
                    &diff,
                    &account_ids,
                    &in_cmds,
                    &uncommitted,
                );
            }
            TransactionPoolAction::Rebroadcast { accepted, rejected } => {
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
        }
    }
}

pub trait VerifyUserCommandsService: redux::Service {
    fn verify_init(
        &mut self,
        req_id: SnarkUserCommandVerifyId,
        commands: Vec<WithStatus<verifiable::UserCommand>>,
        verifier_index: Arc<VerifierIndex>,
        verifier_srs: Arc<Mutex<VerifierSRS>>,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::State;
    use redux::Dispatcher;

    #[allow(unused)]
    #[test]
    fn test_replay_pool() {
        let vec = std::fs::read("/tmp/pool.bin").unwrap();
        let slice = vec.as_slice();

        let (mut state, rest) = postcard::take_from_bytes::<State>(slice).unwrap();
        let mut slice = rest;

        while let Ok((action, rest)) = postcard::take_from_bytes::<TransactionPoolAction>(slice) {
            slice = rest;

            let mut dispatcher = Dispatcher::new();
            let state = crate::Substate::<TransactionPoolState>::new(&mut state, &mut dispatcher);

            TransactionPoolState::handle_action(state, &action);
        }
    }
}
