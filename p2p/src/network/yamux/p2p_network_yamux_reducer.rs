use std::collections::VecDeque;

use openmina_core::{bug_condition, fuzz_maybe, fuzzed_maybe, Substate, SubstateAccess};

use crate::{
    yamux::p2p_network_yamux_state::{YamuxFrame, YamuxFrameInner},
    Data, P2pLimits, P2pNetworkAuthState, P2pNetworkConnectionError, P2pNetworkConnectionMuxState,
    P2pNetworkNoiseAction, P2pNetworkSchedulerAction, P2pNetworkSchedulerState,
    P2pNetworkSelectAction, P2pNetworkStreamState, SelectKind,
};

use super::{
    p2p_network_yamux_state::{YamuxStreamState, MAX_WINDOW_SIZE},
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
                let mut pending_outgoing = VecDeque::default();
                let Some(frame) = yamux_state.incoming.pop_front() else {
                    bug_condition!(
                        "Frame not found for action `P2pNetworkYamuxAction::IncomingFrame`"
                    );
                    return Ok(());
                };

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

                match &frame.inner {
                    YamuxFrameInner::Data(_) => {
                        if let Some(stream) = yamux_state.streams.get_mut(&frame.stream_id) {
                            // must not underflow
                            // TODO: check it and disconnect peer that violates flow rules
                            stream.window_ours =
                                stream.window_ours.saturating_sub(frame.len_as_u32());
                        }
                    }
                    YamuxFrameInner::WindowUpdate { difference } => {
                        let stream = yamux_state
                            .streams
                            .entry(frame.stream_id)
                            .or_insert_with(YamuxStreamState::incoming);

                        stream.window_theirs = stream.window_theirs.saturating_add(*difference);

                        if *difference > 0 {
                            // have some fresh space in the window
                            // try send as many frames as can
                            let mut window = stream.window_theirs;
                            while let Some(frame) = stream.pending.pop_front() {
                                let len = frame.len_as_u32();
                                pending_outgoing.push_back(frame);
                                if let Some(new_window) = window.checked_sub(len) {
                                    window = new_window;
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    YamuxFrameInner::Ping { .. } => {}
                    YamuxFrameInner::GoAway(res) => yamux_state.set_res(*res),
                }

                let (dispatcher, state) = state_context.into_dispatcher_and_state();
                let limits: &P2pLimits = state.substate()?;
                let connection_state =
                    <State as SubstateAccess<P2pNetworkSchedulerState>>::substate(state)?
                        .connection_state(&addr)
                        .ok_or_else(|| format!("Connection not found {}", addr))?;

                let stream = connection_state.get_yamux_stream(frame.stream_id)
                    .ok_or_else(|| format!("Stream with id {} not found for `P2pNetworkYamuxAction::IncomingFrame`", frame.stream_id))?;

                let Some(peer_id) = connection_state.peer_id().cloned() else {
                    return Ok(());
                };

                // connection was reset by the peer
                if frame.flags.contains(YamuxFlags::RST) {
                    dispatcher.push(P2pNetworkSchedulerAction::Error {
                        addr,
                        error: P2pNetworkConnectionError::StreamReset(frame.stream_id),
                    });
                    return Ok(());
                }

                // if the peer tries to open more streams than allowed, close the stream
                if frame.flags.contains(YamuxFlags::SYN) && frame.stream_id != 0 {
                    if limits.max_streams() >= connection_state.incoming_streams_count() {
                        dispatcher.push(P2pNetworkSelectAction::Init {
                            addr,
                            kind: SelectKind::Stream(peer_id, frame.stream_id),
                            incoming: true,
                        });
                    } else {
                        dispatcher.push(P2pNetworkYamuxAction::OutgoingFrame {
                            addr,
                            frame: YamuxFrame {
                                flags: YamuxFlags::RST,
                                stream_id: frame.stream_id,
                                inner: YamuxFrameInner::Data(vec![].into()),
                            },
                        });
                    }
                }

                match &frame.inner {
                    YamuxFrameInner::Data(data) => {
                        // when our window size is less than half of the max window size send window update
                        if stream.window_ours < stream.max_window_size / 2 {
                            let difference =
                                stream.max_window_size.saturating_mul(2).min(1024 * 1024);

                            dispatcher.push(P2pNetworkYamuxAction::OutgoingFrame {
                                addr,
                                frame: YamuxFrame {
                                    stream_id: frame.stream_id,
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
                    YamuxFrameInner::WindowUpdate { .. } => {
                        while let Some(frame) = pending_outgoing.pop_front() {
                            dispatcher.push(P2pNetworkYamuxAction::OutgoingFrame { addr, frame });
                        }
                    }
                    _ => {}
                }

                Ok(())
            }
            P2pNetworkYamuxAction::OutgoingFrame { mut frame, addr } => {
                let stream_id = frame.stream_id;
                let Some(stream) = yamux_state.streams.get_mut(&stream_id) else {
                    return Ok(());
                };
                match &mut frame.inner {
                    YamuxFrameInner::Data(_) => {
                        if let Some(new_window) =
                            stream.window_theirs.checked_sub(frame.len_as_u32())
                        {
                            // their window is big enough, decrease the size
                            // and send the whole frame
                            stream.window_theirs = new_window;
                        } else {
                            // their window is not big enough
                            // split the frame to send as much as you can and put the rest in the queue
                            if let Some(remaining) = frame.split_at(stream.window_theirs as usize) {
                                stream.pending.push_front(remaining);
                            }

                            // the window will be zero after sending
                            stream.window_theirs = 0;

                            // if size of pending that is above the limit, ignore the peer
                            if stream.pending.iter().map(YamuxFrame::len).sum::<usize>()
                                > yamux_state.pending_outgoing_limit
                            {
                                let dispatcher = state_context.into_dispatcher();
                                let error = P2pNetworkConnectionError::YamuxOverflow(stream_id);
                                dispatcher.push(P2pNetworkSchedulerAction::Error { addr, error });
                                return Ok(());
                            }
                        }
                    }
                    YamuxFrameInner::WindowUpdate { difference } => {
                        stream.window_ours = stream.window_ours.saturating_add(*difference);
                        if stream.window_ours > stream.max_window_size {
                            stream.max_window_size = stream.window_ours.min(MAX_WINDOW_SIZE);
                        }
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
