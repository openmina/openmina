use std::collections::{BTreeMap, BTreeSet};

use ledger::{scan_state::transaction_logic::UserCommand, transaction_pool::{diff, ApplyDecision}, Account, AccountId, BaseLedger, Mask};
use mina_p2p_messages::v2::LedgerHash;

use crate::{Service, Store};

pub mod transaction_pool_actions;

pub use transaction_pool_actions::TransactionPoolAction;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TransactionPoolState {
    pool: ledger::transaction_pool::TransactionPool,
}

type TransactionPoolActionWithMeta = redux::ActionWithMeta<TransactionPoolAction>;
type TransactionPoolActionWithMetaRef<'a> = redux::ActionWithMeta<&'a TransactionPoolAction>;

impl TransactionPoolState {
    pub fn new() -> Self {
        Self {
            pool: ledger::transaction_pool::TransactionPool::new(),
        }
    }

    fn rebroadcast(&self, accepted: Vec<UserCommand>, rejected: Vec<(UserCommand, diff::Error)>) {
        // TODO
    }

    pub fn reducer(&mut self, action: TransactionPoolActionWithMetaRef<'_>) {
        use TransactionPoolAction::*;

        let (action, meta) = action.split();
        match action {
            BestTipChanged { best_tip_hash: _ } => {}
            BestTipChangedWithAccounts { accounts } => {
                self.pool.on_new_best_tip(accounts);
            }
            ApplyVerifiedDiff {
                best_tip_hash: _,
                diff: _,
                is_sender_local: _,
            } => {}
            ApplyVerifiedDiffWithAccounts {
                diff,
                is_sender_local,
                accounts,
            } => match self.pool.unsafe_apply(diff, &accounts, *is_sender_local) {
                Ok((ApplyDecision::Accept, accepted, rejected)) => self.rebroadcast(accepted, rejected),
                Ok((ApplyDecision::Reject, accepted, rejected)) => todo!(),
                Err(_e) => eprintln!("unsafe_apply: {:?}", e),
            },
            ApplyTransitionFrontierDiff {
                best_tip_hash: _,
                diff: _,
            } => {}
            ApplyTransitionFrontierDiffWithAccounts { diff, accounts } => {
                self.pool.handle_transition_frontier_diff(diff, &accounts);
            }
            Rebroadcast => {},
        }
    }
}

fn load_accounts_from_ledger<S: Service>(
    store: &mut Store<S>,
    best_tip_hash: &LedgerHash,
    account_ids: BTreeSet<AccountId>,
) -> BTreeMap<AccountId, Account> {
    let best_tip_mask = store.service.get_mask(&best_tip_hash).unwrap(); // TODO Handle error

    account_ids
        .into_iter()
        .filter_map(|account_id| {
            best_tip_mask
                .location_of_account(&account_id)
                .and_then(|addr| {
                    best_tip_mask
                        .get(addr)
                        .map(|account| (account_id, *account))
                })
        })
        .collect::<BTreeMap<_, _>>()
}

pub fn transaction_pool_effects<S: Service>(
    store: &mut Store<S>,
    action: TransactionPoolActionWithMeta,
) {
    let (action, _meta) = action.split();

    match action {
        TransactionPoolAction::BestTipChanged { best_tip_hash } => {
            let state = &store.state().transaction_pool;
            let account_ids = state.pool.get_accounts_to_revalidate_on_new_best_tip();

            let accounts = load_accounts_from_ledger(store, &best_tip_hash, account_ids);

            store.dispatch(TransactionPoolAction::BestTipChangedWithAccounts { accounts });
        }
        TransactionPoolAction::BestTipChangedWithAccounts { accounts: _ } => {}
        TransactionPoolAction::ApplyVerifiedDiff {
            best_tip_hash,
            diff,
            is_sender_local,
        } => {
            let state = &store.state().transaction_pool;
            let account_ids = state.pool.get_accounts_to_apply_diff(&diff);

            let accounts = load_accounts_from_ledger(store, &best_tip_hash, account_ids);

            store.dispatch(TransactionPoolAction::ApplyVerifiedDiffWithAccounts {
                diff,
                is_sender_local,
                accounts,
            });
        }
        TransactionPoolAction::ApplyVerifiedDiffWithAccounts {
            diff: _,
            is_sender_local: _,
            accounts: _,
        } => {}
        TransactionPoolAction::ApplyTransitionFrontierDiff {
            best_tip_hash,
            diff,
        } => {
            let state = &store.state().transaction_pool;
            let account_ids = state.pool.get_accounts_to_handle_transition_diff(&diff);

            let accounts = load_accounts_from_ledger(store, &best_tip_hash, account_ids);

            store.dispatch(
                TransactionPoolAction::ApplyTransitionFrontierDiffWithAccounts { diff, accounts },
            );
        }
        TransactionPoolAction::ApplyTransitionFrontierDiffWithAccounts {
            diff: _,
            accounts: _,
        } => {}
        TransactionPoolAction::Rebroadcast => todo!(),
    }
}

pub trait TransactionPoolLedgerService: redux::Service {
    fn get_mask(&self, ledger_hash: &LedgerHash) -> Result<Mask, String>;
}
