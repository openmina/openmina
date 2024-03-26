use redux::ActionWithMeta;

use super::{P2pNetworkKadAction, P2pNetworkKadLatestRequestPeerKind, P2pNetworkKadStatus};

use super::stream::P2pNetworkKademliaStreamAction;

impl super::P2pNetworkKadState {
    pub fn reducer(&mut self, action: ActionWithMeta<&P2pNetworkKadAction>) -> Result<(), String> {
        let (action, meta) = action.split();
        match action {
            P2pNetworkKadAction::System(action) => self.system_reducer(meta.with_action(action)),
            P2pNetworkKadAction::Bootstrap(action) => {
                if let P2pNetworkKadStatus::Bootstrapping(state) = &mut self.status {
                    state.reducer(
                        &self.routing_table,
                        meta.with_action(action),
                        self.filter_addrs,
                    )
                } else {
                    Err(format!("kademlia is not bootstrapping: {action:?}"))
                }
            }
            P2pNetworkKadAction::Request(
                action @ super::request::P2pNetworkKadRequestAction::New { addr, peer_id, key },
            ) => self
                .create_request(*addr, *peer_id, *key)
                .map_err(|_request| format!("kademlia request to {addr} is already in progress"))
                .and_then(|request| request.reducer(meta.with_action(action))),
            P2pNetworkKadAction::Request(super::request::P2pNetworkKadRequestAction::Prune {
                peer_id,
            }) => self
                .requests
                .remove(peer_id)
                .map(|_| ())
                .ok_or_else(|| format!("kademlia request for {peer_id} is not found")),
            P2pNetworkKadAction::Request(action) => self
                .requests
                .get_mut(action.peer_id())
                .ok_or_else(|| format!("kademlia request for {} is not found", action.peer_id()))
                .and_then(|request| request.reducer(meta.with_action(action))),

            P2pNetworkKadAction::Stream(action @ P2pNetworkKademliaStreamAction::New { .. }) => {
                self.create_kad_stream_state(action.peer_id(), action.stream_id())
                    .map_err(|stream| {
                        format!("kademlia stream already exists for action {action:?}: {stream:?}")
                    })
                    .and_then(|stream| stream.reducer(meta.with_action(action)))
            }
            P2pNetworkKadAction::Stream(action @ P2pNetworkKademliaStreamAction::Prune { .. }) => {
                self.remove_kad_stream_state(action.peer_id(), action.stream_id())
                    .then_some(())
                    .ok_or_else(|| format!("kademlia stream not found for action {action:?}"))
            }
            P2pNetworkKadAction::Stream(action) => self
                .find_kad_stream_state_mut(action.peer_id(), action.stream_id())
                .ok_or_else(|| format!("kademlia stream not found for action {action:?}"))
                .and_then(|stream| stream.reducer(meta.with_action(action))),
        }
    }

    pub fn system_reducer(
        &mut self,
        action: ActionWithMeta<&super::P2pNetworkKademliaAction>,
    ) -> Result<(), String> {
        use super::P2pNetworkKadStatus::*;
        use super::P2pNetworkKademliaAction::*;
        let (action, meta) = action.split();
        match (&mut self.status, action) {
            (_, AnswerFindNodeRequest { .. }) => Ok(()),
            (_, UpdateFindNodeRequest { closest_peers, .. }) => {
                let mut latest_request_peers = Vec::new();
                for entry in closest_peers {
                    let kind = match self.routing_table.insert(entry.clone()) {
                        Ok(true) => P2pNetworkKadLatestRequestPeerKind::New,
                        Ok(false) => P2pNetworkKadLatestRequestPeerKind::Existing,
                        Err(_) => P2pNetworkKadLatestRequestPeerKind::Discarded,
                    };
                    latest_request_peers.push((entry.peer_id, kind));
                }
                self.latest_request_peers = latest_request_peers.into();
                Ok(())
            }
            (_, StartBootstrap { key }) => {
                self.status =
                    Bootstrapping(super::bootstrap::P2pNetworkKadBootstrapState::new(*key));
                Ok(())
            }
            (Bootstrapping(state), BootstrapFinished {}) => {
                self.status = Bootstrapped {
                    time: meta.time(),
                    stats: state.stats.clone(),
                };
                Ok(())
            }
            (state, action) => Err(format!("invalid action {action:?} for state {state:?}")),
        }
    }
}
