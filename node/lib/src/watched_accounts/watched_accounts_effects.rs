use mina_p2p_messages::{
    bigint::BigInt,
    v2::{MinaBaseAccountIdDigestStableV1, MinaLedgerSyncLedgerQueryStableV1},
};
use p2p::rpc::{
    outgoing::{P2pRpcOutgoingInitAction, P2pRpcRequestor},
    P2pRpcRequest,
};

use crate::Store;

use super::{
    WatchedAccountsAction, WatchedAccountsActionWithMeta,
    WatchedAccountsBlockLedgerQueryInitAction, WatchedAccountsBlockLedgerQueryPendingAction,
};

pub fn watched_accounts_effects<S: redux::Service>(
    store: &mut Store<S>,
    action: WatchedAccountsActionWithMeta,
) {
    let (action, meta) = action.split();

    match action {
        WatchedAccountsAction::Add(_) => {}
        WatchedAccountsAction::TransactionsIncludedInBlock(action) => {
            store.dispatch(WatchedAccountsBlockLedgerQueryInitAction {
                pub_key: action.pub_key,
                block_hash: action.block.hash,
            });
        }
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
                    action.pub_key.clone(),
                    action.block_hash.clone(),
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
