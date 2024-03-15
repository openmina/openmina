use redux::ActionMeta;

use crate::{request::P2pNetworkKadRequestAction, P2pNetworkKademliaAction, P2pStore};

use super::P2pNetworkKadBootstrapAction;

impl P2pNetworkKadBootstrapAction {
    pub fn effects<Store, S>(self, _meta: &ActionMeta, store: &mut Store) -> Result<(), String>
    where
        Store: P2pStore<S>,
    {
        let discovery_state = &store
            .state()
            .network
            .scheduler
            .discovery_state()
            .ok_or_else(|| "discovery is not configured".to_string())?;
        let bootstrap_state = discovery_state
            .bootstrap_state()
            .ok_or_else(|| format!("action {self:?} is not allowed if not bootstrapping"))?;

        use P2pNetworkKadBootstrapAction as A;
        match self {
            A::CreateRequests {} => {
                if bootstrap_state.requests.is_empty() {
                    // no request is added, and none is in progress -> bootstrap is done.
                    store.dispatch(P2pNetworkKademliaAction::BootstrapFinished {});
                } else {
                    // start FIND_NODE request for each address if there is no such request already.
                    let key = bootstrap_state.key;
                    bootstrap_state
                        .requests
                        .iter()
                        .filter_map(|(peer_id, req)| {
                            (!discovery_state.requests.contains_key(peer_id))
                                .then_some((*peer_id, req.addr))
                        })
                        .collect::<Vec<_>>()
                        .into_iter()
                        .for_each(|(peer_id, addr)| {
                            store.dispatch(P2pNetworkKadRequestAction::New { addr, peer_id, key });
                        });
                }
                Ok(())
            }
            A::RequestDone { .. } => {
                if bootstrap_state.successful_requests < 20 {
                    store.dispatch(A::CreateRequests {});
                }
                Ok(())
            }
            A::RequestError { .. } => {
                store.dispatch(A::CreateRequests {});
                Ok(())
            }
        }
    }
}
