use std::collections::BTreeMap;

use openmina_core::error;

use crate::{
    channels::{ChannelId, P2pChannelsState},
    P2pPeerState, P2pPeerStatus, P2pPeerStatusReady, PeerId,
};

use super::{super::*, p2p_network_scheduler_state::P2pNetworkConnectionState, *};

impl P2pNetworkSchedulerState {
    pub fn reducer(
        &mut self,
        peers: &mut BTreeMap<PeerId, P2pPeerState>,
        action: redux::ActionWithMeta<&P2pNetworkSchedulerAction>,
    ) {
        let (action, meta) = action.split();
        match action {
            P2pNetworkSchedulerAction::InterfaceDetected(a) => drop(self.interfaces.insert(a.ip)),
            P2pNetworkSchedulerAction::InterfaceExpired(a) => drop(self.interfaces.remove(&a.ip)),
            P2pNetworkSchedulerAction::IncomingConnectionIsReady(_) => {}
            P2pNetworkSchedulerAction::IncomingDidAccept(a) => {
                let Some(addr) = a.addr else {
                    return;
                };

                self.connections.insert(
                    addr,
                    P2pNetworkConnectionState {
                        incoming: true,
                        pnet: P2pNetworkPnetState::new(self.pnet_key),
                        select_auth: P2pNetworkSelectState::default(),
                        auth: None,
                        select_mux: P2pNetworkSelectState::default(),
                        mux: None,
                        streams: BTreeMap::default(),
                    },
                );
            }
            P2pNetworkSchedulerAction::OutgoingDidConnect(a) => {
                self.connections.insert(
                    a.addr,
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
                    },
                );
            }
            P2pNetworkSchedulerAction::IncomingDataIsReady(_) => {}
            P2pNetworkSchedulerAction::IncomingDataDidReceive(a) => {
                if a.result.is_err() {
                    self.connections.remove(&a.addr);
                }
            }
            P2pNetworkSchedulerAction::SelectDone(a) => {
                let Some(connection) = self.connections.get_mut(&a.addr) else {
                    return;
                };
                match &a.kind {
                    SelectKind::Multiplexing(peer_id) => {
                        let enabled_channels = Some(ChannelId::Rpc).into_iter().collect();
                        let state = P2pPeerState {
                            is_libp2p: true,
                            dial_opts: None,
                            status: P2pPeerStatus::Ready(P2pPeerStatusReady {
                                is_incoming: a.incoming,
                                connected_since: meta.time(),
                                channels: P2pChannelsState::new(&enabled_channels),
                                best_tip: None,
                            }),
                        };
                        peers.insert(*peer_id, state);
                    }
                    _ => {}
                }
                match &a.protocol {
                    Some(token::Protocol::Auth(token::AuthKind::Noise)) => {
                        connection.auth =
                            Some(P2pNetworkAuthState::Noise(P2pNetworkNoiseState::default()));
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
                        let SelectKind::Stream(peer_id, stream_id) = a.kind else {
                            error!(meta.time(); "incorrect stream kind for protocol stream: {stream_kind:?}");
                            return;
                        };
                        match stream_kind {
                            token::StreamKind::Rpc(_) => {
                                if a.incoming {
                                    self.rpc_incoming_streams
                                        .entry(peer_id)
                                        .or_default()
                                        .insert(
                                            stream_id,
                                            P2pNetworkRpcState::new(a.addr, stream_id),
                                        );
                                } else {
                                    self.rpc_outgoing_streams
                                        .entry(peer_id)
                                        .or_default()
                                        .insert(
                                            stream_id,
                                            P2pNetworkRpcState::new(a.addr, stream_id),
                                        );
                                }
                            }
                            token::StreamKind::Broadcast(_) => unimplemented!(),
                            token::StreamKind::Discovery(_) => {}
                        }
                    }
                    None => {}
                }
            }
            P2pNetworkSchedulerAction::SelectError(a) => {
                if let Some(stream_id) = &a.kind.stream_id() {
                    if let Some(connection) = self.connections.get_mut(&a.addr) {
                        connection.streams.remove(stream_id);
                    }
                } else {
                    self.connections.remove(&a.addr);
                }
            }
            P2pNetworkSchedulerAction::YamuxDidInit(a) => {
                if let Some(cn) = self.connections.get_mut(&a.addr) {
                    if let Some(P2pNetworkConnectionMuxState::Yamux(yamux)) = &mut cn.mux {
                        yamux.init = true;
                    }
                }
            }
        }
    }
}
