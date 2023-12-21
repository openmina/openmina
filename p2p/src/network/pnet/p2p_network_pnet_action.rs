use serde::{Deserialize, Serialize};

use crate::P2pState;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkPnetAction {}

impl redux::EnablingCondition<P2pState> for P2pNetworkPnetAction {
    fn is_enabled(&self, _state: &P2pState) -> bool {
        true
    }
}
