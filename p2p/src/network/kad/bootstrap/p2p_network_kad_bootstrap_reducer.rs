use multiaddr::Multiaddr;
use redux::{ActionWithMeta, Timestamp};

use crate::{
    bootstrap::{
        P2pNetworkKadBoostrapRequestState, P2pNetworkKadBootstrapFailedRequest,
        P2pNetworkKadBootstrapOngoingRequest, P2pNetworkKadBootstrapRequestStat,
        P2pNetworkKadBootstrapSuccessfullRequest,
    },
    connection::outgoing::P2pConnectionOutgoingInitOpts,
    socket_addr_try_from_multiaddr, P2pNetworkKadEntry, P2pNetworkKadRoutingTable,
};

use super::{P2pNetworkKadBootstrapAction, P2pNetworkKadBootstrapState};

fn prepare_next_request(
    addrs: &Vec<Multiaddr>,
    time: Timestamp,
) -> Option<P2pNetworkKadBoostrapRequestState> {
    let mut addrs = addrs
        .iter()
        .map(socket_addr_try_from_multiaddr)
        .filter_map(Result::ok)
        // TODO(akoptelov): remove this filtering when multiple address support is added
        .filter(|addr| match addr.ip() {
            std::net::IpAddr::V4(ipv4) if ipv4.is_loopback() || ipv4.is_private() => false,
            std::net::IpAddr::V6(ipv6) if ipv6.is_loopback() => false,
            _ => true,
        });

    let Some(addr) = addrs.next() else {
        return None;
    };
    let addrs_to_use = addrs.collect();
    Some(P2pNetworkKadBoostrapRequestState {
        addr,
        time,
        addrs_to_use,
    })
}

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
                let requests_to_create = 3_usize.saturating_sub(self.requests.len());
                let peer_id_req_vec = routing_table
                    .closest_peers(&self.kademlia_key) // for the next request we take closest peer
                    .filter(|entry| !self.processed_peers.contains(&entry.peer_id)) // that is not yet processed during this bootstrap
                    .filter_map(|P2pNetworkKadEntry { peer_id, addrs, .. }| {
                        // we create a request for it
                        prepare_next_request(addrs, meta.time()).map(|req| (*peer_id, req))
                    })
                    .take(requests_to_create) // and stop when we create enough requests so up to 3 will be executed in parallel
                    .collect::<Vec<_>>();

                for (peer_id, request) in peer_id_req_vec {
                    self.processed_peers.insert(peer_id);
                    let address =
                        P2pConnectionOutgoingInitOpts::LibP2P((peer_id, request.addr).into());
                    self.stats
                        .requests
                        .push(P2pNetworkKadBootstrapRequestStat::Ongoing(
                            P2pNetworkKadBootstrapOngoingRequest {
                                peer_id,
                                address,
                                start: meta.time(),
                            },
                        ));
                    self.requests.insert(peer_id, request);
                }
                Ok(())
            }
            A::RequestDone {
                peer_id,
                closest_peers,
            } => {
                let Some(req) = self.requests.remove(&peer_id) else {
                    return Err(format!("cannot find reques for peer {peer_id}"));
                };
                self.successfull_requests += 1;
                let address = P2pConnectionOutgoingInitOpts::LibP2P((*peer_id, req.addr).into());

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
                            peer_id: *peer_id,
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
            A::RequestError { peer_id, error } => {
                let Some(req) = self.requests.remove(&peer_id) else {
                    return Err(format!("cannot find reques for peer {peer_id}"));
                };
                let address = P2pConnectionOutgoingInitOpts::LibP2P((*peer_id, req.addr).into());

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
                    *request_stats = P2pNetworkKadBootstrapRequestStat::Failed(
                        P2pNetworkKadBootstrapFailedRequest {
                            peer_id: *peer_id,
                            address,
                            start: req.time,
                            finish: meta.time(),
                            error: error.clone(),
                        },
                    );
                } else {
                    return Err(format!("cannot find stats for request {req:?}"));
                }

                Ok(())
            }
        }
    }
}
