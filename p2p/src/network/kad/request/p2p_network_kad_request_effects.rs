use std::net::SocketAddr;

use redux::ActionMeta;

use crate::{
    connection::outgoing::{P2pConnectionOutgoingAction, P2pConnectionOutgoingInitOpts},
    peer::P2pPeerAction,
    socket_addr_try_from_multiaddr, P2pNetworkConnectionMuxState, P2pNetworkKadBootstrapAction,
    P2pNetworkYamuxAction, P2pPeerState,
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
        if let A::Prune { .. } = &self {
            return Ok(());
        }
        let peer_id = self.peer_id();
        let Some(request_state) = discovery_state.request(peer_id) else {
            return Err(format!("no request for {peer_id}"));
        };

        use P2pNetworkKadRequestAction as A;

        match self {
            A::New { peer_id, addr, .. } => {
                let peer_state = store.state().peers.get(&peer_id);

                let on_initialize_connection = |store: &mut Store| {
                    // initialize connection to the peer.
                    // when connection is establised and yamux layer is ready, we will continue with TODO
                    let opts = crate::connection::outgoing::P2pConnectionOutgoingInitOpts::LibP2P(
                        (peer_id, addr).into(),
                    );
                    store.dispatch(P2pConnectionOutgoingAction::Init { opts, rpc_id: None });
                    store.dispatch(A::PeerIsConnecting { peer_id });
                    Ok(())
                };

                let on_connection_in_progress = |store: &mut Store| {
                    store.dispatch(A::PeerIsConnecting { peer_id });
                    Ok(())
                };

                let on_connection_established = |store: &mut Store| {
                    let Some((_, conn_state)) = store.state().network.scheduler.find_peer(&peer_id)
                    else {
                        return Err(format!(
                            "peer {peer_id} is connected, its network connection is {:?}",
                            store
                                .state()
                                .network
                                .scheduler
                                .find_peer(&peer_id)
                                .map(|(_, s)| s)
                        ));
                    };
                    if let Some(stream_id) = conn_state.mux.as_ref().and_then(
                        |P2pNetworkConnectionMuxState::Yamux(yamux)| {
                            yamux.next_stream_id(
                                crate::YamuxStreamKind::Kademlia,
                                conn_state.incoming,
                            )
                        },
                    ) {
                        // multiplexing is ready, open a stream
                        store.dispatch(P2pNetworkYamuxAction::OpenStream {
                            addr,
                            stream_id,
                            stream_kind: crate::token::StreamKind::Discovery(
                                crate::token::DiscoveryAlgorithm::Kademlia1_0_0,
                            ),
                        });
                        store.dispatch(A::StreamIsCreating { peer_id, stream_id });
                    } else {
                        // connection is in progress, so wait for multiplexing to be ready
                        store.dispatch(A::PeerIsConnecting { peer_id });
                    }
                    Ok(())
                };

                return match peer_state {
                    None => on_initialize_connection(store),
                    Some(P2pPeerState { status, .. }) if !status.is_connected_or_connecting() => {
                        on_initialize_connection(store)
                    }
                    Some(P2pPeerState { status, .. }) if status.as_ready().is_none() => {
                        on_connection_in_progress(store)
                    }
                    _ => on_connection_established(store),
                };
            }
            A::PeerIsConnecting { .. } => {}
            A::MuxReady { peer_id, addr } => {
                // connection's multiplexing is initialized, we need to create a stream

                let stream_id = scheduler
                    .connections
                    .get(&addr)
                    .ok_or_else(|| format!("connection with {addr} not found"))
                    .and_then(|conn| {
                        conn.mux
                            .as_ref()
                            .map(|mux| (mux, conn.incoming))
                            .ok_or_else(|| format!("multiplexing is not ready for {addr}"))
                    })
                    .and_then(|(P2pNetworkConnectionMuxState::Yamux(yamux), incoming)| {
                        yamux
                            .next_stream_id(crate::YamuxStreamKind::Kademlia, incoming)
                            .ok_or_else(|| format!("cannot get next stream for {addr}"))
                    })?;
                store.dispatch(P2pNetworkYamuxAction::OpenStream {
                    addr,
                    stream_id,
                    stream_kind: crate::token::StreamKind::Discovery(
                        crate::token::DiscoveryAlgorithm::Kademlia1_0_0,
                    ),
                });
                store.dispatch(A::StreamIsCreating { peer_id, stream_id });
            }
            A::StreamIsCreating { .. } => {}
            A::StreamReady {
                peer_id,
                stream_id,
                addr,
            } => {
                let data = crate::P2pNetworkKademliaRpcRequest::find_node(request_state.key);
                store.dispatch(P2pNetworkKademliaStreamAction::SendRequest {
                    addr,
                    peer_id,
                    stream_id,
                    data,
                });
                store.dispatch(A::RequestSent { peer_id });
            }
            A::RequestSent { .. } => {}
            A::ReplyReceived {
                data,
                peer_id,
                stream_id,
            } => {
                let addr = request_state.addr;
                let bootstrap_request = discovery_state
                    .bootstrap_state()
                    .and_then(|bootstrap_state| bootstrap_state.request(&peer_id))
                    .is_some();
                let closest_peers = bootstrap_request
                    .then(|| discovery_state.latest_request_peers.clone())
                    .unwrap_or_default();
                if bootstrap_request {
                    store.dispatch(P2pNetworkKadBootstrapAction::RequestDone {
                        peer_id,
                        closest_peers,
                    });
                }

                let external_addr = |addr: &SocketAddr| match addr.ip() {
                    std::net::IpAddr::V4(v) => !(v.is_loopback() || v.is_private()),
                    std::net::IpAddr::V6(v) => !(v.is_loopback()),
                };
                for entry in data {
                    let peer_id = entry.peer_id;
                    let to_opts = |addr| (peer_id, addr).into();
                    let mut addresses = entry
                        .addrs
                        .iter()
                        .map(socket_addr_try_from_multiaddr)
                        .filter_map(Result::ok)
                        .filter(external_addr)
                        .map(to_opts)
                        .map(P2pConnectionOutgoingInitOpts::LibP2P);
                    // TODO: use all addresses
                    store.dispatch(P2pPeerAction::Discovered {
                        peer_id,
                        dial_opts: addresses.next(),
                    });
                }
                store.dispatch(P2pNetworkKademliaStreamAction::Close {
                    addr,
                    peer_id,
                    stream_id,
                });
                store.dispatch(P2pNetworkKadRequestAction::Prune { peer_id });
            }
            A::Error { peer_id, error } => {
                let bootstrap_request = discovery_state
                    .bootstrap_state()
                    .and_then(|bootstrap_state| bootstrap_state.request(&peer_id))
                    .is_some();
                if bootstrap_request {
                    store.dispatch(P2pNetworkKadBootstrapAction::RequestError { peer_id, error });
                }
                store.dispatch(P2pNetworkKadRequestAction::Prune { peer_id });
            }
            A::Prune { .. } => {}
        }
        Ok(())
    }
}
