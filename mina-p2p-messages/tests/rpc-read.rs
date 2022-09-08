use mina_p2p_messages::{
    rpc::{Message, RpcMethod, DebuggerMessage},
    GetEpochLedger, GetBestTip,
};

#[macro_use]
mod utils;

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
    DebuggerMessage<<GetBestTip as RpcMethod>::Query>
);

binprot_read_test!(
    get_best_tip_response,
    "rpc-debugger/get-best-tip/response",
    DebuggerMessage<<GetBestTip as RpcMethod>::Response>
);
