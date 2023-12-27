use serde::{Deserialize, Serialize};

use super::{noise::*, pnet::*, scheduler::*, select::*, yamux::*};

use crate::P2pState;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkAction {
    Scheduler(P2pNetworkSchedulerAction),
    Pnet(P2pNetworkPnetAction),
    Select(P2pNetworkSelectAction),
    Noise(P2pNetworkNoiseAction),
    Yamux(P2pNetworkYamuxAction),
}

impl redux::EnablingCondition<P2pState> for P2pNetworkAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        match self {
            Self::Scheduler(v) => v.is_enabled(state),
            Self::Pnet(v) => v.is_enabled(state),
            Self::Select(v) => v.is_enabled(state),
            Self::Noise(v) => v.is_enabled(state),
            Self::Yamux(v) => v.is_enabled(state),
        }
    }
}
