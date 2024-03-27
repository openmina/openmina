use std::collections::BTreeMap;

use ledger::{
    transaction_pool::{
        diff::{BestTipDiff, DiffVerified},
        ApplyDecision,
    },
    Account, BaseLedger, Mask,
};
use mina_p2p_messages::v2::LedgerHash;
use serde::{Deserialize, Serialize};

use crate::{Service, Store};

pub mod transaction_pool_actions;

pub use transaction_pool_actions::TransactionPoolAction;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TransactionPoolState {
    pool: ledger::transaction_pool::TransactionPool,
    // accounts_to_revalidate: Option<BTreeSet<AccountId>>,
}

type TransactionPoolActionWithMeta = redux::ActionWithMeta<TransactionPoolAction>;
type TransactionPoolActionWithMetaRef<'a> = redux::ActionWithMeta<&'a TransactionPoolAction>;

impl TransactionPoolState {
    pub fn new() -> Self {
        Self {
            pool: ledger::transaction_pool::TransactionPool::new(),
        }
    }

    pub fn reducer(&mut self, action: TransactionPoolActionWithMetaRef<'_>) {
        use TransactionPoolAction::*;

        let (action, meta) = action.split();
        match action {
            BestTipChanged { best_tip_hash } => {
                // let account_ids = self.pool.get_accounts_to_revalidate_on_new_best_tip();
                // let best_tip: Mask = Mask::new_unattached(10);
                // self.pool.on_new_best_tip(best_tip.clone());
            }
            BestTipChangedWithAccounts { accounts } => {
                self.pool.on_new_best_tip(accounts);
            }
            ApplyVerifiedDiff {
                diff,
                is_sender_local,
            } => match self.pool.unsafe_apply(diff, *is_sender_local) {
                Ok((ApplyDecision::Accept, accepted, rejected)) => todo!(),
                Ok((ApplyDecision::Reject, accepted, rejected)) => todo!(),
                Err(_) => todo!(),
            },
            ApplyTransitionFrontierDiff {
                best_tip_hash,
                diff,
            } => {
                // self.pool
                //     .handle_transition_frontier_diff(diff, best_tip.clone());
            }
            Rebroadcast => todo!(),
        }
    }
}

pub fn transaction_pool_effects<S: Service>(
    store: &mut Store<S>,
    action: TransactionPoolActionWithMeta,
) {
    let (action, meta) = action.split();

    match action {
        TransactionPoolAction::BestTipChanged { best_tip_hash } => {
            let state = &store.state().transaction_pool;
            let account_ids = state.pool.get_accounts_to_revalidate_on_new_best_tip();

            let best_tip = store.service.get_mask(&best_tip_hash).unwrap(); // TODO Handle error

            let accounts = account_ids
                .into_iter()
                .filter_map(|account_id| {
                    best_tip
                        .location_of_account(&account_id)
                        .and_then(|addr| best_tip.get(addr).map(|account| (account_id, *account)))
                })
                .collect::<BTreeMap<_, _>>();

            store.dispatch(TransactionPoolAction::BestTipChangedWithAccounts { accounts });
        }
        TransactionPoolAction::BestTipChangedWithAccounts { accounts } => todo!(),
        TransactionPoolAction::ApplyVerifiedDiff {
            diff,
            is_sender_local,
        } => todo!(),
        TransactionPoolAction::ApplyTransitionFrontierDiff {
            best_tip_hash,
            diff,
        } => todo!(),
        TransactionPoolAction::Rebroadcast => todo!(),
    }
}

pub trait TransactionPoolLedgerService: redux::Service {
    fn get_mask(&self, ledger_hash: &LedgerHash) -> Result<Mask, String>;
}
