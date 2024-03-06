use serde::{Deserialize, Serialize};

use crate::{P2pPeerState, P2pPeerStatus, PeerId};

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkState {
    pub scheduler: P2pNetworkSchedulerState,
}
