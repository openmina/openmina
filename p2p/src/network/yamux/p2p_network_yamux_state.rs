use std::collections::VecDeque;

use redux::ActionMeta;
use serde::{Deserialize, Serialize};

use super::{super::*, *};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pNetworkYamuxState {
    pub buffer: Vec<u8>,
    pub incoming: VecDeque<YamuxFrame>,

    pub window: i32,

    pub terminated: Option<Result<Result<(), YamuxSessionError>, YamuxFrameParseError>>,
}

impl Default for P2pNetworkYamuxState {
    fn default() -> Self {
        P2pNetworkYamuxState {
            buffer: Vec::default(),
            incoming: VecDeque::default(),
            window: 1 << 18,
            terminated: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum YamuxFrameParseError {
    UnknownVersion(u8),
    UnknownType(u8),
    UnknownErrorCode(u32),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct YamuxFrame {
    pub flags: u16,
    pub stream_id: u32,
    pub inner: YamuxFrameInner,
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
    {
        let state = store.state();
        let Some(state) = state.network.connection.connections.get(&self.addr()) else {
            return;
        };
        let Some(P2pNetworkConnectionMuxState::Yamux(state)) = &state.mux else {
            return;
        };

        let incoming = state.incoming.front().cloned().map(Into::into);

        if let Some(frame) = incoming {
            store.dispatch(P2pNetworkYamuxIncomingFrameAction {
                addr: self.addr(),
                frame,
            });
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

    pub fn reducer(&mut self, action: redux::ActionWithMeta<&P2pNetworkYamuxAction>) {
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
            P2pNetworkYamuxAction::OutgoingData(a) => {
                let _ = a;
                unimplemented!()
            }
            P2pNetworkYamuxAction::IncomingFrame(_) => {
                if let Some(frame) = self.incoming.pop_front() {
                    match frame.inner {
                        YamuxFrameInner::Data(data) => {
                            //
                            let _ = data;
                        }
                        YamuxFrameInner::WindowUpdate { difference } => {
                            self.window += difference;
                        }
                        YamuxFrameInner::Ping { opaque } => {
                            // TODO: ping
                            let _ = opaque;
                        }
                        YamuxFrameInner::GoAway(res) => self.set_res(res),
                    }
                }
            }
        }
    }
}
