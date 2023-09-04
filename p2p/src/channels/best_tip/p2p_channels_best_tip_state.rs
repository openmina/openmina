use openmina_core::block::ArcBlockWithHash;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pChannelsBestTipState {
    Disabled,
    Enabled,
    Init {
        time: redux::Timestamp,
    },
    Pending {
        time: redux::Timestamp,
    },
    Ready {
        time: redux::Timestamp,
        /// We are the requestors here.
        local: BestTipPropagationState,
        /// We are the responders here.
        remote: BestTipPropagationState,
        last_sent: Option<ArcBlockWithHash>,
        last_received: Option<ArcBlockWithHash>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BestTipPropagationState {
    WaitingForRequest { time: redux::Timestamp },
    Requested { time: redux::Timestamp },
    Responded { time: redux::Timestamp },
}

impl P2pChannelsBestTipState {
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }
}
