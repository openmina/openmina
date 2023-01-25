use mina_p2p_messages::{
    bigint::BigInt,
    v2::{MinaBaseAccountIdDigestStableV1, MinaLedgerSyncLedgerQueryStableV1},
};
use p2p::rpc::{
    outgoing::{P2pRpcOutgoingInitAction, P2pRpcRequestor, P2pRpcRequestorWatchedAccount},
    P2pRpcRequest,
};

use crate::Store;

use super::{
    WatchedAccountBlockInfo, WatchedAccountsAction, WatchedAccountsActionWithMeta,
    WatchedAccountsBlockLedgerQueryInitAction, WatchedAccountsBlockLedgerQueryPendingAction,
    WatchedAccountsLedgerInitialStateGetInitAction,
    WatchedAccountsLedgerInitialStateGetPendingAction,
    WatchedAccountsLedgerInitialStateGetRetryAction,
};

pub fn watched_accounts_effects<S: redux::Service>(
    store: &mut Store<S>,
    action: WatchedAccountsActionWithMeta,
) {
    let (action, _) = action.split();

    match action {
        WatchedAccountsAction::Add(action) => {
            store.dispatch(WatchedAccountsLedgerInitialStateGetInitAction {
                pub_key: action.pub_key.clone(),
            });
        }
        WatchedAccountsAction::TransactionsIncludedInBlock(action) => {
            store.dispatch(WatchedAccountsBlockLedgerQueryInitAction {
                pub_key: action.pub_key,
                block_hash: action.block.hash,
            });
        }
        WatchedAccountsAction::LedgerInitialStateGetInit(
            WatchedAccountsLedgerInitialStateGetInitAction { pub_key },
        )
        | WatchedAccountsAction::LedgerInitialStateGetRetry(
            WatchedAccountsLedgerInitialStateGetRetryAction { pub_key },
        ) => {
            let Some((peer_id, p2p_rpc_id)) = store.state().p2p.get_free_peer_id_for_rpc() else { return };
            let block = {
                let Some(block) = store.state().consensus.best_tip() else { return };
                WatchedAccountBlockInfo {
                    level: block.height() as u32,
                    hash: block.hash.clone(),
                    pred_hash: block.header.protocol_state.previous_state_hash.clone(),
                    staged_ledger_hash: block
                        .header
                        .protocol_state
                        .body
                        .blockchain_state
                        .staged_ledger_hash
                        .non_snark
                        .ledger_hash
                        .clone(),
                }
            };

            let token_id = MinaBaseAccountIdDigestStableV1(BigInt::one());

            store.dispatch(P2pRpcOutgoingInitAction {
                peer_id: peer_id.clone(),
                rpc_id: p2p_rpc_id,
                request: P2pRpcRequest::LedgerQuery((
                    block.staged_ledger_hash.0.clone(),
                    MinaLedgerSyncLedgerQueryStableV1::WhatAccountWithPath(
                        pub_key.clone(),
                        token_id.into(),
                    ),
                )),
                requestor: P2pRpcRequestor::WatchedAccount(
                    P2pRpcRequestorWatchedAccount::LedgerInitialGet(pub_key.clone()),
                ),
            });
            store.dispatch(WatchedAccountsLedgerInitialStateGetPendingAction {
                pub_key,
                block,
                peer_id,
                p2p_rpc_id,
            });
        }
        WatchedAccountsAction::LedgerInitialStateGetPending(_) => {}
        WatchedAccountsAction::LedgerInitialStateGetError(_) => {}
        WatchedAccountsAction::LedgerInitialStateGetSuccess(_) => {}
        WatchedAccountsAction::BlockLedgerQueryInit(action) => {
            let Some((peer_id, p2p_rpc_id)) = store.state().p2p.get_free_peer_id_for_rpc() else { return };
            let ledger_hash = {
                let Some(acc) = store.state().watched_accounts.get(&action.pub_key) else { return };
                let Some(block) = acc.block_find_by_hash(&action.block_hash) else { return };
                block.block().staged_ledger_hash.0.clone()
            };
            let token_id = MinaBaseAccountIdDigestStableV1(BigInt::one());

            store.dispatch(P2pRpcOutgoingInitAction {
                peer_id: peer_id.clone(),
                rpc_id: p2p_rpc_id,
                request: P2pRpcRequest::LedgerQuery((
                    ledger_hash,
                    MinaLedgerSyncLedgerQueryStableV1::WhatAccountWithPath(
                        action.pub_key.clone(),
                        token_id.into(),
                    ),
                )),
                requestor: P2pRpcRequestor::WatchedAccount(
                    P2pRpcRequestorWatchedAccount::BlockLedgerGet(
                        action.pub_key.clone(),
                        action.block_hash.clone(),
                    ),
                ),
            });
            store.dispatch(WatchedAccountsBlockLedgerQueryPendingAction {
                pub_key: action.pub_key,
                block_hash: action.block_hash,
                peer_id,
                p2p_rpc_id,
            });
        }
        WatchedAccountsAction::BlockLedgerQueryPending(_) => {}
        WatchedAccountsAction::BlockLedgerQuerySuccess(_) => {}
    }
}
