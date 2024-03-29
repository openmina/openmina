use redux::EnablingCondition;
use serde::{Deserialize, Serialize};

use crate::{P2pListenerId, P2pState};

pub type P2pListenActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pListenAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pListenAction {
    New {
        listener_id: P2pListenerId,
        addr: multiaddr::Multiaddr,
    },
    Expired {
        listener_id: P2pListenerId,
        addr: multiaddr::Multiaddr,
    },
    Error {
        listener_id: P2pListenerId,
        error: String,
    },
    Closed {
        listener_id: P2pListenerId,
        error: Option<String>,
    },
}

impl EnablingCondition<P2pState> for P2pListenAction {}
