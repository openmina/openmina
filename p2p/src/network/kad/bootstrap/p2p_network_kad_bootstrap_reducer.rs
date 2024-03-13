use std::collections::BTreeSet;

use redux::{ActionWithMeta, Timestamp};

use crate::{
    bootstrap::{
        P2pNetworkKadBoostrapRequestState, P2pNetworkKadBootstrapOngoingRequest,
        P2pNetworkKadBootstrapRequestStat, P2pNetworkKadBootstrapSuccessfullRequest,
    },
    connection::outgoing::P2pConnectionOutgoingInitOpts,
    socket_addr_try_from_multiaddr, P2pNetworkKadEntry, P2pNetworkKadKey,
    P2pNetworkKadRoutingTable, PeerId,
};

use super::{P2pNetworkKadBootstrapAction, P2pNetworkKadBootstrapState};

impl P2pNetworkKadBootstrapState {
    pub fn reducer(
        &mut self,
        routing_table: &P2pNetworkKadRoutingTable,
        action: ActionWithMeta<&P2pNetworkKadBootstrapAction>,
    ) -> Result<(), String> {
        let (action, meta) = action.split();
        use P2pNetworkKadBootstrapAction as A;

        match action {
            A::CreateRequests {} => {
                let mut closest_peers =
                    Self::closest_peers(&routing_table, &self.kademlia_key, &self.processed_peers);
                while self.requests.len() < 3 {
                    let Some(entry) = closest_peers.next() else {
                        break;
                    };
                    let Some(next_request) = Self::prepare_next_request(entry, meta.time()) else {
                        continue;
                    };
                    let address = P2pConnectionOutgoingInitOpts::LibP2P(
                        (next_request.peer_id.clone(), next_request.addr).into(),
                    );
                    self.stats
                        .requests
                        .push(P2pNetworkKadBootstrapRequestStat::Ongoing(
                            P2pNetworkKadBootstrapOngoingRequest {
                                address,
                                start: meta.time(),
                            },
                        ));
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
                let address = P2pConnectionOutgoingInitOpts::LibP2P((req.peer_id, *addr).into());

                if let Some(request_stats) = self.stats.requests.iter_mut().rev().find(|req_stat| {
                    matches!(
                        req_stat,
                        P2pNetworkKadBootstrapRequestStat::Ongoing(
                            P2pNetworkKadBootstrapOngoingRequest {
                                address: a,
                                ..
                            },
                        ) if a == &address
                    )
                }) {
                    *request_stats = P2pNetworkKadBootstrapRequestStat::Successfull(
                        P2pNetworkKadBootstrapSuccessfullRequest {
                            address,
                            start: req.time,
                            finish: meta.time(),
                            closest_peers: closest_peers.clone(),
                        },
                    );
                } else {
                    return Err(format!("cannot find stats for request {req:?}"));
                }

                Ok(())
            }
        }
    }

    fn closest_peers<'a>(
        routing_table: &'a P2pNetworkKadRoutingTable,
        key: &'a P2pNetworkKadKey,
        processed_peers: &'a BTreeSet<PeerId>,
    ) -> impl Iterator<Item = &'a P2pNetworkKadEntry> {
        routing_table
            .find_node(key)
            .filter(|entry| !processed_peers.contains(&entry.peer_id))
    }

    fn prepare_next_request(
        P2pNetworkKadEntry { peer_id, addrs, .. }: &P2pNetworkKadEntry,
        time: Timestamp,
    ) -> Option<P2pNetworkKadBoostrapRequestState> {
        let mut addrs = addrs
            .iter()
            .map(socket_addr_try_from_multiaddr)
            .filter_map(Result::ok);
        let Some(addr) = addrs.next() else {
            return None;
        };
        let addrs_to_use = addrs.collect();
        Some(P2pNetworkKadBoostrapRequestState {
            peer_id: peer_id.clone(),
            addr,
            time,
            addrs_to_use,
        })
    }
}
