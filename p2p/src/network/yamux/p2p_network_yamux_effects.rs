use openmina_core::{fuzz_maybe, fuzzed_maybe};

use super::p2p_network_yamux_state::{YamuxFrame, YamuxFrameInner};

use super::{super::*, *};

impl P2pNetworkYamuxAction {
    pub fn effects<Store, S>(self, _meta: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
    {
        let state = store.state();
        let max_streams = state.config.limits.max_streams();
        let Some(state) = state.network.scheduler.connections.get(self.addr()) else {
            return;
        };
        let streams = &state.streams;
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

        match self {
            P2pNetworkYamuxAction::IncomingData { addr, .. } => {
                for frame in state.incoming.clone() {
                    store.dispatch(P2pNetworkYamuxAction::IncomingFrame { addr, frame });
                }
            }
            P2pNetworkYamuxAction::IncomingFrame { addr, frame } => {
                let frame = &frame;
                let Some(stream) = state.streams.get(&frame.stream_id).cloned() else {
                    return;
                };

                if frame.flags.contains(YamuxFlags::RST) {
                    store.dispatch(P2pNetworkSchedulerAction::Error {
                        addr,
                        error: P2pNetworkConnectionError::StreamReset(frame.stream_id),
                    });
                    return;
                }

                if frame.flags.contains(YamuxFlags::SYN) && frame.stream_id != 0 {
                    // count incoming streams
                    let incoming_streams_number =
                        streams.values().filter(|s| s.select.is_incoming()).count();
                    match (max_streams, incoming_streams_number) {
                        (Limit::Some(limit), actual) if actual > limit => {
                            store.dispatch(P2pNetworkYamuxAction::OutgoingFrame {
                                addr,
                                frame: YamuxFrame {
                                    flags: YamuxFlags::FIN,
                                    stream_id: frame.stream_id,
                                    inner: YamuxFrameInner::Data(vec![].into()),
                                },
                            });
                        }
                        _ => {
                            store.dispatch(P2pNetworkSelectAction::Init {
                                addr,
                                kind: SelectKind::Stream(peer_id, frame.stream_id),
                                incoming: true,
                            });
                        }
                    }
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
                            peer_id,
                            stream_id: frame.stream_id,
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
            }
            P2pNetworkYamuxAction::OutgoingFrame { addr, frame } => {
                let data =
                    fuzzed_maybe!(frame.into_bytes().into(), crate::fuzzer::mutate_yamux_frame);
                store.dispatch(P2pNetworkNoiseAction::OutgoingData { addr, data });
            }
            P2pNetworkYamuxAction::OutgoingData {
                addr,
                stream_id,
                data,
                mut flags,
            } => {
                let Some(stream) = state.streams.get(&stream_id) else {
                    return;
                };
                if !stream.incoming && !stream.established && !stream.syn_sent {
                    flags.insert(YamuxFlags::SYN);
                } else if stream.incoming && !stream.established {
                    flags.insert(YamuxFlags::ACK);
                }

                fuzz_maybe!(&mut flags, crate::fuzzer::mutate_yamux_flags);

                let frame = YamuxFrame {
                    flags,
                    stream_id,
                    inner: YamuxFrameInner::Data(data.clone()),
                };
                store.dispatch(P2pNetworkYamuxAction::OutgoingFrame { addr, frame });
            }
            P2pNetworkYamuxAction::PingStream { addr, ping } => {
                store.dispatch(P2pNetworkYamuxAction::OutgoingFrame {
                    addr,
                    frame: ping.clone().into_frame(),
                });
            }
            P2pNetworkYamuxAction::OpenStream {
                addr, stream_id, ..
            } => {
                store.dispatch(P2pNetworkSelectAction::Init {
                    addr,
                    kind: SelectKind::Stream(peer_id, stream_id),
                    incoming: false,
                });
            }
        }
    }
}
