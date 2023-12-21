use serde::{Deserialize, Serialize};

use super::{connection::P2pNetworkConnectionAction, pnet::P2pNetworkPnetAction};

use crate::P2pState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkAction {
    Connection(P2pNetworkConnectionAction),
    Pnet(P2pNetworkPnetAction),
}

impl redux::EnablingCondition<P2pState> for P2pNetworkAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        match self {
            Self::Connection(v) => v.is_enabled(state),
            Self::Pnet(v) => v.is_enabled(state),
        }
    }
}
