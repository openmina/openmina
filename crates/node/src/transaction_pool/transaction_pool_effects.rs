use std::collections::BTreeMap;

use crate::{ledger::LedgerService, snark::SnarkStore};

use super::TransactionPoolEffectfulAction;

impl TransactionPoolEffectfulAction {
    pub fn effects<Store, S>(self, store: &mut Store)
    where
        Store: SnarkStore<S>,
        Store::Service: LedgerService,
    {
        match self {
            TransactionPoolEffectfulAction::FetchAccounts {
                account_ids,
                ledger_hash,
                on_result,
                pending_id,
                from_source,
            } => {
                mina_core::log::info!(
                    mina_core::log::system_time();
                    kind = "TransactionPoolEffectfulFetchAccounts",
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
                        mina_core::log::error!(
                                mina_core::log::system_time();
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

                store.dispatch_callback(on_result, (accounts, pending_id, from_source));
            }
        }
    }
}
