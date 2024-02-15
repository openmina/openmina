use redux::ActionWithMeta;

use super::stream::P2pNetworkKademliaStreamAction;

impl super::P2pNetworkKadState {
    pub fn reducer(
        &mut self,
        action: ActionWithMeta<&super::P2pNetworkKadAction>,
    ) -> Result<(), String> {
        let (action, meta) = action.split();
        match action {
            super::P2pNetworkKadAction::System(action) => {
                self.system_reducer(meta.with_action(action))
            }
            super::P2pNetworkKadAction::Bootstrap(action) => self
                .bootstrap_state_mut()
                .ok_or_else(|| format!("kademlia is not bootstrapping: {action:?}"))
                .and_then(|state| state.reducer(meta.with_action(action))),

            super::P2pNetworkKadAction::Request(
                action @ super::request::P2pNetworkKadRequestAction::New { addr, peer_id, key },
            ) => self
                .create_request(*addr, peer_id.clone(), key.clone())
                .map_err(|_request| format!("kademlia request to {addr} is already in progress"))
                .and_then(|request| request.reducer(meta.with_action(action))),
            super::P2pNetworkKadAction::Request(
                super::request::P2pNetworkKadRequestAction::Prune { addr },
            ) => self
                .requests
                .remove(addr)
                .map(|_| ())
                .ok_or_else(|| format!("kademlia request for {addr} is not found")),
            super::P2pNetworkKadAction::Request(action) => self
                .requests
                .get_mut(action.addr())
                .ok_or_else(|| format!("kademlia request for {} is not found", action.addr()))
                .and_then(|request| request.reducer(meta.with_action(action))),

            super::P2pNetworkKadAction::Stream(
                action @ P2pNetworkKademliaStreamAction::New { .. },
            ) => self
                .create_kad_stream_state(action.peer_id(), action.stream_id())
                .map_err(|stream| {
                    format!("kademlia stream already exists for action {action:?}: {stream:?}")
                })
                .and_then(|stream| stream.reducer(meta.with_action(action))),
            super::P2pNetworkKadAction::Stream(
                action @ P2pNetworkKademliaStreamAction::Prune { .. },
            ) => self
                .remove_kad_stream_state(action.peer_id(), action.stream_id())
                .then_some(())
                .ok_or_else(|| format!("kademlia stream not found for action {action:?}")),
            super::P2pNetworkKadAction::Stream(action) => self
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
                self.routing_table.extend(closest_peers.iter().cloned());
                Ok(())
            }
            (_, StartBootstrap { key }) => {
                let queue = self
                    .routing_table
                    .closest()
                    .map(|entry| (&entry.peer_id, &entry.addrs));
                self.status = Bootstrapping(super::bootstrap::P2pNetworkKadBootstrapState::new(
                    key.clone(),
                    queue,
                ));
                Ok(())
            }
            (_, BootstrapFinished {}) => {
                self.status = Bootstrapped { time: meta.time() };
                Ok(())
            }
        }
    }
}
