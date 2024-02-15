use std::collections::{BTreeMap, VecDeque};

use redux::ActionMeta;
use serde::{Deserialize, Serialize};

use super::{super::*, *};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkYamuxState {
    pub buffer: Vec<u8>,
    pub incoming: VecDeque<YamuxFrame>,
    pub streams: BTreeMap<StreamId, YamuxStreamState>,
    pub terminated: Option<Result<Result<(), YamuxSessionError>, YamuxFrameParseError>>,
    pub init: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct YamuxStreamState {
    pub incoming: bool,
    pub syn_sent: bool,
    pub established: bool,
    pub readable: bool,
    pub writable: bool,
    pub window_theirs: u32,
    pub window_ours: u32,
}

impl YamuxStreamState {
    fn update_window(&mut self, ours: bool, difference: i32) {
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

impl Default for YamuxStreamState {
    fn default() -> Self {
        YamuxStreamState {
            incoming: false,
            syn_sent: false,
            established: false,
            readable: false,
            writable: false,
            window_theirs: 256 * 1024,
            window_ours: 256 * 1024,
        }
    }
}

impl YamuxStreamState {
    pub fn incoming() -> Self {
        YamuxStreamState {
            incoming: true,
            ..Default::default()
        }
    }
}

impl Default for P2pNetworkYamuxState {
    fn default() -> Self {
        P2pNetworkYamuxState {
            buffer: Vec::default(),
            incoming: VecDeque::default(),
            streams: BTreeMap::default(),
            terminated: None,
            init: false,
        }
    }
}

bitflags::bitflags! {
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct YamuxFlags: u16 {
        const SYN = 0b0001;
        const ACK = 0b0010;
        const FIN = 0b0100;
        const RST = 0b1000;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct YamuxPing {
    pub stream_id: StreamId,
    pub opaque: i32,
    pub response: bool,
}

impl YamuxPing {
    pub fn into_frame(self) -> YamuxFrame {
        let YamuxPing {
            stream_id,
            opaque,
            response,
        } = self;
        YamuxFrame {
            flags: if response {
                YamuxFlags::ACK
            } else if stream_id == 0 {
                YamuxFlags::SYN
            } else {
                YamuxFlags::empty()
            },
            stream_id,
            inner: YamuxFrameInner::Ping { opaque },
        }
    }
}

pub type StreamId = u32;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum YamuxFrameParseError {
    UnknownVersion(u8),
    UnknownFlags(u16),
    UnknownType(u8),
    UnknownErrorCode(u32),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct YamuxFrame {
    pub flags: YamuxFlags,
    pub stream_id: StreamId,
    pub inner: YamuxFrameInner,
}

impl YamuxFrame {
    pub fn into_bytes(self) -> Vec<u8> {
        let data_len = if let YamuxFrameInner::Data(data) = &self.inner {
            data.len()
        } else {
            0
        };
        let mut vec = Vec::with_capacity(12 + data_len);
        vec.push(0);
        match self.inner {
            YamuxFrameInner::Data(data) => {
                vec.push(0);
                vec.extend_from_slice(&self.flags.bits().to_be_bytes());
                vec.extend_from_slice(&self.stream_id.to_be_bytes());
                vec.extend_from_slice(&(data.len() as u32).to_be_bytes());
                vec.extend_from_slice(&data);
            }
            YamuxFrameInner::WindowUpdate { difference } => {
                vec.push(1);
                vec.extend_from_slice(&self.flags.bits().to_be_bytes());
                vec.extend_from_slice(&self.stream_id.to_be_bytes());
                vec.extend_from_slice(&difference.to_be_bytes());
            }
            YamuxFrameInner::Ping { opaque } => {
                vec.push(2);
                vec.extend_from_slice(&self.flags.bits().to_be_bytes());
                vec.extend_from_slice(&self.stream_id.to_be_bytes());
                vec.extend_from_slice(&opaque.to_be_bytes());
            }
            YamuxFrameInner::GoAway(res) => {
                vec.push(3);
                vec.extend_from_slice(&self.flags.bits().to_be_bytes());
                vec.extend_from_slice(&self.stream_id.to_be_bytes());
                let code = match res {
                    Ok(()) => 0u32,
                    Err(YamuxSessionError::Protocol) => 1,
                    Err(YamuxSessionError::Internal) => 2,
                };
                vec.extend_from_slice(&code.to_be_bytes());
            }
        }

        vec
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum YamuxFrameInner {
    Data(Data),
    WindowUpdate { difference: i32 },
    Ping { opaque: i32 },
    GoAway(Result<(), YamuxSessionError>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum YamuxSessionError {
    Protocol,
    Internal,
}

impl P2pNetworkYamuxAction {
    pub fn effects<Store, S>(&self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        P2pNetworkYamuxIncomingFrameAction: redux::EnablingCondition<S>,
        P2pNetworkNoiseOutgoingDataAction: redux::EnablingCondition<S>,
        P2pNetworkSelectInitAction: redux::EnablingCondition<S>,
        P2pNetworkSelectIncomingDataAction: redux::EnablingCondition<S>,
        P2pNetworkYamuxOutgoingFrameAction: redux::EnablingCondition<S>,
        P2pNetworkYamuxPingStreamAction: redux::EnablingCondition<S>,
        P2pNetworkSchedulerYamuxDidInitAction: redux::EnablingCondition<S>,
    {
        let state = store.state();
        let Some(state) = state.network.scheduler.connections.get(&self.addr()) else {
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
        let init = state.init;

        match self {
            Self::IncomingData(a) => {
                if let Some(frame) = incoming {
                    store.dispatch(P2pNetworkYamuxIncomingFrameAction {
                        addr: a.addr,
                        frame,
                    });
                }
            }
            Self::IncomingFrame(a) => {
                let frame = &a.frame;
                let Some(stream) = state.streams.get(&frame.stream_id).cloned() else {
                    return;
                };

                if frame.flags.contains(YamuxFlags::SYN) && frame.stream_id != 0 {
                    store.dispatch(P2pNetworkSelectInitAction {
                        addr: a.addr,
                        kind: SelectKind::Stream(peer_id, frame.stream_id),
                        incoming: true,
                        send_handshake: true,
                    });
                }
                match &frame.inner {
                    YamuxFrameInner::Data(data) => {
                        if stream.window_ours < 64 * 1024 {
                            store.dispatch(P2pNetworkYamuxOutgoingFrameAction {
                                addr: a.addr,
                                frame: YamuxFrame {
                                    stream_id: frame.stream_id,
                                    flags: YamuxFlags::empty(),
                                    inner: YamuxFrameInner::WindowUpdate {
                                        difference: 256 * 1024,
                                    },
                                },
                            });
                        }

                        store.dispatch(P2pNetworkSelectIncomingDataAction {
                            addr: a.addr,
                            kind: SelectKind::Stream(peer_id, frame.stream_id),
                            data: data.clone(),
                            fin: a.frame.flags.contains(YamuxFlags::FIN),
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
                            store.dispatch(P2pNetworkYamuxOutgoingFrameAction {
                                addr: a.addr,
                                frame: ping.clone().into_frame(),
                            });
                        }
                        if !init {
                            store.dispatch(P2pNetworkSchedulerYamuxDidInitAction { addr: a.addr });
                        }
                    }
                    _ => {}
                }

                if let Some(frame) = incoming {
                    store.dispatch(P2pNetworkYamuxIncomingFrameAction {
                        addr: a.addr,
                        frame,
                    });
                }
            }
            Self::OutgoingFrame(a) => {
                store.dispatch(P2pNetworkNoiseOutgoingDataAction {
                    addr: a.addr,
                    data: a.frame.clone().into_bytes().into(),
                });
            }
            Self::OutgoingData(a) => {
                let Some(stream) = state.streams.get(&a.stream_id) else {
                    return;
                };
                let mut flags = YamuxFlags::empty();
                if !stream.incoming && !stream.established && !stream.syn_sent {
                    flags.insert(YamuxFlags::SYN);
                } else if stream.incoming && !stream.established {
                    flags.insert(YamuxFlags::ACK);
                }
                if a.fin {
                    flags.insert(YamuxFlags::FIN);
                }
                let frame = YamuxFrame {
                    flags,
                    stream_id: a.stream_id,
                    inner: YamuxFrameInner::Data(a.data.clone()),
                };
                store.dispatch(P2pNetworkYamuxOutgoingFrameAction {
                    addr: a.addr,
                    frame,
                });
            }
            Self::PingStream(a) => {
                store.dispatch(P2pNetworkYamuxOutgoingFrameAction {
                    addr: a.addr,
                    frame: a.ping.clone().into_frame(),
                });
            }
            Self::OpenStream(a) => {
                store.dispatch(P2pNetworkSelectInitAction {
                    addr: a.addr,
                    kind: SelectKind::Stream(peer_id, a.stream_id),
                    incoming: false,
                    send_handshake: true,
                });
            }
        }
    }
}

impl P2pNetworkYamuxState {
    pub fn set_err(&mut self, err: YamuxFrameParseError) {
        self.terminated = Some(Err(err));
    }

    pub fn set_res(&mut self, res: Result<(), YamuxSessionError>) {
        self.terminated = Some(Ok(res));
    }

    pub fn next_stream_id(&self) -> Option<StreamId> {
        if self.init && self.terminated.is_none() {
            Some(self.streams.keys().max().map_or(1, |id| (id + 1) / 2 * 2 + 1))
        } else {
            None
        }
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
                                self.set_err(YamuxFrameParseError::UnknownVersion(unknown));
                                break;
                            }
                        };
                        let flags = u16::from_be_bytes(buf[2..4].try_into().expect("cannot fail"));
                        let Some(flags) = YamuxFlags::from_bits(flags) else {
                            self.set_err(YamuxFrameParseError::UnknownFlags(flags));
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
                                        self.set_err(YamuxFrameParseError::UnknownErrorCode(
                                            unknown,
                                        ));
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
                                self.set_err(YamuxFrameParseError::UnknownType(unknown));
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
                                .or_insert_with(|| YamuxStreamState::incoming())
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
