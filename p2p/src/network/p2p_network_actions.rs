use serde::{Deserialize, Serialize};

use super::{connection::*, noise::*, pnet::*, select::*};

use crate::P2pState;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkAction {
    Connection(P2pNetworkConnectionAction),
    Pnet(P2pNetworkPnetAction),
    Select(P2pNetworkSelectAction),
    Noise(P2pNetworkNoiseAction),
}

impl redux::EnablingCondition<P2pState> for P2pNetworkAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        match self {
            Self::Connection(v) => v.is_enabled(state),
            Self::Pnet(v) => v.is_enabled(state),
            Self::Select(v) => v.is_enabled(state),
            Self::Noise(v) => v.is_enabled(state),
        }
    }
}
