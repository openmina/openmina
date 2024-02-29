use super::{
    account_relevant_transactions_in_diff_iter, WatchedAccountBlockInfo, WatchedAccountBlockState,
    WatchedAccountLedgerInitialState, WatchedAccountState, WatchedAccountsAction,
    WatchedAccountsActionWithMetaRef, WatchedAccountsState,
};

impl WatchedAccountsState {
    pub fn reducer(&mut self, action: WatchedAccountsActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            WatchedAccountsAction::Add { pub_key } => {
                self.insert(
                    pub_key.clone(),
                    WatchedAccountState {
                        initial_state: WatchedAccountLedgerInitialState::Idle { time: meta.time() },
                        blocks: Default::default(),
                    },
                );
            },
            WatchedAccountsAction::LedgerInitialStateGetInit { .. } => {},
            WatchedAccountsAction::LedgerInitialStateGetPending { pub_key, block, peer_id } => {
                let Some(account) = self.get_mut(pub_key) else {
                    return;
                };
                account.blocks.clear();

                account.initial_state = WatchedAccountLedgerInitialState::Pending {
                    time: meta.time(),
                    block: block.clone(),
                    peer_id: *peer_id,
                };
            },
            WatchedAccountsAction::LedgerInitialStateGetError { pub_key, error } => {
                let Some(account) = self.get_mut(pub_key) else {
                    return;
                };
                let peer_id = match &account.initial_state {
                    WatchedAccountLedgerInitialState::Pending { peer_id, .. } => *peer_id,
                    _ => return,
                };
                account.initial_state = WatchedAccountLedgerInitialState::Error {
                    time: meta.time(),
                    error: error.clone(),
                    peer_id,
                };
            },
            WatchedAccountsAction::LedgerInitialStateGetRetry { .. } => {},
            WatchedAccountsAction::LedgerInitialStateGetSuccess { pub_key, data } => {
                let Some(account) = self.get_mut(pub_key) else {
                    return;
                };
                let Some(block) = account.initial_state.block() else {
                    return;
                };
                account.initial_state = WatchedAccountLedgerInitialState::Success {
                    time: meta.time(),
                    block: block.clone(),
                    data: data.clone(),
                };
            },
            WatchedAccountsAction::TransactionsIncludedInBlock { pub_key, block } => {
                let transactions =
                    account_relevant_transactions_in_diff_iter(pub_key, &block.block.body.staged_ledger_diff.diff).collect();

                let Some(account) = self.get_mut(pub_key) else {
                    return;
                };
                account
                    .blocks
                    .push_back(WatchedAccountBlockState::TransactionsInBlockBody {
                        block: WatchedAccountBlockInfo {
                            level: block.block.header.protocol_state.body.consensus_state.blockchain_length.0 .0 as u32,
                            hash: block.hash.clone(),
                            pred_hash: block.block.header.protocol_state.previous_state_hash.clone(),
                            staged_ledger_hash: block.block.header.protocol_state.body.blockchain_state.staged_ledger_hash.non_snark.ledger_hash.clone(),
                        },
                        transactions,
                    });
            },
            WatchedAccountsAction::BlockLedgerQueryInit { .. } => {},
            WatchedAccountsAction::BlockLedgerQueryPending { pub_key, block_hash, .. } => {
                let Some(account) = self.get_mut(pub_key) else {
                    return;
                };
                let Some(block_state) = account.block_find_by_hash_mut(block_hash) else {
                    return;
                };
                *block_state = match block_state {
                    WatchedAccountBlockState::TransactionsInBlockBody { block, transactions } => WatchedAccountBlockState::LedgerAccountGetPending {
                        block: block.clone(),
                        transactions: std::mem::take(transactions),
                    },
                    _ => return,
                };
            },
            WatchedAccountsAction::BlockLedgerQuerySuccess { pub_key, block_hash, ledger_account } => {
                let Some(account) = self.get_mut(pub_key) else {
                    return;
                };
                let Some(block_state) = account.block_find_by_hash_mut(block_hash) else {
                    return;
                };
                *block_state = match block_state {
                    WatchedAccountBlockState::LedgerAccountGetPending { block, transactions, .. } => WatchedAccountBlockState::LedgerAccountGetSuccess {
                        block: block.clone(),
                        transactions: std::mem::take(transactions),
                        ledger_account: ledger_account.clone(),
                    },
                    _ => return,
                };
            },
        }
    }
}
