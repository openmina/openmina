use openmina_core::{bug_condition, fuzz_maybe, fuzzed_maybe, Substate, SubstateAccess};

use crate::P2pLimits;

use self::p2p_network_yamux_state::{
    YamuxFlags, YamuxFrame, YamuxFrameInner, YamuxFrameParseError, YamuxSessionError,
    YamuxStreamState,
};

use super::{super::*, *};

impl P2pNetworkYamuxState {
    pub fn set_err(&mut self, err: YamuxFrameParseError) {
        self.terminated = Some(Err(err));
    }

    pub fn set_res(&mut self, res: Result<(), YamuxSessionError>) {
        self.terminated = Some(Ok(res));
    }

    /// Substate is accessed
    pub fn reducer<State, Action>(
        mut state_context: Substate<Action, State, P2pNetworkSchedulerState>,
        action: redux::ActionWithMeta<P2pNetworkYamuxAction>,
    ) -> Result<(), String>
    where
        State: crate::P2pStateTrait,
        Action: crate::P2pActionTrait<State>,
    {
        let (action, meta) = action.split();
        let connection_state = state_context
            .get_substate_mut()?
            .connection_state_mut(action.addr())
            .ok_or_else(|| format!("Connection not found for action: {action:?}"))
            .inspect_err(|e| bug_condition!("{}", e))?;

        let P2pNetworkConnectionMuxState::Yamux(yamux_state) = connection_state
            .mux
            .as_mut()
            .ok_or_else(|| format!("Invalid yamux state for action: {action:?}"))?;

        if yamux_state.terminated.is_some() {
            return Ok(());
        }

        match action {
            P2pNetworkYamuxAction::IncomingData { data, addr } => {
                yamux_state.buffer.extend_from_slice(&data);
                let mut offset = 0;
                loop {
                    let buf = &yamux_state.buffer[offset..];
                    if buf.len() >= 12 {
                        let _version = match buf[0] {
                            0 => 0,
                            unknown => {
                                yamux_state.set_err(YamuxFrameParseError::Version(unknown));
                                break;
                            }
                        };
                        let flags = u16::from_be_bytes(buf[2..4].try_into().expect("cannot fail"));
                        let Some(flags) = YamuxFlags::from_bits(flags) else {
                            yamux_state.set_err(YamuxFrameParseError::Flags(flags));
                            break;
                        };
                        let stream_id =
                            u32::from_be_bytes(buf[4..8].try_into().expect("cannot fail"));
                        let b = buf[8..12].try_into().expect("cannot fail");

                        match buf[1] {
                            0 => {
                                let len = u32::from_be_bytes(b) as usize;
                                if len > yamux_state.message_size_limit {
                                    yamux_state.set_res(Err(YamuxSessionError::Internal));
                                    break;
                                }
                                if buf.len() >= 12 + len {
                                    let frame = YamuxFrame {
                                        flags,
                                        stream_id,
                                        inner: YamuxFrameInner::Data(
                                            buf[12..(12 + len)].to_vec().into(),
                                        ),
                                    };
                                    yamux_state.incoming.push_back(frame);
                                    offset += 12 + len;
                                    continue;
                                }
                            }
                            1 => {
                                let difference = i32::from_be_bytes(b);
                                let frame = YamuxFrame {
                                    flags,
                                    stream_id,
                                    inner: YamuxFrameInner::WindowUpdate { difference },
                                };
                                yamux_state.incoming.push_back(frame);
                                offset += 12;
                                continue;
                            }
                            2 => {
                                let opaque = i32::from_be_bytes(b);
                                let frame = YamuxFrame {
                                    flags,
                                    stream_id,
                                    inner: YamuxFrameInner::Ping { opaque },
                                };
                                yamux_state.incoming.push_back(frame);
                                offset += 12;
                                continue;
                            }
                            3 => {
                                let code = u32::from_be_bytes(b);
                                let result = match code {
                                    0 => Ok(()),
                                    1 => Err(YamuxSessionError::Protocol),
                                    2 => Err(YamuxSessionError::Internal),
                                    unknown => {
                                        yamux_state
                                            .set_err(YamuxFrameParseError::ErrorCode(unknown));
                                        break;
                                    }
                                };
                                let frame = YamuxFrame {
                                    flags,
                                    stream_id,
                                    inner: YamuxFrameInner::GoAway(result),
                                };
                                yamux_state.incoming.push_back(frame);
                                offset += 12;
                                continue;
                            }
                            unknown => {
                                yamux_state.set_err(YamuxFrameParseError::Type(unknown));
                                break;
                            }
                        }
                    }

                    break;
                }

                yamux_state.buffer = yamux_state.buffer[offset..].to_vec();

                let incoming_data = yamux_state.incoming.clone();
                let dispatcher = state_context.into_dispatcher();
                incoming_data.into_iter().for_each(|frame| {
                    dispatcher.push(P2pNetworkYamuxAction::IncomingFrame { addr, frame })
                });

                Ok(())
            }
            P2pNetworkYamuxAction::OutgoingData {
                addr,
                stream_id,
                data,
                mut flags,
            } => {
                let yamux_state = yamux_state
                    .streams
                    .get(&stream_id)
                    .ok_or_else(|| format!("Stream with id {stream_id} not found for `P2pNetworkYamuxAction::OutgoingData`"))?;

                if !yamux_state.incoming && !yamux_state.established && !yamux_state.syn_sent {
                    flags.insert(YamuxFlags::SYN);
                } else if yamux_state.incoming && !yamux_state.established {
                    flags.insert(YamuxFlags::ACK);
                }

                fuzz_maybe!(&mut flags, crate::fuzzer::mutate_yamux_flags);

                let frame = YamuxFrame {
                    flags,
                    stream_id,
                    inner: YamuxFrameInner::Data(data),
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkYamuxAction::OutgoingFrame { addr, frame });

                Ok(())
            }
            P2pNetworkYamuxAction::IncomingFrame { addr, frame } => {
                if let Some(frame) = yamux_state.incoming.pop_front() {
                    if frame.flags.contains(YamuxFlags::SYN) {
                        yamux_state
                            .streams
                            .insert(frame.stream_id, YamuxStreamState::incoming());

                        if frame.stream_id != 0 {
                            connection_state.streams.insert(
                                frame.stream_id,
                                P2pNetworkStreamState::new_incoming(meta.time()),
                            );
                        }
                    }
                    if frame.flags.contains(YamuxFlags::ACK) {
                        yamux_state
                            .streams
                            .entry(frame.stream_id)
                            .or_default()
                            .established = true;
                    }

                    match frame.inner {
                        YamuxFrameInner::Data(data) => {
                            if let Some(stream) = yamux_state.streams.get_mut(&frame.stream_id) {
                                // must not underflow
                                // TODO: check it and disconnect peer that violates flow rules
                                stream.window_ours =
                                    stream.window_ours.wrapping_sub(data.len() as u32);
                            }
                        }
                        YamuxFrameInner::WindowUpdate { difference } => {
                            yamux_state
                                .streams
                                .entry(frame.stream_id)
                                .or_insert_with(YamuxStreamState::incoming)
                                .update_window(false, difference);
                        }
                        YamuxFrameInner::Ping { .. } => {}
                        YamuxFrameInner::GoAway(res) => yamux_state.set_res(res),
                    }
                }

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let limits: &P2pLimits = state.substate()?;
                let max_streams = limits.max_streams();
                let connection_state =
                    <State as SubstateAccess<P2pNetworkSchedulerState>>::substate(state)?
                        .connection_state(&addr)
                        .ok_or_else(|| format!("Connection not found {}", addr))?;

                let stream = connection_state
                    .yamux_state()
                    .and_then(|yamux_state| yamux_state.streams.get(&frame.stream_id))
                    .ok_or_else(|| format!("Stream with id {} not found for `P2pNetworkYamuxAction::IncomingFrame`", frame.stream_id))?;

                let peer_id = match connection_state
                    .auth
                    .as_ref()
                    .and_then(|P2pNetworkAuthState::Noise(noise)| noise.peer_id())
                {
                    Some(peer_id) => *peer_id,
                    None => return Ok(()),
                };

                if frame.flags.contains(YamuxFlags::RST) {
                    dispatcher.push(P2pNetworkSchedulerAction::Error {
                        addr,
                        error: P2pNetworkConnectionError::StreamReset(frame.stream_id),
                    });
                    return Ok(());
                }

                if frame.flags.contains(YamuxFlags::SYN) && frame.stream_id != 0 {
                    // count incoming streams
                    let incoming_streams_number = connection_state
                        .streams
                        .values()
                        .filter(|s| s.select.is_incoming())
                        .count();

                    match (max_streams, incoming_streams_number) {
                        (Limit::Some(limit), actual) if actual > limit => {
                            dispatcher.push(P2pNetworkYamuxAction::OutgoingFrame {
                                addr,
                                frame: YamuxFrame {
                                    flags: YamuxFlags::FIN,
                                    stream_id: frame.stream_id,
                                    inner: YamuxFrameInner::Data(vec![].into()),
                                },
                            });
                        }
                        _ => {
                            dispatcher.push(P2pNetworkSelectAction::Init {
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
                            dispatcher.push(P2pNetworkYamuxAction::OutgoingFrame {
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

                        dispatcher.push(P2pNetworkSelectAction::IncomingData {
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
                            dispatcher.push(P2pNetworkYamuxAction::OutgoingFrame {
                                addr,
                                frame: ping.into_frame(),
                            });
                        }
                    }
                    _ => {}
                }

                Ok(())
            }
            P2pNetworkYamuxAction::OutgoingFrame { frame, addr } => {
                let Some(stream) = yamux_state.streams.get_mut(&frame.stream_id) else {
                    return Ok(());
                };
                match &frame.inner {
                    YamuxFrameInner::Data(data) => {
                        // must not underflow
                        // the action must not dispatch if it doesn't fit in the window
                        // TODO: add pending queue, where frames will wait for window increase
                        stream.window_theirs = stream.window_theirs.wrapping_sub(data.len() as u32);
                    }
                    YamuxFrameInner::WindowUpdate { difference } => {
                        stream.update_window(true, *difference);
                    }
                    _ => {}
                }

                if frame.flags.contains(YamuxFlags::FIN) {
                    connection_state.streams.remove(&frame.stream_id);
                    stream.writable = false;
                } else {
                    if frame.flags.contains(YamuxFlags::ACK) {
                        stream.established |= true;
                    }
                    if frame.flags.contains(YamuxFlags::SYN) {
                        stream.syn_sent = true;
                    }
                }

                let dispatcher = state_context.into_dispatcher();
                let data = fuzzed_maybe!(
                    Data::from(frame.into_bytes()),
                    crate::fuzzer::mutate_yamux_frame
                );
                dispatcher.push(P2pNetworkNoiseAction::OutgoingData { addr, data });
                Ok(())
            }
            P2pNetworkYamuxAction::PingStream { addr, ping } => {
                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkYamuxAction::OutgoingFrame {
                    addr,
                    frame: ping.into_frame(),
                });

                Ok(())
            }
            P2pNetworkYamuxAction::OpenStream {
                stream_id,
                stream_kind,
                addr,
            } => {
                yamux_state
                    .streams
                    .insert(stream_id, YamuxStreamState::default());
                connection_state.streams.insert(
                    stream_id,
                    P2pNetworkStreamState::new(stream_kind, meta.time()),
                );

                let peer_id = match connection_state
                    .auth
                    .as_ref()
                    .and_then(|P2pNetworkAuthState::Noise(noise)| noise.peer_id())
                {
                    Some(peer_id) => *peer_id,
                    None => return Ok(()),
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkSelectAction::Init {
                    addr,
                    kind: SelectKind::Stream(peer_id, stream_id),
                    incoming: false,
                });
                Ok(())
            }
        }
    }
}

impl YamuxStreamState {
    pub fn update_window(&mut self, ours: bool, difference: i32) {
        let window = if ours {
            &mut self.window_ours
        } else {
            &mut self.window_theirs
        };
        if difference < 0 {
            let decreasing = (-difference) as u32;
            if *window < decreasing {
                *window = 0;
            } else {
                *window = (*window).wrapping_sub(decreasing);
            }
        } else {
            let increasing = difference as u32;
            *window = (*window).wrapping_add(increasing);
        }
    }
}
