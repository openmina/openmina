use redux::{ActionWithMeta, Timestamp};

use crate::{bootstrap::P2pNetworkKadBoostrapRequestState, socket_addr_try_from_multiaddr};

use super::{P2pNetworkKadBootstrapAction, P2pNetworkKadBootstrapState};

impl P2pNetworkKadBootstrapState {
    pub fn reducer(
        &mut self,
        action: ActionWithMeta<&P2pNetworkKadBootstrapAction>,
    ) -> Result<(), String> {
        let (action, _meta) = action.split();
        use P2pNetworkKadBootstrapAction as A;

        match action {
            A::CreateRequests {} => {
                while self.requests.len() < 3 {
                    let Some(next_request) = self.prepare_next_request(_meta.time())? else {
                        // No more peers in the queue
                        return Ok(());
                    };
                    self.requests.push(next_request);
                }
                Ok(())
            }
            A::RequestDone {
                addr,
                closest_peers,
            } => {
                let Some((i, _)) = self.request(addr) else {
                    return Err(format!("cannot find reques with address {addr}"));
                };
                let req = self.requests.swap_remove(i);
                self.processed_peers.insert(req.peer_id);
                let new_peers = closest_peers.iter().filter_map(|entry| {
                    if self.processed_peers.contains(&entry.peer_id) {
                        return None;
                    }
                    if entry.addrs.is_empty() {
                        return None;
                    };
                    Some((entry.peer_id.clone(), entry.addrs.clone()))
                });
                self.queue.extend(new_peers);
                Ok(())
            }
        }
    }

    fn prepare_next_request(
        &mut self,
        time: Timestamp,
    ) -> Result<Option<P2pNetworkKadBoostrapRequestState>, String> {
        while let Some((peer_id, addrs)) = self.queue.pop_front() {
            if self.processed_peers.contains(&peer_id) {
                continue;
            }
            let mut addrs = addrs
                .iter()
                .map(socket_addr_try_from_multiaddr)
                .filter_map(Result::ok);
            let Some(addr) = addrs.next() else {
                continue;
            };
            let addrs_to_use = addrs.collect();
            return Ok(Some(P2pNetworkKadBoostrapRequestState {
                peer_id,
                addr,
                time,
                addrs_to_use,
            }));
        }
        Ok(None)
    }
}
