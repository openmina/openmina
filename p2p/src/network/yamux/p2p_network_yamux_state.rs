use std::collections::{BTreeMap, VecDeque};

use redux::ActionMeta;
use serde::{Deserialize, Serialize};

use super::{super::*, *};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkYamuxState {
    pub buffer: Vec<u8>,
    pub incoming: VecDeque<YamuxFrame>,

    pub terminated: Option<Result<Result<(), YamuxSessionError>, YamuxFrameParseError>>,
}

impl Default for P2pNetworkYamuxState {
    fn default() -> Self {
        P2pNetworkYamuxState {
            buffer: Vec::default(),
            incoming: VecDeque::default(),
            terminated: None,
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
        let streams = &state.streams;
        let Some(P2pNetworkConnectionMuxState::Yamux(state)) = &state.mux else {
            return;
        };

        let incoming = state.incoming.front().cloned().map(Into::into);

        match self {
            Self::IncomingData(_) => {
                if let Some(frame) = incoming {
                    store.dispatch(P2pNetworkYamuxIncomingFrameAction {
                        addr: self.addr(),
                        frame,
                    });
                }
            }
            Self::IncomingFrame(a) => {
                let frame = &a.frame;
                match &frame.inner {
                    YamuxFrameInner::Data(data) => {
                        store.dispatch(P2pNetworkSelectIncomingDataAction {
                            addr: self.addr(),
                            kind: SelectKind::Stream(peer_id, frame.stream_id),
                            data: data.clone(),
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
                            store.dispatch(P2pNetworkNoiseOutgoingDataAction {
                                addr: self.addr(),
                                data: ping.clone().into_frame().into_bytes().into(),
                            });
                        }
                    }
                    _ => {}
                }
                if frame.flags.contains(YamuxFlags::SYN) && frame.stream_id != 0 {
                    store.dispatch(P2pNetworkSelectInitAction {
                        addr: self.addr(),
                        kind: SelectKind::Stream(peer_id, frame.stream_id),
                        incoming: true,
                    });
                }
            }
            Self::OutgoingData(a) => {
                let Some(stream) = streams.get(&a.stream_id) else {
                    return;
                };
                let frame = YamuxFrame {
                    flags: if stream.acked {
                        YamuxFlags::empty()
                    } else {
                        YamuxFlags::ACK
                    },
                    stream_id: a.stream_id,
                    inner: YamuxFrameInner::Data(a.data.clone()),
                };
                store.dispatch(P2pNetworkNoiseOutgoingDataAction {
                    addr: a.addr,
                    data: frame.into_bytes().into(),
                });
            }
            Self::PingStream(a) => {
                store.dispatch(P2pNetworkNoiseOutgoingDataAction {
                    addr: a.addr,
                    data: a.ping.clone().into_frame().into_bytes().into(),
                });
            }
            Self::OpenStream(a) => {
                let frame = YamuxFrame {
                    flags: YamuxFlags::SYN,
                    stream_id: a.stream_id,
                    inner: YamuxFrameInner::WindowUpdate { difference: 0 },
                };
                store.dispatch(P2pNetworkNoiseOutgoingDataAction {
                    addr: a.addr,
                    data: frame.into_bytes().into(),
                });
            }
            Self::SendStream(a) => {
                let frame = YamuxFrame {
                    flags: YamuxFlags::empty(),
                    stream_id: a.stream_id,
                    inner: YamuxFrameInner::Data(a.data.clone()),
                };
                store.dispatch(P2pNetworkNoiseOutgoingDataAction {
                    addr: a.addr,
                    data: frame.into_bytes().into(),
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
                    match frame.inner {
                        YamuxFrameInner::Data(_) => {
                            if frame.flags.contains(YamuxFlags::SYN) {
                                streams
                                    .insert(frame.stream_id, P2pNetworkStreamState::new_incoming());
                            }
                        }
                        YamuxFrameInner::WindowUpdate { difference } => {
                            // TODO: action
                            let stream = streams
                                .entry(frame.stream_id)
                                .or_insert_with(|| P2pNetworkStreamState::new_incoming());
                            stream.window += difference;
                        }
                        YamuxFrameInner::Ping { .. } => {}
                        YamuxFrameInner::GoAway(res) => self.set_res(res),
                    }
                }
            }
            P2pNetworkYamuxAction::PingStream(_) => {}
            P2pNetworkYamuxAction::OpenStream(a) => {
                streams.insert(a.stream_id, P2pNetworkStreamState::new(a.stream_kind));
            }
            P2pNetworkYamuxAction::SendStream(_) => {}
        }
    }
}
