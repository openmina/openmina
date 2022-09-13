use mina_p2p_messages::{
    rpc::{DebuggerMessage, Message, RpcMethod},
    AnswerSyncLedgerQueryV1, GetBestTipV1, GetEpochLedger,
    GetStagedLedgerAuxAndPendingCoinbasesAtHashV1, GetTransitionChainV1, GetTransitionChainProofV1, GetAncestryV1, GetTransitionKnowledgeV1, VersionedRpcMenuV1,
};

#[macro_use]
mod utils;

binprot_read_test!(
    menu_query,
    "rpc-debugger/menu/query",
    DebuggerMessage<<VersionedRpcMenuV1 as RpcMethod>::Query>
);

binprot_read_test!(
    menu_response,
    "rpc-debugger/menu/response",
    DebuggerMessage<<VersionedRpcMenuV1 as RpcMethod>::Response>
);

binprot_read_test!(
    get_epoch_ledger_query,
    "rpc/get-epoch-ledger/queries",
    Message<<GetEpochLedger as RpcMethod>::Query>
);
binprot_read_test!(
    get_epoch_ledger_response,
    "rpc/get-epoch-ledger/responses",
    Message<<GetEpochLedger as RpcMethod>::Response>
);

binprot_read_test!(
    get_best_tip_query,
    "rpc-debugger/get-best-tip/query",
    DebuggerMessage<<GetBestTipV1 as RpcMethod>::Query>
);

binprot_read_test!(
    get_best_tip_response,
    "rpc-debugger/get-best-tip/response",
    DebuggerMessage<<GetBestTipV1 as RpcMethod>::Response>
);

// macro_rules! debugger_rpc_test {
//     ($name:ident, $rpc:ty, $path:literal) => {
//         binprot_read_test!(
//             concat_idents!($name, _query),
//             concat!("rpc-debugger/", $path, "/query"),
//             DebuggerMessage<<$rpc as RpcMethod>::Query>
//         );

//         binprot_read_test!(
//             concat_idents!($name, _response),
//             concat!("rpc-debugger/", $path, "/response"),
//             DebuggerMessage<<$rpc as RpcMethod>::Response>
//         );
//     };
// }

// debugger_rpc_test!(get_staged_ledger_aux, GetStagedLedgerAuxAndPendingCoinbasesAtHashV1, "get-staged-ledger-aux");

binprot_read_test!(
    get_staged_ledger_aux_query,
    "rpc-debugger/get-staged-ledger-aux/query",
    DebuggerMessage<<GetStagedLedgerAuxAndPendingCoinbasesAtHashV1 as RpcMethod>::Query>
);

binprot_read_test!(
    get_staged_ledger_aux_response,
    "rpc-debugger/get-staged-ledger-aux/response",
    DebuggerMessage<<GetStagedLedgerAuxAndPendingCoinbasesAtHashV1 as RpcMethod>::Response>
);

binprot_read_test!(
    answer_sync_ledger_query,
    "rpc-debugger/answer-sync-ledger/query",
    DebuggerMessage<<AnswerSyncLedgerQueryV1 as RpcMethod>::Query>
);

binprot_read_test!(
    answer_sync_ledger_response,
    "rpc-debugger/answer-sync-ledger/response",
    DebuggerMessage<<AnswerSyncLedgerQueryV1 as RpcMethod>::Response>
);


binprot_read_test!(
    get_transition_chain_query,
    "rpc-debugger/get-transition-chain/query",
    DebuggerMessage<<GetTransitionChainV1 as RpcMethod>::Query>
);

binprot_read_test!(
    get_transition_chain_response,
    "rpc-debugger/get-transition-chain/response",
    DebuggerMessage<<GetTransitionChainV1 as RpcMethod>::Response>
);


binprot_read_test!(
    get_transition_chain_proof_query,
    "rpc-debugger/get-transition-chain-proof/query",
    DebuggerMessage<<GetTransitionChainProofV1 as RpcMethod>::Query>
);

binprot_read_test!(
    get_transition_chain_proof_response,
    "rpc-debugger/get-transition-chain-proof/response",
    DebuggerMessage<<GetTransitionChainProofV1 as RpcMethod>::Response>
);


binprot_read_test!(
    ignore("No test data"),
    get_transition_knowledge_query,
    "rpc-debugger/get-transition-knowledge/query",
    DebuggerMessage<<GetTransitionKnowledgeV1 as RpcMethod>::Query>
);

binprot_read_test!(
    ignore("No test data"),
    get_transition_knowledge_response,
    "rpc-debugger/get-transition-knowledge/response",
    DebuggerMessage<<GetTransitionKnowledgeV1 as RpcMethod>::Response>
);


binprot_read_test!(
    get_ancestry_query,
    "rpc-debugger/get-ancestry/query",
    DebuggerMessage<<GetAncestryV1 as RpcMethod>::Query>
);

binprot_read_test!(
    get_ancestry_response,
    "rpc-debugger/get-ancestry/response",
    DebuggerMessage<<GetAncestryV1 as RpcMethod>::Response>
);
