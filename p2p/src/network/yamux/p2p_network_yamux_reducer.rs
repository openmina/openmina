use std::collections::BTreeMap;

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

    pub fn reducer(
        &mut self,
        streams: &mut BTreeMap<StreamId, P2pNetworkStreamState>,
        action: redux::ActionWithMeta<&P2pNetworkYamuxAction>,
    ) {
        if self.terminated.is_some() {
            return;
        }

        match action.action() {
            P2pNetworkYamuxAction::IncomingData(a) => {
                self.buffer.extend_from_slice(&a.data);
                let mut offset = 0;
                loop {
                    let buf = &self.buffer[offset..];
                    if buf.len() >= 12 {
                        let _version = match buf[0] {
                            0 => 0,
                            unknown => {
                                self.set_err(YamuxFrameParseError::Version(unknown));
                                break;
                            }
                        };
                        let flags = u16::from_be_bytes(buf[2..4].try_into().expect("cannot fail"));
                        let Some(flags) = YamuxFlags::from_bits(flags) else {
                            self.set_err(YamuxFrameParseError::Flags(flags));
                            break;
                        };
                        let stream_id =
                            u32::from_be_bytes(buf[4..8].try_into().expect("cannot fail"));
                        let b = buf[8..12].try_into().expect("cannot fail");

                        match buf[1] {
                            0 => {
                                let len = u32::from_be_bytes(b) as usize;
                                if buf.len() >= 12 + len {
                                    let frame = YamuxFrame {
                                        flags,
                                        stream_id,
                                        inner: YamuxFrameInner::Data(
                                            buf[12..(12 + len)].to_vec().into(),
                                        ),
                                    };
                                    self.incoming.push_back(frame);
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
                                self.incoming.push_back(frame);
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
                                self.incoming.push_back(frame);
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
                                        self.set_err(YamuxFrameParseError::ErrorCode(unknown));
                                        break;
                                    }
                                };
                                let frame = YamuxFrame {
                                    flags,
                                    stream_id,
                                    inner: YamuxFrameInner::GoAway(result),
                                };
                                self.incoming.push_back(frame);
                                offset += 12;
                                continue;
                            }
                            unknown => {
                                self.set_err(YamuxFrameParseError::Type(unknown));
                                break;
                            }
                        }
                    }

                    break;
                }

                self.buffer = self.buffer[offset..].to_vec();
            }
            P2pNetworkYamuxAction::OutgoingData(_) => {}
            P2pNetworkYamuxAction::IncomingFrame(_) => {
                if let Some(frame) = self.incoming.pop_front() {
                    if frame.flags.contains(YamuxFlags::SYN) {
                        self.streams
                            .insert(frame.stream_id, YamuxStreamState::incoming());
                        streams.insert(frame.stream_id, P2pNetworkStreamState::new_incoming());
                    }
                    if frame.flags.contains(YamuxFlags::ACK) {
                        self.streams.entry(frame.stream_id).or_default().established = true;
                    }

                    match frame.inner {
                        YamuxFrameInner::Data(data) => {
                            if let Some(stream) = self.streams.get_mut(&frame.stream_id) {
                                // must not underflow
                                // TODO: check it and disconnect peer that violates flow rules
                                stream.window_ours -= data.len() as u32;
                            }
                        }
                        YamuxFrameInner::WindowUpdate { difference } => {
                            self.streams
                                .entry(frame.stream_id)
                                .or_insert_with(YamuxStreamState::incoming)
                                .update_window(false, difference);
                        }
                        YamuxFrameInner::Ping { .. } => {}
                        YamuxFrameInner::GoAway(res) => self.set_res(res),
                    }
                }
            }
            P2pNetworkYamuxAction::OutgoingFrame(a) => {
                let frame = &a.frame;

                let Some(stream) = self.streams.get_mut(&frame.stream_id) else {
                    return;
                };
                match &frame.inner {
                    YamuxFrameInner::Data(data) => {
                        // must not underflow
                        // the action must not dispatch if it doesn't fit in the window
                        // TODO: add pending queue, where frames will wait for window increase
                        stream.window_theirs -= data.len() as u32;
                    }
                    YamuxFrameInner::WindowUpdate { difference } => {
                        stream.update_window(true, *difference);
                    }
                    _ => {}
                }

                if frame.flags.contains(YamuxFlags::FIN) {
                    streams.remove(&frame.stream_id);
                    stream.writable = false;
                } else {
                    if frame.flags.contains(YamuxFlags::ACK) {
                        stream.established |= true;
                    }
                    if frame.flags.contains(YamuxFlags::SYN) {
                        stream.syn_sent = true;
                    }
                }
            }
            P2pNetworkYamuxAction::PingStream(_) => {}
            P2pNetworkYamuxAction::OpenStream(a) => {
                self.streams
                    .insert(a.stream_id, YamuxStreamState::default());
                streams.insert(a.stream_id, P2pNetworkStreamState::new(a.stream_kind));
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
                *window -= decreasing;
            }
        } else {
            let increasing = difference as u32;
            *window += increasing;
        }
    }
}
