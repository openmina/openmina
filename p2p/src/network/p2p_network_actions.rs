use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use super::{
    identify::*, kad::*, noise::*, pnet::*, pubsub::*, rpc::*, scheduler::*, select::*, yamux::*,
};

use crate::P2pState;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum P2pNetworkAction {
    Scheduler(P2pNetworkSchedulerAction),
    Pnet(P2pNetworkPnetAction),
    Select(P2pNetworkSelectAction),
    Noise(P2pNetworkNoiseAction),
    Yamux(P2pNetworkYamuxAction),
    Identify(P2pNetworkIdentifyAction),
    Kad(P2pNetworkKadAction),
    Pubsub(P2pNetworkPubsubAction),
    Rpc(P2pNetworkRpcAction),
}

impl redux::EnablingCondition<P2pState> for P2pNetworkAction {
    fn is_enabled(&self, state: &P2pState, time: redux::Timestamp) -> bool {
        match self {
            Self::Scheduler(v) => v.is_enabled(state, time),
            Self::Pnet(v) => v.is_enabled(state, time),
            Self::Select(v) => v.is_enabled(state, time),
            Self::Noise(v) => v.is_enabled(state, time),
            Self::Yamux(v) => v.is_enabled(state, time),
            Self::Identify(v) => v.is_enabled(state, time),
            Self::Kad(v) => v.is_enabled(state, time),
            Self::Pubsub(v) => v.is_enabled(state, time),
            Self::Rpc(v) => v.is_enabled(state, time),
        }
    }
}
