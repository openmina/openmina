mod rpc_state;
pub use rpc_state::*;

mod rpc_actions;
pub use rpc_actions::*;

mod rpc_reducer;
pub use rpc_reducer::*;

mod rpc_effects;
pub use rpc_effects::*;

mod rpc_service;
pub use rpc_service::*;

use mina_p2p_messages::v2::{
    MinaBaseAccountBinableArgStableV2, NonZeroCurvePoint,
    StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B,
};
use serde::{Deserialize, Serialize};
pub use shared::requests::{RpcId, RpcIdType};

use crate::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
use crate::p2p::pubsub::{GossipNetMessageV2, PubsubTopic};
use crate::watched_accounts::WatchedAccountBlockState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RpcRequest {
    GetState,
    P2pConnectionOutgoing(P2pConnectionOutgoingInitOpts),
    P2pPubsubPublish(PubsubTopic, GossipNetMessageV2),
    WatchedAccountsAdd(NonZeroCurvePoint),
    WatchedAccountsGet(NonZeroCurvePoint),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WatchedAccountInfo {
    pub latest_state: Option<MinaBaseAccountBinableArgStableV2>,
    pub blocks: Vec<WatchedAccountBlockState>,
}
