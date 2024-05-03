use openmina_core::debug;
use redux::ActionMeta;

use super::{
    bootstrap::P2pNetworkKadBootstrapAction, request::P2pNetworkKadRequestAction,
    stream::P2pNetworkKademliaStreamAction, P2pNetworkKadState, P2pNetworkKademliaAction,
};

use super::P2pNetworkKadAction;

use crate::P2pNetworkKademliaRpcReply;

impl P2pNetworkKadAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store) -> Result<(), String>
    where
        Store: crate::P2pStore<S>,
    {
        match self {
            crate::P2pNetworkKadAction::System(action) => action.effects(meta, store),
            crate::P2pNetworkKadAction::Bootstrap(action) => action.effects(meta, store),
            crate::P2pNetworkKadAction::Request(action) => action.effects(meta, store),
            crate::P2pNetworkKadAction::Stream(action) => action.effects(meta, store),
        }
    }
}

impl P2pNetworkKademliaAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store) -> Result<(), String>
    where
        Store: crate::P2pStore<S>,
    {
        // use super::P2pNetworkKadStatus::*;
        use super::P2pNetworkKademliaAction::*;
        let Some(state) = &store.state().network.scheduler.discovery_state else {
            return Err(String::from("peer discovery is not configured"));
        };

        match (self, &state.status) {
            (
                AnswerFindNodeRequest {
                    addr,
                    peer_id,
                    stream_id,
                    key,
                },
                _,
            ) => {
                let kad_key = key.into();
                let closer_peers = state
                    .routing_table
                    .find_node(&kad_key)
                    .cloned()
                    .collect::<Vec<_>>();

                debug!(meta.time(); "found {} peers", closer_peers.len());
                let message = P2pNetworkKademliaRpcReply::FindNode { closer_peers };
                store.dispatch(P2pNetworkKademliaStreamAction::SendResponse {
                    addr,
                    peer_id,
                    stream_id,
                    data: message,
                });
                Ok(())
            }
            (
                UpdateFindNodeRequest {
                    addr: _,
                    peer_id,
                    stream_id,
                    closest_peers,
                },
                _,
            ) => {
                let data = closest_peers.clone();
                store.dispatch(P2pNetworkKadRequestAction::ReplyReceived {
                    peer_id,
                    stream_id,
                    data,
                });
                Ok(())
            }
            (StartBootstrap { .. }, _) => {
                if store
                    .state()
                    .network
                    .scheduler
                    .discovery_state
                    .as_ref()
                    .and_then(P2pNetworkKadState::bootstrap_state)
                    .map_or(false, |bootstrap_state| bootstrap_state.requests.len() < 3)
                {
                    store.dispatch(P2pNetworkKadBootstrapAction::CreateRequests);
                }

                Ok(())
            }
            (BootstrapFinished {}, _) => Ok(()),
            (UpdateRoutingTable { .. }, _) => Ok(()),
        }
    }
}
