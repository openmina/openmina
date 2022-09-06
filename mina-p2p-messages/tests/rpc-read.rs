use mina_p2p_messages::{
    rpc::{Message, RpcMethod},
    GetEpochLedger,
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
