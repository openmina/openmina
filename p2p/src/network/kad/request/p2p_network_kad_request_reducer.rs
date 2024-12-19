use openmina_core::{bug_condition, requests::RpcId, Substate, SubstateAccess};
use redux::{ActionWithMeta, Dispatcher};

use crate::{
    connection::outgoing::P2pConnectionOutgoingAction, ConnectionAddr,
    P2pNetworkConnectionMuxState, P2pNetworkKadBootstrapAction, P2pNetworkKadEffectfulAction,
    P2pNetworkKadState, P2pNetworkKademliaRpcRequest, P2pNetworkKademliaStreamAction,
    P2pNetworkYamuxAction, P2pPeerState, P2pState, PeerId,
};

use super::{P2pNetworkKadRequestAction, P2pNetworkKadRequestState, P2pNetworkKadRequestStatus};

impl P2pNetworkKadRequestState {
    pub fn reducer<State, Action>(
        mut state_context: Substate<Action, State, P2pNetworkKadState>,
        action: ActionWithMeta<P2pNetworkKadRequestAction>,
    ) -> Result<(), String>
    where
        State: SubstateAccess<P2pNetworkKadState> + SubstateAccess<P2pState>,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, _meta) = action.split();
        let state = state_context.get_substate_mut()?;
        let filter_local_addrs = state.filter_addrs;

        let request_state = match action {
            P2pNetworkKadRequestAction::New { peer_id, addr, key } => state
                .create_request(addr, peer_id, key)
                .map_err(|_request| format!("kademlia request to {addr} is already in progress"))?,
            P2pNetworkKadRequestAction::Prune { peer_id } => {
                return state
                    .requests
                    .remove(&peer_id)
                    .map(|_| ())
                    .ok_or_else(|| "kademlia request for {peer_id} is not found".to_owned());
            }
            _ => state
                .requests
                .get_mut(action.peer_id())
                .ok_or_else(|| format!("kademlia request for {} is not found", action.peer_id()))?,
        };

        match action {
            P2pNetworkKadRequestAction::New { peer_id, addr, .. } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;
                let peer_state = p2p_state.peers.get(&peer_id);

                let on_initialize_connection = |dispatcher: &mut Dispatcher<Action, State>| {
                    let opts = crate::connection::outgoing::P2pConnectionOutgoingInitOpts::LibP2P(
                        (peer_id, addr).into(),
                    );
                    let callback = redux::callback!(
                        on_p2p_connection_outgoing_kad_connection_success((peer_id: PeerId, _rpc_id: Option<RpcId>)) -> crate::P2pAction {
                            P2pNetworkKadRequestAction::PeerIsConnecting { peer_id }
                        }
                    );
                    dispatcher.push(P2pConnectionOutgoingAction::Init {
                        opts,
                        rpc_id: None,
                        on_success: Some(callback),
                    });
                    Ok(())
                };

                let on_connection_in_progress = |dispatcher: &mut Dispatcher<Action, State>| {
                    dispatcher.push(P2pNetworkKadRequestAction::PeerIsConnecting { peer_id });
                    Ok(())
                };

                let on_connection_established = |dispatcher: &mut Dispatcher<Action, State>| {
                    let Some((_, conn_state)) = p2p_state.network.scheduler.find_peer(&peer_id)
                    else {
                        bug_condition!(
                            "peer {peer_id} is connected, its network connection is {:?}",
                            p2p_state
                                .network
                                .scheduler
                                .find_peer(&peer_id)
                                .map(|(_, s)| s)
                        );

                        return Ok(());
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
                        // TODO: add callbacks
                        dispatcher.push(P2pNetworkYamuxAction::OpenStream {
                            addr: crate::ConnectionAddr {
                                sock_addr: addr,
                                incoming: false,
                            },
                            stream_id,
                            stream_kind: crate::token::StreamKind::Discovery(
                                crate::token::DiscoveryAlgorithm::Kademlia1_0_0,
                            ),
                        });
                        dispatcher.push(P2pNetworkKadRequestAction::StreamIsCreating {
                            peer_id,
                            stream_id,
                        });
                    } else {
                        // connection is in progress, so wait for multiplexing to be ready
                        dispatcher.push(P2pNetworkKadRequestAction::PeerIsConnecting { peer_id });
                    }
                    Ok(())
                };

                match peer_state {
                    None => on_initialize_connection(dispatcher),
                    Some(P2pPeerState { status, .. }) if !status.is_connected_or_connecting() => {
                        on_initialize_connection(dispatcher)
                    }
                    Some(P2pPeerState { status, .. }) if status.as_ready().is_none() => {
                        on_connection_in_progress(dispatcher)
                    }
                    Some(P2pPeerState { status, .. }) if status.as_ready().is_some() => {
                        on_connection_established(dispatcher)
                    }
                    _ => {
                        bug_condition!("state must be either ready or not ready, peer {peer_id}");
                        Ok(())
                    }
                }
            }
            P2pNetworkKadRequestAction::PeerIsConnecting { .. } => {
                request_state.status = P2pNetworkKadRequestStatus::WaitingForConnection;
                Ok(())
            }
            P2pNetworkKadRequestAction::MuxReady { peer_id, addr } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;

                let stream_id = p2p_state
                    .network
                    .scheduler
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

                // TODO: add callbacks
                dispatcher.push(P2pNetworkYamuxAction::OpenStream {
                    addr,
                    stream_id,
                    stream_kind: crate::token::StreamKind::Discovery(
                        crate::token::DiscoveryAlgorithm::Kademlia1_0_0,
                    ),
                });
                dispatcher
                    .push(P2pNetworkKadRequestAction::StreamIsCreating { peer_id, stream_id });
                Ok(())
            }
            P2pNetworkKadRequestAction::StreamIsCreating { stream_id, .. } => {
                request_state.status = P2pNetworkKadRequestStatus::WaitingForKadStream(stream_id);

                Ok(())
            }
            P2pNetworkKadRequestAction::StreamReady {
                peer_id,
                stream_id,
                addr,
                callback,
            } => {
                let find_node = match P2pNetworkKademliaRpcRequest::find_node(request_state.key) {
                    Ok(find_node) => find_node,
                    Err(error) => {
                        bug_condition!(
                            "P2pNetworkKadRequestAction::StreamReady invalid request key error: {error}"
                        );
                        return Ok(());
                    }
                };

                let message = super::super::Message::from(&find_node);
                request_state.status = quick_protobuf::serialize_into_vec(&message).map_or_else(
                    |e| {
                        super::P2pNetworkKadRequestStatus::Error(format!(
                            "error serializing message: {e}"
                        ))
                    },
                    super::P2pNetworkKadRequestStatus::Request,
                );

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push_callback(callback, (addr, peer_id, stream_id, find_node));
                Ok(())
            }
            P2pNetworkKadRequestAction::RequestSent { .. } => {
                request_state.status = P2pNetworkKadRequestStatus::WaitingForReply;
                Ok(())
            }
            P2pNetworkKadRequestAction::ReplyReceived {
                peer_id,
                stream_id,
                data,
            } => {
                request_state.status = P2pNetworkKadRequestStatus::Reply(data.clone());
                let addr = request_state.addr;

                let bootstrap_request = state
                    .bootstrap_state()
                    .and_then(|bootstrap_state| bootstrap_state.request(&peer_id))
                    .is_some();

                let closest_peers = bootstrap_request
                    .then(|| state.latest_request_peers.clone())
                    .unwrap_or_default();

                let dispatcher = state_context.into_dispatcher();

                if bootstrap_request {
                    dispatcher.push(P2pNetworkKadBootstrapAction::RequestDone {
                        peer_id,
                        closest_peers,
                    });
                }

                for entry in data {
                    let peer_id = entry.peer_id;

                    for multiaddr in entry.addresses().iter() {
                        let multiaddr = multiaddr.clone();
                        dispatcher.push(P2pNetworkKadEffectfulAction::Discovered {
                            multiaddr,
                            filter_local: filter_local_addrs,
                            peer_id,
                        });
                    }
                }
                dispatcher.push(P2pNetworkKademliaStreamAction::Close {
                    addr: ConnectionAddr {
                        sock_addr: addr,
                        incoming: false,
                    },
                    peer_id,
                    stream_id,
                });
                dispatcher.push(P2pNetworkKadRequestAction::Prune { peer_id });
                Ok(())
            }
            P2pNetworkKadRequestAction::Prune { .. } => {
                bug_condition!("Handled above shouldn't happen");
                Ok(())
            }
            P2pNetworkKadRequestAction::Error { peer_id, error } => {
                request_state.status = P2pNetworkKadRequestStatus::Error(error.clone());
                let bootstrap_request = state
                    .bootstrap_state()
                    .and_then(|bootstrap_state| bootstrap_state.request(&peer_id))
                    .is_some();

                let dispatcher = state_context.into_dispatcher();

                if bootstrap_request {
                    dispatcher.push(P2pNetworkKadBootstrapAction::RequestError { peer_id, error });
                }

                dispatcher.push(P2pNetworkKadRequestAction::Prune { peer_id });
                Ok(())
            }
        }
    }
}
