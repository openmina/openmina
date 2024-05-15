use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use ledger::{
    scan_state::transaction_logic::{verifiable, UserCommand, WithStatus},
    transaction_pool::{diff, ApplyDecision},
    Account, AccountId,
};
use redux::callback;
use snark::{user_command_verify::SnarkUserCommandVerifyId, VerifierIndex, VerifierSRS};

pub mod transaction_pool_actions;

pub use transaction_pool_actions::{TransactionPoolAction, TransactionPoolEffectfulAction};

type PendingId = u32;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TransactionPoolState {
    pool: ledger::transaction_pool::TransactionPool,
    pending_actions: BTreeMap<PendingId, TransactionPoolAction>,
    pending_id: PendingId,
}

type TransactionPoolActionWithMetaRef<'a> = redux::ActionWithMeta<&'a TransactionPoolAction>;

impl TransactionPoolState {
    pub fn new() -> Self {
        Self {
            pool: ledger::transaction_pool::TransactionPool::new(),
            pending_actions: Default::default(),
            pending_id: 0,
        }
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
        use TransactionPoolAction::*;

        let (action, _meta) = action.split();
        match action {
            BestTipChanged { best_tip_hash } => {
                let account_ids = state.pool.get_accounts_to_revalidate_on_new_best_tip();

                let dispatcher = state.into_dispatcher();
                dispatcher.push(TransactionPoolEffectfulAction::FetchAccounts {
                    account_ids,
                    ledger_hash: best_tip_hash.clone(),
                    on_result: callback!(|(accounts: BTreeMap<AccountId, Account>, id: Option<PendingId>)|  -> crate::Action {
                        TransactionPoolAction::BestTipChangedWithAccounts { accounts }
                    }),
                    pending_id: None,
                });
            }
            BestTipChangedWithAccounts { accounts } => {
                state.pool.on_new_best_tip(accounts);
            }
            ApplyVerifiedDiff {
                best_tip_hash,
                diff,
                is_sender_local: _,
            } => {
                let account_ids = state.pool.get_accounts_to_apply_diff(&diff);
                let pending_id = state.make_action_pending(action);

                let dispatcher = state.into_dispatcher();
                dispatcher.push(TransactionPoolEffectfulAction::FetchAccounts {
                    account_ids,
                    ledger_hash: best_tip_hash.clone(),
                    on_result: callback!(|(accounts: BTreeMap<AccountId, Account>, id: Option<PendingId>)|  -> crate::Action {
                        TransactionPoolAction::ApplyVerifiedDiffWithAccounts {
                            accounts,
                            pending_id: id.unwrap(),
                        }
                    }),
                    pending_id: Some(pending_id),
                });
            }
            ApplyVerifiedDiffWithAccounts {
                accounts,
                pending_id,
            } => {
                let ApplyVerifiedDiff {
                    best_tip_hash: _,
                    diff,
                    is_sender_local,
                } = state.pending_actions.remove(pending_id).unwrap()
                else {
                    panic!()
                };

                match state.pool.unsafe_apply(&diff, &accounts, is_sender_local) {
                    Ok((ApplyDecision::Accept, accepted, rejected)) => {
                        state.rebroadcast(accepted, rejected)
                    }
                    Ok((ApplyDecision::Reject, accepted, rejected)) => {
                        state.rebroadcast(accepted, rejected)
                    }
                    Err(e) => eprintln!("unsafe_apply error: {:?}", e),
                }
            }
            ApplyTransitionFrontierDiff {
                best_tip_hash,
                diff,
            } => {
                let account_ids = state.pool.get_accounts_to_handle_transition_diff(&diff);
                let pending_id = state.make_action_pending(action);

                let dispatcher = state.into_dispatcher();
                dispatcher.push(TransactionPoolEffectfulAction::FetchAccounts {
                    account_ids,
                    ledger_hash: best_tip_hash.clone(),
                    on_result: callback!(|(accounts: BTreeMap<AccountId, Account>, id: Option<PendingId>)|  -> crate::Action {
                        TransactionPoolAction::ApplyTransitionFrontierDiffWithAccounts {
                            accounts,
                            pending_id: id.unwrap(),
                        }
                    }),
                    pending_id: Some(pending_id),
                });
            }
            ApplyTransitionFrontierDiffWithAccounts {
                accounts,
                pending_id,
            } => {
                let ApplyTransitionFrontierDiff {
                    best_tip_hash: _,
                    diff,
                } = state.pending_actions.remove(pending_id).unwrap()
                else {
                    panic!()
                };

                state.pool.handle_transition_frontier_diff(&diff, &accounts);
            }
            Rebroadcast => {}
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
