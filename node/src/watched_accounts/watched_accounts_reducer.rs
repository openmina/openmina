use super::{
    account_relevant_transactions_in_diff_iter, WatchedAccountBlockInfo, WatchedAccountBlockState,
    WatchedAccountLedgerInitialState, WatchedAccountState, WatchedAccountsAction,
    WatchedAccountsActionWithMetaRef, WatchedAccountsState,
};

impl WatchedAccountsState {
    pub fn reducer(
        mut state_context: crate::Substate<Self>,
        action: WatchedAccountsActionWithMetaRef<'_>,
    ) {
        let Ok(state) = state_context.get_substate_mut() else {
            // TODO: log or propagate
            return;
        };
        let (action, meta) = action.split();

        match action {
            WatchedAccountsAction::Add { pub_key } => {
                state.insert(
                    pub_key.clone(),
                    WatchedAccountState {
                        initial_state: WatchedAccountLedgerInitialState::Idle { time: meta.time() },
                        blocks: Default::default(),
                    },
                );

                // Dispatch
                let pub_key = pub_key.clone();
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(WatchedAccountsAction::LedgerInitialStateGetInit { pub_key });
            }
            WatchedAccountsAction::LedgerInitialStateGetInit { pub_key: _ }
            | WatchedAccountsAction::LedgerInitialStateGetRetry { pub_key: _ } => {
                // TODO(binier)
                // let Some((peer_id, p2p_rpc_id)) = store.state().p2p.get_free_peer_id_for_rpc() else { return };
                // let block = {
                //     let Some(block) = store.state().consensus.best_tip() else { return };
                //     WatchedAccountBlockInfo {
                //         level: block.height() as u32,
                //         hash: block.hash.clone(),
                //         pred_hash: block.header.protocol_state.previous_state_hash.clone(),
                //         staged_ledger_hash: block
                //             .header
                //             .protocol_state
                //             .body
                //             .blockchain_state
                //             .staged_ledger_hash
                //             .non_snark
                //             .ledger_hash
                //             .clone(),
                //     }
                // };

                // let token_id = MinaBaseAccountIdDigestStableV1(BigInt::one());

                // dispatcher.push(P2pRpcOutgoingInitAction {
                //     peer_id: peer_id.clone(),
                //     rpc_id: p2p_rpc_id,
                //     request: P2pRpcRequest::LedgerQuery((
                //         block.staged_ledger_hash.0.clone(),
                //         MinaLedgerSyncLedgerQueryStableV1::WhatAccountWithPath(
                //             pub_key.clone(),
                //             token_id.into(),
                //         ),
                //     )),
                //     requestor: P2pRpcRequestor::WatchedAccount(
                //         P2pRpcRequestorWatchedAccount::LedgerInitialGet(pub_key.clone()),
                //     ),
                // });
                // dispatcher.push(WatchedAccountsLedgerInitialStateGetPendingAction {
                //     pub_key,
                //     block,
                //     peer_id,
                //     p2p_rpc_id,
                // });
            }
            WatchedAccountsAction::LedgerInitialStateGetPending {
                pub_key,
                block,
                peer_id,
            } => {
                let Some(account) = state.get_mut(pub_key) else {
                    return;
                };
                account.blocks.clear();

                account.initial_state = WatchedAccountLedgerInitialState::Pending {
                    time: meta.time(),
                    block: block.clone(),
                    peer_id: *peer_id,
                };
            }
            WatchedAccountsAction::LedgerInitialStateGetError { pub_key, error } => {
                let Some(account) = state.get_mut(pub_key) else {
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
            }
            WatchedAccountsAction::LedgerInitialStateGetSuccess { pub_key, data } => {
                let Some(account) = state.get_mut(pub_key) else {
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
            }
            WatchedAccountsAction::TransactionsIncludedInBlock { pub_key, block } => {
                let transactions = account_relevant_transactions_in_diff_iter(
                    pub_key,
                    &block.block.body.staged_ledger_diff.diff,
                )
                .collect();

                let Some(account) = state.get_mut(pub_key) else {
                    return;
                };
                account
                    .blocks
                    .push_back(WatchedAccountBlockState::TransactionsInBlockBody {
                        block: WatchedAccountBlockInfo {
                            level: block
                                .block
                                .header
                                .protocol_state
                                .body
                                .consensus_state
                                .blockchain_length
                                .0
                                 .0,
                            hash: block.hash.clone(),
                            pred_hash: block
                                .block
                                .header
                                .protocol_state
                                .previous_state_hash
                                .clone(),
                            staged_ledger_hash: block
                                .block
                                .header
                                .protocol_state
                                .body
                                .blockchain_state
                                .staged_ledger_hash
                                .non_snark
                                .ledger_hash
                                .clone(),
                        },
                        transactions,
                    });

                let pub_key = pub_key.clone();
                let block_hash = block.hash.clone();
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(WatchedAccountsAction::BlockLedgerQueryInit {
                    pub_key,
                    block_hash,
                });
            }
            WatchedAccountsAction::BlockLedgerQueryInit { .. } => {
                // TODO(binier)
                // let Some((peer_id, p2p_rpc_id)) = store.state().p2p.get_free_peer_id_for_rpc() else { return };
                // let ledger_hash = {
                //     let Some(acc) = store.state().watched_accounts.get(&action.pub_key) else { return };
                //     let Some(block) = acc.block_find_by_hash(&action.block_hash) else { return };
                //     block.block().staged_ledger_hash.0.clone()
                // };
                // let token_id = MinaBaseAccountIdDigestStableV1(BigInt::one());

                // store.dispatch(P2pRpcOutgoingInitAction {
                //     peer_id: peer_id.clone(),
                //     rpc_id: p2p_rpc_id,
                //     request: P2pRpcRequest::LedgerQuery((
                //         ledger_hash,
                //         MinaLedgerSyncLedgerQueryStableV1::WhatAccountWithPath(
                //             action.pub_key.clone(),
                //             token_id.into(),
                //         ),
                //     )),
                //     requestor: P2pRpcRequestor::WatchedAccount(
                //         P2pRpcRequestorWatchedAccount::BlockLedgerGet(
                //             action.pub_key.clone(),
                //             action.block_hash.clone(),
                //         ),
                //     ),
                // });
                // store.dispatch(WatchedAccountsBlockLedgerQueryPendingAction {
                //     pub_key: action.pub_key,
                //     block_hash: action.block_hash,
                //     peer_id,
                //     p2p_rpc_id,
                // });
            }
            WatchedAccountsAction::BlockLedgerQueryPending {
                pub_key,
                block_hash,
                ..
            } => {
                let Some(account) = state.get_mut(pub_key) else {
                    return;
                };
                let Some(block_state) = account.block_find_by_hash_mut(block_hash) else {
                    return;
                };
                *block_state = match block_state {
                    WatchedAccountBlockState::TransactionsInBlockBody {
                        block,
                        transactions,
                    } => WatchedAccountBlockState::LedgerAccountGetPending {
                        block: block.clone(),
                        transactions: std::mem::take(transactions),
                    },
                    _ => return,
                };
            }
            WatchedAccountsAction::BlockLedgerQuerySuccess {
                pub_key,
                block_hash,
                ledger_account,
            } => {
                let Some(account) = state.get_mut(pub_key) else {
                    return;
                };
                let Some(block_state) = account.block_find_by_hash_mut(block_hash) else {
                    return;
                };
                *block_state = match block_state {
                    WatchedAccountBlockState::LedgerAccountGetPending {
                        block,
                        transactions,
                        ..
                    } => WatchedAccountBlockState::LedgerAccountGetSuccess {
                        block: block.clone(),
                        transactions: std::mem::take(transactions),
                        ledger_account: ledger_account.clone(),
                    },
                    _ => return,
                };
            }
        }
    }
}
