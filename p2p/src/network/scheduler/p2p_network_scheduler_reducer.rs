use std::{collections::BTreeMap, sync::OnceLock};

use identify::P2pNetworkIdentifyStreamAction;
use openmina_core::{bug_condition, error, warn, Substate};
use request::{P2pNetworkKadRequestState, P2pNetworkKadRequestStatus};
use token::{
    AuthKind, DiscoveryAlgorithm, IdentifyAlgorithm, MuxKind, PingAlgorithm, Protocol,
    RpcAlgorithm, StreamKind,
};

use crate::{
    connection::{
        incoming::P2pConnectionIncomingAction, outgoing::P2pConnectionOutgoingAction,
        P2pConnectionState,
    },
    disconnection::P2pDisconnectionAction,
    identify::P2pIdentifyAction,
    P2pConfig, P2pPeerStatus, P2pState,
};

use super::{super::*, p2p_network_scheduler_state::P2pNetworkConnectionState, *};

impl P2pNetworkSchedulerState {
    pub fn reducer<Action, State>(
        mut state_context: Substate<Action, State, Self>,
        action: redux::ActionWithMeta<&P2pNetworkSchedulerAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();
        let scheduler_state = state_context.get_substate_mut()?;

        match action {
            P2pNetworkSchedulerAction::InterfaceDetected { ip, .. } => {
                scheduler_state.interfaces.insert(*ip);

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_config: &P2pConfig = state.substate()?;

                if let Some(port) = p2p_config.libp2p_port {
                    dispatcher.push(P2pNetworkSchedulerEffectfulAction::InterfaceDetected {
                        ip: *ip,
                        port,
                    });
                }

                Ok(())
            }
            P2pNetworkSchedulerAction::InterfaceExpired { ip, .. } => {
                scheduler_state.interfaces.remove(ip);
                Ok(())
            }
            P2pNetworkSchedulerAction::ListenerReady { listener } => {
                scheduler_state.listeners.insert(*listener);
                Ok(())
            }
            P2pNetworkSchedulerAction::ListenerError { listener, .. } => {
                scheduler_state.listeners.remove(listener);
                Ok(())
            }
            P2pNetworkSchedulerAction::IncomingDataIsReady { addr } => {
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let scheduler: &Self = state.substate()?;
                let Some(connection_state) = scheduler.connection_state(addr) else {
                    bug_condition!(
                        "Invalid state for `P2pNetworkSchedulerAction::IncomingDataIsReady`"
                    );
                    return Ok(());
                };

                let limit = connection_state.limit();
                if limit > 0 {
                    dispatcher.push(P2pNetworkSchedulerEffectfulAction::IncomingDataIsReady {
                        addr: *addr,
                        limit,
                    });
                }

                Ok(())
            }
            P2pNetworkSchedulerAction::IncomingDidAccept { addr, result } => {
                if let Some(addr) = addr {
                    scheduler_state.connections.insert(
                        *addr,
                        P2pNetworkConnectionState {
                            incoming: true,
                            pnet: P2pNetworkPnetState::new(scheduler_state.pnet_key, meta.time()),
                            select_auth: P2pNetworkSelectState::default(),
                            auth: None,
                            select_mux: P2pNetworkSelectState::default(),
                            mux: None,
                            streams: BTreeMap::default(),
                            closed: None,
                            limit: P2pNetworkConnectionState::INITIAL_LIMIT,
                        },
                    );
                };

                let dispatcher = state_context.into_dispatcher();
                if let Some(addr) = addr {
                    dispatcher.push(P2pNetworkSchedulerEffectfulAction::IncomingDidAccept {
                        addr: *addr,
                        result: result.clone(),
                    });
                }

                Ok(())
            }
            P2pNetworkSchedulerAction::OutgoingConnect { addr } => {
                scheduler_state.connections.insert(
                    ConnectionAddr {
                        sock_addr: *addr,
                        incoming: false,
                    },
                    P2pNetworkConnectionState {
                        incoming: false,
                        pnet: P2pNetworkPnetState::new(scheduler_state.pnet_key, meta.time()),
                        select_auth: P2pNetworkSelectState::initiator_auth(
                            token::AuthKind::Noise,
                            meta.time(),
                        ),
                        auth: None,
                        select_mux: P2pNetworkSelectState::initiator_mux(
                            token::MuxKind::Yamux1_0_0,
                            meta.time(),
                        ),
                        mux: None,
                        streams: BTreeMap::default(),
                        closed: None,
                        limit: P2pNetworkConnectionState::INITIAL_LIMIT,
                    },
                );

                let dispatcher = state_context.into_dispatcher();
                dispatcher
                    .push(P2pNetworkSchedulerEffectfulAction::OutgoingConnect { addr: *addr });
                Ok(())
            }
            P2pNetworkSchedulerAction::OutgoingDidConnect { addr, result } => {
                // TODO: change to connected

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;

                match result {
                    Ok(()) => {
                        dispatcher.push(P2pNetworkSchedulerEffectfulAction::OutgoingDidConnect {
                            addr: *addr,
                        });
                    }
                    Err(error) => {
                        let Some((peer_id, peer_state)) = p2p_state.peer_with_connection(*addr)
                        else {
                            bug_condition!(
                                "outgoing connection to {addr} failed, but there is no peer for it"
                            );
                            return Ok(());
                        };
                        if matches!(
                            peer_state.status,
                            P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(_))
                        ) {
                            dispatcher.push(P2pConnectionOutgoingAction::FinalizeError {
                                peer_id,
                                error: error.to_string(),
                            });
                        }
                    }
                }
                Ok(())
            }
            P2pNetworkSchedulerAction::IncomingDataDidReceive { result, addr } => {
                if let Some(state) = scheduler_state.connections.get_mut(addr) {
                    if let Ok(data) = result {
                        state.consume(data.len());
                    }
                };

                let dispatcher = state_context.into_dispatcher();
                match result {
                    Ok(data) => {
                        dispatcher.push(P2pNetworkPnetAction::IncomingData {
                            addr: *addr,
                            data: data.clone(),
                        });
                    }
                    Err(error) => dispatcher.push(P2pNetworkSchedulerAction::Error {
                        addr: *addr,
                        error: P2pNetworkConnectionError::MioError(error.clone()),
                    }),
                }
                Ok(())
            }
            P2pNetworkSchedulerAction::SelectDone {
                addr,
                kind,
                protocol,
                incoming,
                expected_peer_id,
            } => {
                if let Some(connection) = scheduler_state.connections.get_mut(addr) {
                    match protocol {
                        Some(token::Protocol::Auth(token::AuthKind::Noise)) => {
                            connection.auth =
                                Some(P2pNetworkAuthState::Noise(P2pNetworkNoiseState::new(
                                    scheduler_state.local_pk.clone(),
                                    *expected_peer_id,
                                )));
                        }
                        Some(token::Protocol::Mux(
                            token::MuxKind::Yamux1_0_0 | token::MuxKind::YamuxNoNewLine1_0_0,
                        )) => {
                            connection.mux =
                                Some(P2pNetworkConnectionMuxState::Yamux(P2pNetworkYamuxState {
                                    init: true,
                                    ..Default::default()
                                }));
                        }
                        Some(token::Protocol::Stream(stream_kind)) => {
                            let SelectKind::Stream(peer_id, stream_id) = kind else {
                                error!(meta.time(); "incorrect stream kind {kind:?} for protocol stream: {stream_kind:?}");
                                return Ok(());
                            };
                            match stream_kind {
                                token::StreamKind::Rpc(_) => {
                                    if *incoming {
                                        scheduler_state
                                            .rpc_incoming_streams
                                            .entry(*peer_id)
                                            .or_default()
                                            .insert(
                                                *stream_id,
                                                P2pNetworkRpcState::new(*addr, *stream_id),
                                            );
                                    } else {
                                        scheduler_state
                                            .rpc_outgoing_streams
                                            .entry(*peer_id)
                                            .or_default()
                                            .insert(
                                                *stream_id,
                                                P2pNetworkRpcState::new(*addr, *stream_id),
                                            );
                                    }
                                }
                                token::StreamKind::Broadcast(_) => {}
                                token::StreamKind::Identify(_) => {}
                                token::StreamKind::Discovery(_) => {}
                                token::StreamKind::Ping(_) => {}
                                token::StreamKind::Bitswap(_) => {}
                                token::StreamKind::Status(_) => {}
                            }
                        }
                        None => {}
                    }
                };

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let p2p_state: &P2pState = state.substate()?;
                let select_kind = *kind;
                let addr = *addr;
                let incoming = *incoming;

                match protocol {
                    Some(Protocol::Auth(AuthKind::Noise)) => {
                        dispatcher.push(P2pNetworkSchedulerEffectfulAction::NoiseSelectDone {
                            addr,
                            incoming,
                        });
                    }
                    Some(Protocol::Mux(MuxKind::Yamux1_0_0 | MuxKind::YamuxNoNewLine1_0_0)) => {
                        let SelectKind::Multiplexing(peer_id) = select_kind else {
                            error!(meta.time(); "wrong kind for multiplexing protocol action: {select_kind:?}");
                            return Ok(());
                        };
                        let message_size_limit = p2p_state.config.limits.yamux_message_size();
                        dispatcher.push(P2pNetworkSchedulerAction::YamuxDidInit {
                            addr,
                            peer_id,
                            message_size_limit,
                        });
                    }
                    Some(Protocol::Stream(kind)) => {
                        let SelectKind::Stream(peer_id, stream_id) = select_kind else {
                            error!(meta.time(); "wrong kind for stream protocol action: {kind:?}");
                            return Ok(());
                        };
                        match kind {
                            StreamKind::Status(_)
                            | StreamKind::Identify(IdentifyAlgorithm::IdentifyPush1_0_0)
                            | StreamKind::Bitswap(_)
                            | StreamKind::Ping(PingAlgorithm::Ping1_0_0) => {
                                //unimplemented!()
                            }
                            StreamKind::Identify(IdentifyAlgorithm::Identify1_0_0) => {
                                dispatcher.push(P2pNetworkIdentifyStreamAction::New {
                                    addr,
                                    peer_id,
                                    stream_id,
                                    incoming,
                                });
                            }

                            StreamKind::Broadcast(protocol) => {
                                dispatcher.push(P2pNetworkPubsubAction::NewStream {
                                    incoming,
                                    peer_id,
                                    addr,
                                    stream_id,
                                    protocol: *protocol,
                                });
                            }
                            StreamKind::Discovery(DiscoveryAlgorithm::Kademlia1_0_0) => {
                                if let Some(discovery_state) =
                                    p2p_state.network.scheduler.discovery_state()
                                {
                                    let request =
                                        !incoming && discovery_state.request(&peer_id).is_some();
                                    dispatcher.push(P2pNetworkKademliaStreamAction::New {
                                        addr,
                                        peer_id,
                                        stream_id,
                                        incoming,
                                    });
                                    // if our node initiated a request to the peer, notify that the stream is ready.
                                    if request {
                                        dispatcher.push(P2pNetworkKadRequestAction::StreamReady {
                                            peer_id,
                                            addr,
                                            stream_id,
                                        });
                                    }
                                }
                            }
                            StreamKind::Rpc(RpcAlgorithm::Rpc0_0_1) => {
                                dispatcher.push(P2pNetworkRpcAction::Init {
                                    addr,
                                    peer_id,
                                    stream_id,
                                    incoming,
                                });
                            }
                        }
                    }
                    None => {
                        match &select_kind {
                            SelectKind::Authentication => {
                                // TODO: close the connection
                            }
                            SelectKind::MultiplexingNoPeerId => {
                                // WARNING: must not happen
                            }
                            SelectKind::Multiplexing(_) => {
                                // TODO: close the connection
                            }
                            SelectKind::Stream(peer_id, stream_id) => {
                                if let Some(discovery_state) =
                                    p2p_state.network.scheduler.discovery_state()
                                {
                                    if let Some(P2pNetworkKadRequestState {
                                        status: P2pNetworkKadRequestStatus::WaitingForKadStream(id),
                                        ..
                                    }) = discovery_state.request(peer_id)
                                    {
                                        if id == stream_id {
                                            dispatcher.push(P2pNetworkKadRequestAction::Error {
                                                peer_id: *peer_id,
                                                error: "stream protocol is not negotiated".into(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(())
            }
            P2pNetworkSchedulerAction::SelectError { addr, kind, error } => {
                let dispatcher = state_context.into_dispatcher();

                match kind {
                    SelectKind::Stream(peer_id, stream_id)
                        if keep_connection_with_unknown_stream() =>
                    {
                        warn!(meta.time(); summary="select error for stream", addr = display(addr), peer_id = display(peer_id));
                        // just close the stream
                        dispatcher.push(P2pNetworkYamuxAction::OutgoingData {
                            addr: *addr,
                            stream_id: *stream_id,
                            data: Data::default(),
                            flags: YamuxFlags::RST,
                        });
                        dispatcher.push(P2pNetworkSchedulerAction::PruneStream {
                            peer_id: *peer_id,
                            stream_id: *stream_id,
                        });
                    }
                    _ => {
                        dispatcher.push(P2pNetworkSchedulerAction::Error {
                            addr: *addr,
                            error: P2pNetworkConnectionError::SelectError,
                        });
                    }
                }

                dispatcher.push(P2pNetworkSchedulerEffectfulAction::SelectError {
                    addr: *addr,
                    kind: *kind,
                    error: error.to_owned(),
                });

                Ok(())
            }
            P2pNetworkSchedulerAction::YamuxDidInit {
                addr,
                message_size_limit,
                peer_id,
            } => {
                let Some(cn) = scheduler_state.connections.get_mut(addr) else {
                    return Ok(());
                };
                if let Some(P2pNetworkConnectionMuxState::Yamux(yamux)) = &mut cn.mux {
                    yamux.init = true;
                    yamux.message_size_limit = *message_size_limit;
                }

                let incoming = cn.incoming;
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let peer_id = *peer_id;
                let addr = *addr;

                if incoming {
                    dispatcher.push(P2pConnectionIncomingAction::Libp2pReceived { peer_id });
                } else {
                    dispatcher.push(P2pConnectionOutgoingAction::FinalizeSuccess { peer_id });
                }

                // for each negotiated yamux conenction open a new outgoing RPC stream
                // TODO(akoptelov,vlad): should we do that? shouldn't upper layer decide when to open RPC streams?
                // Also rpc streams are short-living -- they only persist for a single request-response (?)
                let stream_id = YamuxStreamKind::Rpc.stream_id(incoming);
                dispatcher.push(P2pNetworkYamuxAction::OpenStream {
                    addr,
                    stream_id,
                    stream_kind: StreamKind::Rpc(RpcAlgorithm::Rpc0_0_1),
                });
                dispatcher.push(P2pNetworkYamuxAction::OpenStream {
                    addr,
                    stream_id: stream_id + 2,
                    stream_kind: StreamKind::Broadcast(token::BroadcastAlgorithm::Meshsub1_1_0),
                });

                // TODO: open RPC and Kad connections only after identify reports support for it?
                dispatcher.push(P2pIdentifyAction::NewRequest { peer_id, addr });

                // Kademlia: if the connection is initiated by Kademlia request, notify that it is ready.
                let kad_state: Option<&P2pNetworkKadState> = state.substate().ok();

                if kad_state.map_or(false, |state| state.request(&peer_id).is_some()) {
                    dispatcher.push(P2pNetworkKadRequestAction::MuxReady { peer_id, addr });
                }
                Ok(())
            }
            P2pNetworkSchedulerAction::Disconnect { addr, reason } => {
                let Some(conn_state) = scheduler_state.connections.get_mut(addr) else {
                    error!(meta.time(); "P2pNetworkSchedulerAction::Disconnect: connection {addr} does not exist");
                    return Ok(());
                };
                if conn_state.closed.is_some() {
                    error!(meta.time(); "P2pNetworkSchedulerAction::Disconnect: {addr} already disconnected");
                    return Ok(());
                }
                conn_state.closed = Some(reason.clone().into());

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkSchedulerEffectfulAction::Disconnect {
                    addr: *addr,
                    reason: reason.clone(),
                });
                Ok(())
            }
            P2pNetworkSchedulerAction::Error { addr, error } => {
                let Some(conn_state) = scheduler_state.connections.get_mut(addr) else {
                    error!(meta.time(); "P2pNetworkSchedulerAction::Disconnect: connection {addr} does not exist");
                    return Ok(());
                };
                if conn_state.closed.is_some() {
                    error!(meta.time(); "P2pNetworkSchedulerAction::Disconnect: {addr} already disconnected");
                    return Ok(());
                }
                conn_state.closed = Some(error.clone().into());

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkSchedulerEffectfulAction::Error {
                    addr: *addr,
                    error: error.clone(),
                });
                Ok(())
            }
            P2pNetworkSchedulerAction::Disconnected { addr, reason } => {
                let Some(cn) = scheduler_state.connections.get_mut(addr) else {
                    bug_condition!(
                        "P2pNetworkSchedulerAction::Disconnected: connection {addr} does not exist"
                    );
                    return Ok(());
                };
                if cn.closed.is_none() {
                    bug_condition!(
                        "P2pNetworkSchedulerAction::Disconnect: {addr} is not disconnecting"
                    );
                }

                let incoming = cn.incoming;
                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let state: &P2pState = state.substate()?;

                let peer_with_state = state.peer_with_connection(*addr);
                dispatcher.push(P2pNetworkSchedulerAction::Prune { addr: *addr });

                if reason.is_disconnected() {
                    // statemachine behaviour should continue with this, i.e. dispatch P2pDisconnectionAction::Finish
                    return Ok(());
                }

                match peer_with_state {
                    Some((peer_id, peer_state)) => {
                        // TODO: connection state type should tell if it is finalized
                        match &peer_state.status {
                            crate::P2pPeerStatus::Connecting(
                                crate::connection::P2pConnectionState::Incoming(_),
                            ) => {
                                dispatcher.push(P2pConnectionIncomingAction::FinalizeError {
                                    peer_id,
                                    error: reason.to_string(),
                                });
                            }
                            crate::P2pPeerStatus::Connecting(
                                crate::connection::P2pConnectionState::Outgoing(_),
                            ) => {
                                dispatcher.push(P2pConnectionOutgoingAction::FinalizeError {
                                    peer_id,
                                    error: reason.to_string(),
                                });
                            }
                            crate::P2pPeerStatus::Disconnected { .. } => {
                                // sanity check, should be incoming connection
                                if !incoming {
                                    error!(meta.time(); "disconnected peer connection for address {addr}");
                                } else {
                                    // TODO: introduce action for incoming connection finalization without peer_id
                                }
                            }
                            crate::P2pPeerStatus::Ready(_) => {
                                dispatcher.push(P2pDisconnectionAction::Finish { peer_id });
                            }
                        }
                        dispatcher.push(P2pNetworkSchedulerAction::PruneStreams { peer_id });
                    }
                    None => {
                        // sanity check, should be incoming connection
                        if !incoming {
                            // TODO: error!(meta.time(); "non-existing peer connection for address {addr}");
                        } else {
                            // TODO: introduce action for incoming connection finalization without peer_id
                        }
                    }
                }
                Ok(())
            }
            P2pNetworkSchedulerAction::Prune { addr } => {
                let _ = scheduler_state.connections.remove(addr);

                Ok(())
            }
            P2pNetworkSchedulerAction::PruneStreams { peer_id } => {
                scheduler_state.prune_peer_state(peer_id);

                Ok(())
            }
            P2pNetworkSchedulerAction::PruneStream { peer_id, stream_id } => {
                let Some((_, conn_state)) = scheduler_state
                    .connections
                    .iter_mut()
                    .find(|(_, conn_state)| conn_state.peer_id() == Some(peer_id))
                else {
                    error!(meta.time(); "PruneStream: peer {peer_id} not found");
                    return Ok(());
                };

                if conn_state.streams.remove(stream_id).is_none() {
                    error!(meta.time(); "PruneStream: peer {peer_id} does not have stream {stream_id}");
                }

                Ok(())
            }
        }
    }
}

fn keep_connection_with_unknown_stream() -> bool {
    static VAL: OnceLock<bool> = OnceLock::new();
    *VAL.get_or_init(|| {
        std::env::var("KEEP_CONNECTION_WITH_UNKNOWN_STREAM")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(false)
    })
}
