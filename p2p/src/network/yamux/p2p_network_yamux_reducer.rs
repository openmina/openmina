use openmina_core::{bug_condition, fuzz_maybe, fuzzed_maybe, Substate, SubstateAccess};

use crate::{
    yamux::p2p_network_yamux_state::{YamuxFrame, YamuxFrameInner},
    Data, P2pLimits, P2pNetworkConnectionError, P2pNetworkConnectionMuxState,
    P2pNetworkNoiseAction, P2pNetworkSchedulerAction, P2pNetworkSchedulerState,
    P2pNetworkSelectAction, P2pNetworkStreamState, SelectKind,
};

use super::{
    p2p_network_yamux_state::{YamuxFrameKind, YamuxStreamState},
    P2pNetworkYamuxAction, P2pNetworkYamuxState, YamuxFlags, YamuxPing,
};

impl P2pNetworkYamuxState {
    /// Handles the main reducer logic for Yamux protocol actions. It processes incoming and outgoing
    /// data, selects appropriate behavior based on frame types, and manages the state of streams
    /// within a Yamux session.
    ///
    /// # High-Level Overview
    ///
    /// - When data arrives, it is appended to an internal buffer. The buffer is then parsed for
    ///   valid Yamux frames (using protocol-specific header fields and logic). Incomplete data
    ///   remains in the buffer for future parsing.
    /// - On successful parsing, frames are enqueued for further handling (e.g., dispatching
    ///   actions to notify higher-level protocols or responding to pings).
    /// - If protocol inconsistencies or invalid headers are encountered, it marks an error or
    ///   terminates gracefully, preventing further processing of unexpected data.
    /// - Outgoing data is prepared as frames that respect the window constraints and established
    ///   flags (e.g., SYN, ACK, FIN), and they are dispatched for transmission.
    /// - Once frames are processed, the function checks if the buffer has grown beyond a certain
    ///   threshold relative to its initial capacity. If so, and if the remaining data is small,
    ///   it resets the buffer capacity to a default size to avoid excessive memory usage.
    /// - The function also manages streams and their states, ensuring that proper handshake
    ///   flags are set (SYN, ACK) when a new stream is opened or accepted, enforcing limits on
    ///   the number of streams, and notifying higher-level components about events like
    ///   incoming data or connection errors.
    pub fn reducer<State, Action>(
        // Substate is accessed
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
                yamux_state.extend_buffer(&data);
                yamux_state.parse_frames();

                let frame_count = yamux_state.incoming_frame_count();
                let dispatcher = state_context.into_dispatcher();

                for _ in 0..frame_count {
                    dispatcher.push(P2pNetworkYamuxAction::IncomingFrame { addr })
                }

                Ok(())
            }
            P2pNetworkYamuxAction::OutgoingData {
                addr,
                stream_id,
                data,
                mut flags,
            } => {
                let stream_state = yamux_state
                    .streams
                    .get(&stream_id)
                    .ok_or_else(|| format!("Stream with id {stream_id} not found for `P2pNetworkYamuxAction::OutgoingData`"))?;

                if !stream_state.incoming && !stream_state.established && !stream_state.syn_sent {
                    flags.insert(YamuxFlags::SYN);
                } else if stream_state.incoming && !stream_state.established {
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
            P2pNetworkYamuxAction::IncomingFrame { addr } => {
                let frame = yamux_state.incoming.front().unwrap(); // Cannot fail
                let time = meta.time();
                let mut is_new_stream = false;

                if frame.flags.contains(YamuxFlags::SYN) {
                    // If there is a SYN flag, it means a new incoming stream is being opened
                    yamux_state
                        .streams
                        .insert(frame.stream_id, YamuxStreamState::incoming());

                    is_new_stream = !frame.is_session_stream();
                    if is_new_stream {
                        connection_state
                            .streams
                            .insert(frame.stream_id, P2pNetworkStreamState::new_incoming(time));
                    }
                }

                // If there is an ACK flag, it means the outgoing stream has been established
                if frame.flags.contains(YamuxFlags::ACK) {
                    // TODO: what if the stream doesn't exist?
                    if let Some(stream) = yamux_state.streams.get_mut(&frame.stream_id) {
                        stream.established = true;
                    }
                }

                let frame_stream_id = frame.stream_id;
                let frame_flags = frame.flags;
                let frame_kind = frame.kind();

                let (dispatcher, state) = state_context.into_dispatcher_and_state();

                let limits: &P2pLimits = state.substate()?;
                let connection_state =
                    <State as SubstateAccess<P2pNetworkSchedulerState>>::substate(state)?
                        .connection_state(&addr)
                        .ok_or_else(|| format!("Connection not found {}", addr))?;

                let stream_exists = connection_state.get_yamux_stream(frame_stream_id).is_some();
                let peer_id = match connection_state.peer_id() {
                    Some(peer_id) => *peer_id,
                    None => {
                        bug_condition!("Peer id must exist");
                        return Ok(());
                    }
                };

                // Peer reset this stream
                if stream_exists && frame_flags.contains(YamuxFlags::RST) {
                    dispatcher.push(P2pNetworkSchedulerAction::Error {
                        addr,
                        error: P2pNetworkConnectionError::StreamReset(frame_stream_id),
                    });
                    return Ok(());
                }

                // Enforce stream limits
                if is_new_stream {
                    if connection_state.incoming_streams_count() > limits.max_streams() {
                        dispatcher.push(P2pNetworkYamuxAction::OutgoingFrame {
                            addr,
                            frame: YamuxFrame {
                                flags: YamuxFlags::FIN,
                                stream_id: frame_stream_id,
                                inner: YamuxFrameInner::WindowUpdate { difference: 0 },
                            },
                        });
                    } else {
                        dispatcher.push(P2pNetworkSelectAction::Init {
                            addr,
                            kind: SelectKind::Stream(peer_id, frame_stream_id),
                            incoming: true,
                        });
                    }
                }

                dispatcher.push(match frame_kind {
                    YamuxFrameKind::Data => P2pNetworkYamuxAction::IncomingFrameData { addr },
                    YamuxFrameKind::WindowUpdate => {
                        P2pNetworkYamuxAction::IncomingFrameWindowUpdate { addr }
                    }
                    YamuxFrameKind::Ping => P2pNetworkYamuxAction::IncomingFramePing { addr },
                    YamuxFrameKind::GoAway => P2pNetworkYamuxAction::IncomingFrameGoAway { addr },
                });

                Ok(())
            }
            P2pNetworkYamuxAction::IncomingFrameData { addr } => {
                let frame = yamux_state.incoming.pop_front().unwrap(); // Cannot fail
                let YamuxFrameInner::Data(data) = &frame.inner else {
                    bug_condition!("Expected Data frame");
                    return Ok(());
                };

                let Some(stream) = yamux_state.streams.get_mut(&frame.stream_id) else {
                    return Ok(());
                };

                // Process incoming data and check if we need to update window
                let window_update_info =
                    if let Some(window_increase) = stream.process_incoming_data(&frame) {
                        Some((frame.stream_id, window_increase))
                    } else {
                        None
                    };

                let peer_id = match connection_state.peer_id() {
                    Some(peer_id) => *peer_id,
                    None => {
                        bug_condition!("Peer id must exist");
                        return Ok(());
                    }
                };

                let dispatcher = state_context.into_dispatcher();

                if let Some((update_stream_id, difference)) = window_update_info {
                    dispatcher.push(P2pNetworkYamuxAction::OutgoingFrame {
                        addr,
                        frame: YamuxFrame {
                            stream_id: update_stream_id,
                            flags: YamuxFlags::empty(),
                            inner: YamuxFrameInner::WindowUpdate { difference },
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
                Ok(())
            }
            P2pNetworkYamuxAction::IncomingFramePing { addr } => {
                let frame = yamux_state.incoming.pop_front().unwrap(); // Cannot fail

                // If the frame has an ACK flag, it means the ping was a response, nothing to do
                if frame.flags.contains(YamuxFlags::ACK) {
                    return Ok(());
                }

                let YamuxFrameInner::Ping { opaque } = frame.inner else {
                    bug_condition!(
                        "Expected Ping frame for action `P2pNetworkYamuxAction::IncomingFramePing`"
                    );
                    return Ok(());
                };

                let ping = YamuxPing {
                    stream_id: frame.stream_id,
                    opaque,
                    response: true,
                };

                let dispatcher = state_context.into_dispatcher();
                dispatcher.push(P2pNetworkYamuxAction::OutgoingFrame {
                    addr,
                    frame: ping.into_frame(),
                });

                Ok(())
            }
            P2pNetworkYamuxAction::IncomingFrameWindowUpdate { addr } => {
                let frame = yamux_state.incoming.pop_front().unwrap(); // Cannot fail
                let YamuxFrameInner::WindowUpdate { difference } = frame.inner else {
                    bug_condition!("Expected WindowUpdate frame for action `P2pNetworkYamuxAction::IncomingFrameWindowUpdate`");
                    return Ok(());
                };

                let stream = yamux_state
                    .streams
                    .entry(frame.stream_id)
                    .or_insert_with(YamuxStreamState::incoming);

                let sendable_frames = stream.update_remote_window(difference);

                let dispatcher = state_context.into_dispatcher();

                for frame in sendable_frames {
                    dispatcher.push(P2pNetworkYamuxAction::OutgoingFrame { addr, frame });
                }

                Ok(())
            }
            P2pNetworkYamuxAction::IncomingFrameGoAway { .. } => {
                let frame = yamux_state.incoming.pop_front().unwrap(); // Cannot fail
                let YamuxFrameInner::GoAway(res) = frame.inner else {
                    bug_condition!("Expected GoAway frame for action `P2pNetworkYamuxAction::IncomingFrameGoAway`");
                    return Ok(());
                };

                yamux_state.set_res(res);
                Ok(())
            }
            P2pNetworkYamuxAction::OutgoingFrame { mut frame, addr } => {
                let stream_id = frame.stream_id;
                let Some(stream) = yamux_state.streams.get_mut(&stream_id) else {
                    return Ok(());
                };

                match &frame.inner {
                    YamuxFrameInner::Data(_) => {
                        let (accepted, remaining) =
                            stream.queue_frame(frame, yamux_state.pending_outgoing_limit);

                        if remaining.is_some() {
                            let dispatcher = state_context.into_dispatcher();
                            let error = P2pNetworkConnectionError::YamuxOverflow(stream_id);
                            dispatcher.push(P2pNetworkSchedulerAction::Error { addr, error });
                            return Ok(());
                        }

                        frame =
                            accepted.expect("frame should be accepted or error should be returned");
                    }
                    YamuxFrameInner::WindowUpdate { difference } => {
                        stream.update_local_window(*difference);
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

                let peer_id = match connection_state.peer_id() {
                    Some(peer_id) => *peer_id,
                    None => {
                        bug_condition!("Peer id must exist");
                        return Ok(());
                    }
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
