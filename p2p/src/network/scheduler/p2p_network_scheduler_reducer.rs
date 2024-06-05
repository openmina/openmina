use std::collections::BTreeMap;

use openmina_core::error;

use super::{super::*, p2p_network_scheduler_state::P2pNetworkConnectionState, *};

impl P2pNetworkSchedulerState {
    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkSchedulerAction>) {
        let (action, meta) = action.split();
        match action {
            P2pNetworkSchedulerAction::InterfaceDetected { ip, .. } => {
                self.interfaces.insert(*ip);
            }
            P2pNetworkSchedulerAction::InterfaceExpired { ip, .. } => {
                self.interfaces.remove(ip);
            }
            P2pNetworkSchedulerAction::ListenerReady { listener } => {
                self.listeners.insert(*listener);
            }
            P2pNetworkSchedulerAction::ListenerError { listener, error: _ } => {
                self.listeners.remove(listener);
            }
            P2pNetworkSchedulerAction::IncomingConnectionIsReady { .. } => {}
            P2pNetworkSchedulerAction::IncomingDidAccept { addr, .. } => {
                let Some(addr) = addr else {
                    return;
                };
                self.connections.insert(
                    *addr,
                    P2pNetworkConnectionState {
                        incoming: true,
                        pnet: P2pNetworkPnetState::new(self.pnet_key),
                        select_auth: P2pNetworkSelectState::default(),
                        auth: None,
                        select_mux: P2pNetworkSelectState::default(),
                        mux: None,
                        streams: BTreeMap::default(),
                        closed: None,
                        limit: P2pNetworkConnectionState::INITIAL_LIMIT,
                    },
                );
            }
            P2pNetworkSchedulerAction::OutgoingConnect { addr } => {
                self.connections.insert(
                    *addr,
                    P2pNetworkConnectionState {
                        incoming: false,
                        pnet: P2pNetworkPnetState::new(self.pnet_key),
                        select_auth: P2pNetworkSelectState::initiator_auth(token::AuthKind::Noise),
                        auth: None,
                        select_mux: P2pNetworkSelectState::initiator_mux(
                            token::MuxKind::Yamux1_0_0,
                        ),
                        mux: None,
                        streams: BTreeMap::default(),
                        closed: None,
                        limit: P2pNetworkConnectionState::INITIAL_LIMIT,
                    },
                );
            }
            P2pNetworkSchedulerAction::OutgoingDidConnect { .. } => {
                // TODO: change to connected
            }
            P2pNetworkSchedulerAction::IncomingDataIsReady { .. } => {}
            P2pNetworkSchedulerAction::IncomingDataDidReceive { result, addr } => {
                let Some(state) = self.connections.get_mut(addr) else {
                    return;
                };

                if let Ok(data) = result {
                    state.consume(data.len());
                }
            }
            P2pNetworkSchedulerAction::SelectDone {
                addr,
                kind,
                protocol,
                incoming,
                ..
            } => {
                let Some(connection) = self.connections.get_mut(addr) else {
                    return;
                };
                match protocol {
                    Some(token::Protocol::Auth(token::AuthKind::Noise)) => {
                        connection.auth = Some(P2pNetworkAuthState::Noise(
                            P2pNetworkNoiseState::new(self.local_pk.clone(), false),
                        ));
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
                            return;
                        };
                        match stream_kind {
                            token::StreamKind::Rpc(_) => {
                                if *incoming {
                                    self.rpc_incoming_streams
                                        .entry(*peer_id)
                                        .or_default()
                                        .insert(
                                            *stream_id,
                                            P2pNetworkRpcState::new(*addr, *stream_id),
                                        );
                                } else {
                                    self.rpc_outgoing_streams
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
            }
            P2pNetworkSchedulerAction::SelectError { .. } => {
                // NOOP, error should be triggered
            }
            P2pNetworkSchedulerAction::YamuxDidInit { addr, .. } => {
                if let Some(cn) = self.connections.get_mut(addr) {
                    if let Some(P2pNetworkConnectionMuxState::Yamux(yamux)) = &mut cn.mux {
                        yamux.init = true;
                    }
                }
            }
            P2pNetworkSchedulerAction::Disconnect { addr, reason } => {
                let Some(conn_state) = self.connections.get_mut(addr) else {
                    error!(meta.time(); "P2pNetworkSchedulerAction::Disconnect: connection {addr} does not exist");
                    return;
                };
                if conn_state.closed.is_some() {
                    error!(meta.time(); "P2pNetworkSchedulerAction::Disconnect: {addr} already disconnected");
                    return;
                }
                conn_state.closed = Some(reason.clone().into());
            }
            P2pNetworkSchedulerAction::Error { addr, error } => {
                let Some(conn_state) = self.connections.get_mut(addr) else {
                    error!(meta.time(); "P2pNetworkSchedulerAction::Disconnect: connection {addr} does not exist");
                    return;
                };
                if conn_state.closed.is_some() {
                    error!(meta.time(); "P2pNetworkSchedulerAction::Disconnect: {addr} already disconnected");
                    return;
                }
                conn_state.closed = Some(error.clone().into());
            }
            P2pNetworkSchedulerAction::Disconnected { addr, .. } => {
                let Some(cn) = self.connections.get_mut(addr) else {
                    error!(meta.time(); "P2pNetworkSchedulerAction::Disconnected: connection {addr} does not exist");
                    return;
                };
                if cn.closed.is_none() {
                    error!(meta.time(); "P2pNetworkSchedulerAction::Disconnect: {addr} is not disconnecting");
                }
            }
            P2pNetworkSchedulerAction::Prune { addr } => {
                let _ = self.connections.remove(addr);
            }
            P2pNetworkSchedulerAction::PruneStreams { peer_id } => {
                self.rpc_incoming_streams.remove(peer_id);
                self.rpc_outgoing_streams.remove(peer_id);
                if let Some(discovery_state) = self.discovery_state.as_mut() {
                    discovery_state.streams.remove(peer_id);
                }
            }
        }
    }
}
