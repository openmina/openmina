use self::p2p_network_yamux_state::{YamuxFlags, YamuxFrame, YamuxFrameInner};

use super::{super::*, *};

impl P2pNetworkYamuxAction {
    pub fn effects<Store, S>(self, _meta: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
    {
        let state = store.state();
        let Some(state) = state.network.scheduler.connections.get(self.addr()) else {
            return;
        };
        let peer_id = match &state.auth {
            Some(P2pNetworkAuthState::Noise(noise)) => match &noise.inner {
                Some(P2pNetworkNoiseStateInner::Done { remote_peer_id, .. }) => *remote_peer_id,
                _ => return,
            },
            _ => return,
        };
        let Some(P2pNetworkConnectionMuxState::Yamux(state)) = &state.mux else {
            return;
        };

        let incoming = state.incoming.front().cloned();

        match self {
            Self::IncomingData { addr, .. } => {
                if let Some(frame) = incoming {
                    store.dispatch(P2pNetworkYamuxAction::IncomingFrame { addr, frame });
                }
            }
            Self::IncomingFrame { addr, frame } => {
                let frame = &frame;
                let Some(stream) = state.streams.get(&frame.stream_id).cloned() else {
                    return;
                };

                if frame.flags.contains(YamuxFlags::SYN) && frame.stream_id != 0 {
                    store.dispatch(P2pNetworkSelectAction::Init {
                        addr,
                        kind: SelectKind::Stream(peer_id, frame.stream_id),
                        incoming: true,
                        send_handshake: true,
                    });
                }
                match &frame.inner {
                    YamuxFrameInner::Data(data) => {
                        if stream.window_ours < 64 * 1024 {
                            store.dispatch(P2pNetworkYamuxAction::OutgoingFrame {
                                addr,
                                frame: YamuxFrame {
                                    stream_id: frame.stream_id,
                                    flags: YamuxFlags::empty(),
                                    inner: YamuxFrameInner::WindowUpdate {
                                        difference: 256 * 1024,
                                    },
                                },
                            });
                        }

                        store.dispatch(P2pNetworkSelectAction::IncomingData {
                            addr,
                            kind: SelectKind::Stream(peer_id, frame.stream_id),
                            data: data.clone(),
                            fin: frame.flags.contains(YamuxFlags::FIN),
                        });
                    }
                    YamuxFrameInner::Ping { opaque } => {
                        let response = frame.flags.contains(YamuxFlags::ACK);
                        // if this ping is not response create our response
                        if !response {
                            let ping = YamuxPing {
                                stream_id: frame.stream_id,
                                opaque: *opaque,
                                response: true,
                            };
                            store.dispatch(P2pNetworkYamuxAction::OutgoingFrame {
                                addr,
                                frame: ping.clone().into_frame(),
                            });
                        }
                    }
                    _ => {}
                }

                if let Some(frame) = incoming {
                    store.dispatch(P2pNetworkYamuxAction::IncomingFrame { addr, frame });
                }
            }
            Self::OutgoingFrame { addr, frame } => {
                store.dispatch(P2pNetworkNoiseAction::OutgoingData {
                    addr,
                    data: frame.clone().into_bytes().into(),
                });
            }
            Self::OutgoingData {
                addr,
                stream_id,
                data,
                fin,
            } => {
                let Some(stream) = state.streams.get(&stream_id) else {
                    return;
                };
                let mut flags = YamuxFlags::empty();
                if !stream.incoming && !stream.established && !stream.syn_sent {
                    flags.insert(YamuxFlags::SYN);
                } else if stream.incoming && !stream.established {
                    flags.insert(YamuxFlags::ACK);
                }
                if fin {
                    flags.insert(YamuxFlags::FIN);
                }
                let frame = YamuxFrame {
                    flags,
                    stream_id,
                    inner: YamuxFrameInner::Data(data.clone()),
                };
                store.dispatch(P2pNetworkYamuxAction::OutgoingFrame { addr, frame });
            }
            Self::PingStream { addr, ping } => {
                store.dispatch(P2pNetworkYamuxAction::OutgoingFrame {
                    addr,
                    frame: ping.clone().into_frame(),
                });
            }
            Self::OpenStream {
                addr, stream_id, ..
            } => {
                store.dispatch(P2pNetworkSelectAction::Init {
                    addr,
                    kind: SelectKind::Stream(peer_id, stream_id),
                    incoming: false,
                    send_handshake: true,
                });
            }
        }
    }
}
