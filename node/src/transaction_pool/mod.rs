use ledger::{
    scan_state::transaction_logic::{verifiable, UserCommand, WithStatus},
    transaction_pool::{
        diff::{self, DiffVerified},
        transaction_hash, ApplyDecision, Config, ValidCommandWithHash,
    },
    Account, AccountId,
};
use mina_p2p_messages::v2;
use openmina_core::consensus::ConsensusConstants;
use redux::callback;
use snark::{user_command_verify::SnarkUserCommandVerifyId, VerifierIndex, VerifierSRS};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};

pub mod transaction_pool_actions;

pub use transaction_pool_actions::{TransactionPoolAction, TransactionPoolEffectfulAction};

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
            pending_id: self.pending_id.clone(),
            best_tip_hash: self.best_tip_hash.clone(),
            file: None,
        }
    }
}

type TransactionPoolActionWithMetaRef<'a> = redux::ActionWithMeta<&'a TransactionPoolAction>;

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

    pub fn get_all_transactions(&self) -> Vec<ValidCommandWithHash> {
        self.pool.get_all_transactions()
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

    fn rebroadcast(&self, _accepted: Vec<UserCommand>, _rejected: Vec<(UserCommand, diff::Error)>) {
        // TODO
    }

    pub fn reducer(mut state: crate::Substate<Self>, action: TransactionPoolActionWithMetaRef<'_>) {
        let (action, _meta) = action.split();
        let substate = state.get_substate_mut().unwrap();

        // Uncoment to save actions to `/tmp/pool.bin`
        // if substate.file.is_none() {
        //     let mut file = std::fs::File::create("/tmp/pool.bin").unwrap();
        //     postcard::to_io(&substate.pool.config, &mut file).unwrap();
        //     postcard::to_io(&substate.pool.pool.config.consensus_constants, &mut file).unwrap();
        //     substate.file = Some(file);
        // }

        if let Some(file) = substate.file.as_mut() {
            postcard::to_io(action, file).unwrap();
        };

        match action {
            TransactionPoolAction::StartVerify { commands } => {
                let commands = commands.iter().map(UserCommand::from).collect::<Vec<_>>();
                let account_ids = commands
                    .iter()
                    .flat_map(|c| c.accounts_referenced())
                    .collect::<BTreeSet<_>>();
                let best_tip_hash = substate.best_tip_hash.clone().unwrap();
                let pending_id = substate.make_action_pending(action);

                let dispatcher = state.into_dispatcher();
                dispatcher.push(TransactionPoolEffectfulAction::FetchAccounts {
                    account_ids,
                    ledger_hash: best_tip_hash.clone(),
                    on_result: callback!(fetch_to_verify((accounts: BTreeMap<AccountId, Account>, id: Option<PendingId>)) -> crate::Action {
                        TransactionPoolAction::StartVerifyWithAccounts { accounts, pending_id: id.unwrap() }
                    }),
                    pending_id: Some(pending_id),
                });
            }
            TransactionPoolAction::StartVerifyWithAccounts {
                accounts,
                pending_id,
            } => {
                let TransactionPoolAction::StartVerify { commands } =
                    substate.pending_actions.remove(pending_id).unwrap()
                else {
                    panic!()
                };

                // TODO: Convert those commands only once
                let commands = commands.iter().map(UserCommand::from).collect::<Vec<_>>();
                let diff = diff::Diff { list: commands };

                let valids = substate.pool.verify(diff, accounts).unwrap(); // TODO: Handle invalids
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
                    is_sender_local: false,
                });
            }
            TransactionPoolAction::BestTipChanged { best_tip_hash } => {
                let account_ids = substate.pool.get_accounts_to_revalidate_on_new_best_tip();
                substate.best_tip_hash = Some(best_tip_hash.clone());

                let dispatcher = state.into_dispatcher();
                dispatcher.push(TransactionPoolEffectfulAction::FetchAccounts {
                    account_ids,
                    ledger_hash: best_tip_hash.clone(),
                    on_result: callback!(fetch_for_best_tip((accounts: BTreeMap<AccountId, Account>, id: Option<PendingId>)) -> crate::Action {
                        TransactionPoolAction::BestTipChangedWithAccounts { accounts }
                    }),
                    pending_id: None,
                });
            }
            TransactionPoolAction::BestTipChangedWithAccounts { accounts } => {
                substate.pool.on_new_best_tip(accounts);
            }
            TransactionPoolAction::ApplyVerifiedDiff {
                best_tip_hash,
                diff,
                is_sender_local: _,
            } => {
                let account_ids = substate.pool.get_accounts_to_apply_diff(&diff);
                let pending_id = substate.make_action_pending(action);

                let dispatcher = state.into_dispatcher();
                dispatcher.push(TransactionPoolEffectfulAction::FetchAccounts {
                    account_ids,
                    ledger_hash: best_tip_hash.clone(),
                    on_result: callback!(fetch_for_apply((accounts: BTreeMap<AccountId, Account>, id: Option<PendingId>)) -> crate::Action {
                        TransactionPoolAction::ApplyVerifiedDiffWithAccounts {
                            accounts,
                            pending_id: id.unwrap(),
                        }
                    }),
                    pending_id: Some(pending_id),
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
                } = substate.pending_actions.remove(pending_id).unwrap()
                else {
                    panic!()
                };

                match substate
                    .pool
                    .unsafe_apply(&diff, &accounts, is_sender_local)
                {
                    Ok((ApplyDecision::Accept, accepted, rejected)) => {
                        substate.rebroadcast(accepted, rejected)
                    }
                    Ok((ApplyDecision::Reject, accepted, rejected)) => {
                        substate.rebroadcast(accepted, rejected)
                    }
                    Err(e) => eprintln!("unsafe_apply error: {:?}", e),
                }
            }
            TransactionPoolAction::ApplyTransitionFrontierDiff {
                best_tip_hash,
                diff,
            } => {
                let account_ids = substate.pool.get_accounts_to_handle_transition_diff(&diff);
                let pending_id = substate.make_action_pending(action);

                let dispatcher = state.into_dispatcher();
                dispatcher.push(TransactionPoolEffectfulAction::FetchAccounts {
                    account_ids,
                    ledger_hash: best_tip_hash.clone(),
                    on_result: callback!(fetch_for_diff((accounts: BTreeMap<AccountId, Account>, id: Option<PendingId>)) -> crate::Action {
                        TransactionPoolAction::ApplyTransitionFrontierDiffWithAccounts {
                            accounts,
                            pending_id: id.unwrap(),
                        }
                    }),
                    pending_id: Some(pending_id),
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

                substate
                    .pool
                    .handle_transition_frontier_diff(&diff, &accounts);
            }
            TransactionPoolAction::Rebroadcast => {}
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
    use crate::ActionKindGet;

    use super::*;

    #[allow(unused)]
    #[test]
    fn test_replay_pool() {
        let vec = std::fs::read("/tmp/pool.bin").unwrap();
        let slice = vec.as_slice();

        let (config, rest) = postcard::take_from_bytes::<Config>(slice).unwrap();
        let (constants, rest) = postcard::take_from_bytes::<ConsensusConstants>(rest).unwrap();
        let mut slice = rest;

        let mut substate = TransactionPoolState::new(config, &constants);

        while let Ok((action, rest)) = postcard::take_from_bytes::<TransactionPoolAction>(slice) {
            slice = rest;

            dbg!(action.kind());

            let action = &action;

            match action {
                TransactionPoolAction::StartVerify { commands } => {
                    dbg!(commands.len());

                    let commands = commands.iter().map(UserCommand::from).collect::<Vec<_>>();
                    let account_ids = commands
                        .iter()
                        .flat_map(|c| c.accounts_referenced())
                        .collect::<BTreeSet<_>>();
                    let best_tip_hash = substate.best_tip_hash.clone().unwrap();
                    let pending_id = substate.make_action_pending(action);
                }
                TransactionPoolAction::StartVerifyWithAccounts {
                    accounts,
                    pending_id,
                } => {
                    let TransactionPoolAction::StartVerify { commands } =
                        substate.pending_actions.remove(pending_id).unwrap()
                    else {
                        panic!()
                    };

                    dbg!(accounts.len());

                    // TODO: Convert those commands only once
                    let commands = commands.iter().map(UserCommand::from).collect::<Vec<_>>();
                    let diff = diff::Diff { list: commands };

                    let valids = substate.pool.verify(diff, accounts).unwrap(); // TODO: Handle invalids
                    let valids = valids
                        .into_iter()
                        .map(transaction_hash::hash_command)
                        .collect::<Vec<_>>();
                    let best_tip_hash = substate.best_tip_hash.clone().unwrap();
                    let diff = DiffVerified { list: valids };
                }
                TransactionPoolAction::BestTipChanged { best_tip_hash } => {
                    let account_ids = substate.pool.get_accounts_to_revalidate_on_new_best_tip();
                    substate.best_tip_hash = Some(best_tip_hash.clone());

                    dbg!(best_tip_hash);
                }
                TransactionPoolAction::BestTipChangedWithAccounts { accounts } => {
                    dbg!(accounts.len());
                    substate.pool.on_new_best_tip(accounts);
                }
                TransactionPoolAction::ApplyVerifiedDiff {
                    best_tip_hash,
                    diff,
                    is_sender_local: _,
                } => {
                    dbg!(best_tip_hash);
                    dbg!(diff.list.len());

                    let account_ids = substate.pool.get_accounts_to_apply_diff(&diff);
                    let pending_id = substate.make_action_pending(action);
                }
                TransactionPoolAction::ApplyVerifiedDiffWithAccounts {
                    accounts,
                    pending_id,
                } => {
                    let TransactionPoolAction::ApplyVerifiedDiff {
                        best_tip_hash: _,
                        diff,
                        is_sender_local,
                    } = substate.pending_actions.remove(pending_id).unwrap()
                    else {
                        panic!()
                    };

                    dbg!(accounts.len());

                    match substate
                        .pool
                        .unsafe_apply(&diff, &accounts, is_sender_local)
                    {
                        Ok((ApplyDecision::Accept, accepted, rejected)) => {
                            substate.rebroadcast(accepted, rejected)
                        }
                        Ok((ApplyDecision::Reject, accepted, rejected)) => {
                            substate.rebroadcast(accepted, rejected)
                        }
                        Err(e) => eprintln!("unsafe_apply error: {:?}", e),
                    }
                }
                TransactionPoolAction::ApplyTransitionFrontierDiff {
                    best_tip_hash,
                    diff,
                } => {
                    dbg!(best_tip_hash);
                    dbg!(diff.new_commands.len());
                    dbg!(diff.removed_commands.len());
                    dbg!(diff.reorg_best_tip);

                    let account_ids = substate.pool.get_accounts_to_handle_transition_diff(&diff);
                    let pending_id = substate.make_action_pending(action);
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

                    dbg!(accounts.len());

                    substate
                        .pool
                        .handle_transition_frontier_diff(&diff, &accounts);
                }
                TransactionPoolAction::Rebroadcast => {}
            }
        }
    }
}
