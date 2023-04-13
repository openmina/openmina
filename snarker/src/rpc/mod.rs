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

use serde::{Deserialize, Serialize};
pub use shared::requests::{RpcId, RpcIdType};

use crate::p2p::channels::snark_job_commitment::SnarkJobId;
use crate::p2p::connection::incoming::P2pConnectionIncomingInitOpts;
use crate::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RpcRequest {
    GetState,
    ActionStatsGet(ActionStatsQuery),
    P2pConnectionOutgoing(P2pConnectionOutgoingInitOpts),
    P2pConnectionIncoming(P2pConnectionIncomingInitOpts),
    SnarkerJobPickAndCommit { available_jobs: Vec<SnarkJobId> },
}
