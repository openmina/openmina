use std::collections::{BTreeMap, BTreeSet};

use ledger::{
    transaction_pool::{
        diff::{self, BestTipDiff, DiffVerified},
        ValidCommandWithHash,
    },
    Account, AccountId,
};
use mina_p2p_messages::{
    list::List,
    v2::{self, LedgerHash},
};
use openmina_core::{requests::RpcId, ActionEvent};
use redux::Callback;
use serde::{Deserialize, Serialize};

use crate::ledger::LedgerService;

use super::PendingId;

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(level = info)]
pub enum TransactionPoolAction {
    StartVerify {
        commands: List<v2::MinaBaseUserCommandStableV2>,
        from_rpc: Option<RpcId>,
    },
    StartVerifyWithAccounts {
        accounts: BTreeMap<AccountId, Account>,
        pending_id: PendingId,
        from_rpc: Option<RpcId>,
    },
    #[action_event(level = warn, fields(debug(errors)))]
    VerifyError {
        errors: Vec<String>,
    },
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
        from_rpc: Option<RpcId>,
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
    Rebroadcast {
        accepted: Vec<ValidCommandWithHash>,
        rejected: Vec<(ValidCommandWithHash, diff::Error)>,
    },
    CollectTransactionsByFee,
}

impl redux::EnablingCondition<crate::State> for TransactionPoolAction {}

type TransactionPoolEffectfulActionCallback = Callback<(
    BTreeMap<AccountId, Account>,
    Option<PendingId>,
    Option<RpcId>,
)>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransactionPoolEffectfulAction {
    FetchAccounts {
        account_ids: BTreeSet<AccountId>,
        ledger_hash: LedgerHash,
        on_result: TransactionPoolEffectfulActionCallback,
        pending_id: Option<PendingId>,
        from_rpc: Option<RpcId>,
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
                from_rpc,
            } => {
                openmina_core::log::info!(
                    openmina_core::log::system_time();
                    kind = "Info",
                    summary = "fetching accounts for tx pool");
                // FIXME: the ledger ctx `get_accounts` function doesn't ensure that every account we
                // asked for is included in the result.
                // TODO: should be asynchronous. Once asynchronous, watch out for race
                // conditions between tx pool and transition frontier. By the time the
                // accounts have been fetched the best tip may have changed already.
                let accounts = match store
                    .service()
                    .ledger_manager()
                    .get_accounts(&ledger_hash, account_ids.iter().cloned().collect())
                {
                    Ok(accounts) => accounts,
                    Err(err) => {
                        openmina_core::log::error!(
                                openmina_core::log::system_time();
                                kind = "Error",
                                summary = "failed to fetch accounts for tx pool",
                                error = format!("ledger {:?}, error: {:?}", ledger_hash, err));
                        return;
                    }
                };

                let accounts = accounts
                    .into_iter()
                    .map(|account| (account.id(), account))
                    .collect::<BTreeMap<_, _>>();

                store.dispatch_callback(on_result, (accounts, pending_id, from_rpc));
            }
        }
    }
}
