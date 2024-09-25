use multiaddr::Multiaddr;
use openmina_core::ActionEvent;
use redux::EnablingCondition;
use serde::{Deserialize, Serialize};

use crate::{
    kad::stream::P2pNetworkKademliaStreamAction, request::P2pNetworkKadRequestAction,
    ConnectionAddr, P2pAction, P2pNetworkAction, P2pNetworkKadEntry, P2pState, PeerId, StreamId,
};

use super::{bootstrap::P2pNetworkKadBootstrapAction, CID};

/// Kademlia actions.
#[derive(Debug, Clone, Serialize, Deserialize, derive_more::From, ActionEvent)]
pub enum P2pNetworkKadAction {
    System(P2pNetworkKademliaAction),
    Bootstrap(P2pNetworkKadBootstrapAction),
    Request(P2pNetworkKadRequestAction),
    Stream(P2pNetworkKademliaStreamAction),
}

impl EnablingCondition<P2pState> for P2pNetworkKadAction {
    fn is_enabled(&self, state: &P2pState, time: redux::Timestamp) -> bool {
        match self {
            P2pNetworkKadAction::System(action) => action.is_enabled(state, time),
            P2pNetworkKadAction::Bootstrap(action) => action.is_enabled(state, time),
            P2pNetworkKadAction::Request(action) => action.is_enabled(state, time),
            P2pNetworkKadAction::Stream(action) => action.is_enabled(state, time),
        }
    }
}

impl From<P2pNetworkKadAction> for P2pAction {
    fn from(value: P2pNetworkKadAction) -> Self {
        P2pNetworkAction::Kad(value).into()
    }
}

/// Kademlia system actions
#[derive(Debug, Clone, Serialize, Deserialize, ActionEvent)]
#[action_event(fields(
    display(addr),
    display(peer_id),
    stream_id,
    debug(key),
    debug(closest_peers),
    debug(addrs)
))]
pub enum P2pNetworkKademliaAction {
    /// Answer `FIND_NODE` request.
    ///
    /// Answers peer's `FIND_NODE` request by querying routing table for closest nodes.
    AnswerFindNodeRequest {
        addr: ConnectionAddr,
        peer_id: PeerId,
        stream_id: StreamId,
        key: CID,
    },
    /// Udate result of scheduled outgoing `FIND_NODE`.
    ///
    /// Udates result of scheduled outgoing `FIND_NODE` request to a peer.
    UpdateFindNodeRequest {
        addr: ConnectionAddr,
        peer_id: PeerId,
        stream_id: StreamId,
        closest_peers: Vec<P2pNetworkKadEntry>,
    },
    /// Perform local node's Kademlia bootstrap.
    #[action_event(level = info)]
    StartBootstrap { key: PeerId },
    /// Bootstrap is finished.
    BootstrapFinished,

    /// Update routing table with peer addresses
    #[action_event(level = info)]
    UpdateRoutingTable {
        peer_id: PeerId,
        addrs: Vec<Multiaddr>,
    },
}

impl EnablingCondition<P2pState> for P2pNetworkKademliaAction {
    fn is_enabled(&self, state: &P2pState, time: redux::Timestamp) -> bool {
        let Some(discovery_state) = &state.network.scheduler.discovery_state else {
            return false;
        };
        match self {
            P2pNetworkKademliaAction::AnswerFindNodeRequest {
                peer_id, stream_id, ..
            } => discovery_state
                .find_kad_stream_state(peer_id, stream_id)
                .is_some(),
            P2pNetworkKademliaAction::UpdateFindNodeRequest {
                addr: _,
                peer_id,
                stream_id,
                ..
            } => {
                discovery_state
                    .find_kad_stream_state(peer_id, stream_id)
                    .is_some()
                    && discovery_state.request(peer_id).is_some()
            }
            P2pNetworkKademliaAction::StartBootstrap { .. } => discovery_state
                .status
                .can_bootstrap(time, &state.config.timeouts),
            P2pNetworkKademliaAction::BootstrapFinished { .. } => {
                // TODO: also can run bootstrap on timely basis.
                matches!(
                    discovery_state.status,
                    super::P2pNetworkKadStatus::Bootstrapping(_)
                )
            }
            P2pNetworkKademliaAction::UpdateRoutingTable { .. } => true,
        }
    }
}

impl From<P2pNetworkKademliaAction> for P2pAction {
    fn from(value: P2pNetworkKademliaAction) -> Self {
        P2pAction::Network(P2pNetworkKadAction::System(value).into())
    }
}
