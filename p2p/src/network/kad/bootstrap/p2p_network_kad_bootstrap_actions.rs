use redux::EnablingCondition;
use serde::{Deserialize, Serialize};

use crate::{P2pAction, P2pNetworkKadAction, P2pNetworkKadLatestRequestPeers, P2pState, PeerId};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum P2pNetworkKadBootstrapAction {
    CreateRequests,
    RequestDone {
        peer_id: PeerId,
        closest_peers: P2pNetworkKadLatestRequestPeers,
    },
    RequestError {
        peer_id: PeerId,
        error: String,
    },
}

impl EnablingCondition<P2pState> for P2pNetworkKadBootstrapAction {
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
        match self {
            P2pNetworkKadBootstrapAction::CreateRequests => state
                .network
                .scheduler
                .discovery_state
                .as_ref()
                .and_then(|discovery_state| discovery_state.bootstrap_state())
                .map_or(false, |bootstrap_state| bootstrap_state.requests.len() < 3),
            P2pNetworkKadBootstrapAction::RequestDone { peer_id, .. }
            | P2pNetworkKadBootstrapAction::RequestError { peer_id, .. } => state
                .network
                .scheduler
                .discovery_state
                .as_ref()
                .and_then(|discovery_state| discovery_state.bootstrap_state())
                .map_or(false, |bootstrap_state| {
                    bootstrap_state.request(peer_id).is_some()
                }),
        }
    }
}

impl From<P2pNetworkKadBootstrapAction> for P2pAction {
    fn from(value: P2pNetworkKadBootstrapAction) -> Self {
        P2pNetworkKadAction::Bootstrap(value).into()
    }
}
