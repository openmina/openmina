use std::collections::{BTreeMap, BTreeSet};

use ledger::{
    transaction_pool::diff::{BestTipDiff, DiffVerified},
    Account, AccountId, BaseLedger as _,
};
use mina_p2p_messages::v2::LedgerHash;
use redux::Callback;
use serde::{Deserialize, Serialize};

use crate::ledger::LedgerService;

use super::PendingId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransactionPoolAction {
    BestTipChanged {
        best_tip_hash: LedgerHash,
    },
    BestTipChangedWithAccounts {
        accounts: BTreeMap<AccountId, Account>,
    },
    ApplyVerifiedDiff {
        best_tip_hash: LedgerHash,
        diff: DiffVerified,
        /// Diff was crearted locally, or from remote peer ?
        is_sender_local: bool,
    },
    ApplyVerifiedDiffWithAccounts {
        accounts: BTreeMap<AccountId, Account>,
        pending_id: PendingId,
    },
    ApplyTransitionFrontierDiff {
        best_tip_hash: LedgerHash,
        diff: BestTipDiff,
    },
    ApplyTransitionFrontierDiffWithAccounts {
        accounts: BTreeMap<AccountId, Account>,
        pending_id: PendingId,
    },
    /// Rebroadcast locally generated pool items every 10 minutes. Do so for 50
    /// minutes - at most 5 rebroadcasts - before giving up.
    Rebroadcast,
}

impl redux::EnablingCondition<crate::State> for TransactionPoolAction {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransactionPoolEffectfulAction {
    FetchAccounts {
        account_ids: BTreeSet<AccountId>,
        ledger_hash: LedgerHash,
        on_result: Callback<(BTreeMap<AccountId, Account>, Option<PendingId>)>,
        pending_id: Option<PendingId>,
    },
}

impl redux::EnablingCondition<crate::State> for TransactionPoolEffectfulAction {}

impl TransactionPoolEffectfulAction {
    pub fn effects<Store, S>(self, store: &mut Store)
    where
        Store: snark::SnarkStore<S>,
        Store::Service: LedgerService,
    {
        match self {
            TransactionPoolEffectfulAction::FetchAccounts {
                account_ids,
                ledger_hash,
                on_result,
                pending_id,
            } => {
                let (best_tip_mask, _) = store
                    .service()
                    .ledger_manager()
                    .get_mask(&ledger_hash)
                    .unwrap(); // TODO Handle error

                let accounts = account_ids
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
                    .collect::<BTreeMap<_, _>>();

                store.dispatch_callback(on_result, (accounts, pending_id));
            }
        }
    }
}