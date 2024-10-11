use serde::{Deserialize, Serialize};

use super::exchange::P2pChannelsSignalingExchangeState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsSignalingState {
    pub exchange: P2pChannelsSignalingExchangeState,
}
