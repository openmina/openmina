use redux::{ActionMeta, EnablingCondition};

use super::{
    bootstrap::P2pNetworkKadBootstrapAction, request::P2pNetworkKadRequestAction,
    stream::P2pNetworkKademliaStreamAction, P2pNetworkKadState, P2pNetworkKademliaAction,
};

use super::P2pNetworkKadAction;

use crate::P2pNetworkKademliaRpcReply;
use crate::{
    connection::outgoing::P2pConnectionOutgoingAction, P2pNetworkYamuxOpenStreamAction,
    P2pNetworkYamuxOutgoingDataAction,
};

impl P2pNetworkKadAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store) -> Result<(), String>
    where
        Store: crate::P2pStore<S>,
        P2pNetworkYamuxOpenStreamAction: EnablingCondition<S>,
        P2pNetworkYamuxOutgoingDataAction: EnablingCondition<S>,
        P2pConnectionOutgoingAction: EnablingCondition<S>,
        P2pNetworkKademliaAction: EnablingCondition<S>,
        P2pNetworkKadBootstrapAction: EnablingCondition<S>,
        P2pNetworkKadRequestAction: EnablingCondition<S>,
        P2pNetworkKademliaStreamAction: EnablingCondition<S>,
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
    pub fn effects<Store, S>(self, _meta: &ActionMeta, store: &mut Store) -> Result<(), String>
    where
        Store: crate::P2pStore<S>,
        P2pNetworkKademliaStreamAction: redux::EnablingCondition<S>,
        P2pNetworkKadRequestAction: redux::EnablingCondition<S>,
        P2pNetworkKadBootstrapAction: redux::EnablingCondition<S>,
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

                println!("=== found {} peers", closer_peers.len());
                let message = P2pNetworkKademliaRpcReply::FindNode { closer_peers };
                store.dispatch(P2pNetworkKademliaStreamAction::SendReply {
                    addr,
                    peer_id,
                    stream_id,
                    data: message,
                });
                Ok(())
            }
            (
                UpdateFindNodeRequest {
                    addr,
                    peer_id: _,
                    stream_id: _,
                    closest_peers,
                },
                _,
            ) => {
                let bootstrap_request = state
                    .bootstrap_state()
                    .and_then(|bootstrap_state| bootstrap_state.request(&addr))
                    .is_some();
                store.dispatch(P2pNetworkKadRequestAction::ReplyReceived {
                    addr,
                    data: closest_peers.clone(),
                });
                if bootstrap_request {
                    store.dispatch(P2pNetworkKadBootstrapAction::RequestDone {
                        addr,
                        closest_peers,
                    });
                }
                Ok(())
            }
            (StartBootstrap { .. }, _) => {
                while store
                    .state()
                    .network
                    .scheduler
                    .discovery_state
                    .as_ref()
                    .and_then(P2pNetworkKadState::bootstrap_state)
                    .map_or(false, |bootstrap_state| {
                        bootstrap_state.requests.len() < 3 && !bootstrap_state.queue.is_empty()
                    })
                {
                    store.dispatch(P2pNetworkKadBootstrapAction::CreateRequests {});
                }

                Ok(())
            }
            (BootstrapFinished {}, _) => Ok(()),
        }
    }
}