mod p2p_rpc_outgoing_state;
pub use p2p_rpc_outgoing_state::*;

mod p2p_rpc_outgoing_actions;
pub use p2p_rpc_outgoing_actions::*;

mod p2p_rpc_outgoing_reducer;
pub use p2p_rpc_outgoing_reducer::*;

mod p2p_rpc_outgoing_effects;
pub use p2p_rpc_outgoing_effects::*;

use mina_p2p_messages::v2::{NonZeroCurvePoint, StateHash};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pRpcRequestor {
    WatchedAccount(P2pRpcRequestorWatchedAccount),
    Interval,
    Other,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pRpcRequestorWatchedAccount {
    BlockLedgerGet(NonZeroCurvePoint, StateHash),
    LedgerInitialGet(NonZeroCurvePoint),
}

impl Default for P2pRpcRequestor {
    fn default() -> Self {
        Self::Other
    }
}
