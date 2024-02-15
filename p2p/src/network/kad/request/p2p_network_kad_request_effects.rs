use std::net::SocketAddr;

use redux::ActionMeta;

use crate::{
    connection::outgoing::{P2pConnectionOutgoingAction, P2pConnectionOutgoingInitOpts},
    discovery::P2pDiscoveryAction,
    socket_addr_try_from_multiaddr, P2pNetworkConnectionMuxState, P2pNetworkYamuxOpenStreamAction,
};

use super::{super::stream::P2pNetworkKademliaStreamAction, P2pNetworkKadRequestAction};

impl P2pNetworkKadRequestAction {
    pub fn effects<Store, S>(self, _meta: &ActionMeta, store: &mut Store) -> Result<(), String>
    where
        Store: crate::P2pStore<S>,
    {
        let scheduler = &store.state().network.scheduler;
        let discovery_state = scheduler
            .discovery_state()
            .ok_or_else(|| String::from("discovery is not configured"))?;
        let addr = self.addr();
        let (_, request_state) = discovery_state
            .bootstrap_state()
            .ok_or_else(|| String::from("not in bootstrapping"))?
            .request(addr)
            .ok_or_else(|| format!("no ongoing request for {addr}"))?;

        use P2pNetworkKadRequestAction as A;

        match self {
            A::New { addr, .. } => {
                if let Some(conn_state) = scheduler.connections.get(&addr) {
                    // TODO: check if the connection is terminated with error...
                    if let Some(stream_id) = conn_state.mux.as_ref().and_then(
                        |P2pNetworkConnectionMuxState::Yamux(yamux)| yamux.next_stream_id(),
                    ) {
                        // multiplexing is ready, open a stream
                        store.dispatch(P2pNetworkYamuxOpenStreamAction {
                            addr,
                            stream_id,
                            stream_kind: crate::token::StreamKind::Discovery(
                                crate::token::DiscoveryAlgorithm::Kademlia1_0_0,
                            ),
                        });
                        store.dispatch(A::StreamIsCreating { addr, stream_id });
                    } else {
                        // connection is in progress, so wait for multiplexing to be ready
                        store.dispatch(A::PeerIsConnecting { addr });
                    }
                } else {
                    // initialize connection to the peer.
                    // when connection is establised and yamux layer is ready, we will continue with TODO
                    let opts = crate::connection::outgoing::P2pConnectionOutgoingInitOpts::LibP2P(
                        (request_state.peer_id.clone(), addr).into(),
                    );
                    store.dispatch(P2pConnectionOutgoingAction::Init { opts, rpc_id: None });
                    store.dispatch(A::PeerIsConnecting { addr });
                }
            }
            A::PeerIsConnecting { .. } => {}
            A::MuxReady { addr } => {
                // connection's multiplexing is initialized, we need to create a stream
                let stream_id = scheduler
                    .connections
                    .get(&addr)
                    .ok_or_else(|| format!("connection with {addr} not found"))
                    .and_then(|conn| {
                        conn.mux
                            .as_ref()
                            .ok_or_else(|| format!("multiplexing is not ready for {addr}"))
                    })
                    .and_then(|P2pNetworkConnectionMuxState::Yamux(yamux)| {
                        yamux
                            .next_stream_id()
                            .ok_or_else(|| format!("cannot get next stream for {addr}"))
                    })?;
                store.dispatch(P2pNetworkYamuxOpenStreamAction {
                    addr,
                    stream_id,
                    stream_kind: crate::token::StreamKind::Discovery(
                        crate::token::DiscoveryAlgorithm::Kademlia1_0_0,
                    ),
                });
                store.dispatch(A::StreamIsCreating { addr, stream_id });
            }
            A::StreamIsCreating { .. } => {}
            A::StreamReady { addr, stream_id } => {
                let data = crate::P2pNetworkKademliaRpcRequest::find_node(
                    store.state().config.identity_pub_key.peer_id(),
                );
                store.dispatch(P2pNetworkKademliaStreamAction::SendRequest {
                    addr,
                    peer_id: request_state.peer_id.clone(),
                    stream_id,
                    data,
                });
                store.dispatch(A::RequestSent { addr });
            }
            A::RequestSent { .. } => {}
            A::ReplyReceived { data, .. } => {
                let external_addr = |addr: &SocketAddr| match addr.ip() {
                    std::net::IpAddr::V4(v) => !(v.is_loopback() || v.is_private()),
                    std::net::IpAddr::V6(v) => !(v.is_loopback()),
                };
                for entry in data {
                    let peer_id = entry.peer_id;
                    let to_opts = |addr| (peer_id.clone(), addr).into();
                    let addresses = entry
                        .addrs
                        .iter()
                        .map(socket_addr_try_from_multiaddr)
                        .filter_map(Result::ok)
                        .filter(external_addr)
                        .map(to_opts)
                        .map(P2pConnectionOutgoingInitOpts::LibP2P)
                        .collect();
                    store.dispatch(P2pDiscoveryAction::KademliaAddRoute { peer_id, addresses });
                }
            }
            A::Prune { .. } => {}
            A::Error { .. } => {}
        }
        Ok(())
    }
}
