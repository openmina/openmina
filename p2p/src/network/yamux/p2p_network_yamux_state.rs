use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};

use super::super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct P2pNetworkYamuxState {
    pub message_size_limit: Limit<usize>,
    pub buffer: Vec<u8>,
    pub incoming: VecDeque<YamuxFrame>,
    pub streams: BTreeMap<StreamId, YamuxStreamState>,
    pub terminated: Option<Result<Result<(), YamuxSessionError>, YamuxFrameParseError>>,
    pub init: bool,
}

impl P2pNetworkYamuxState {
    /// Calculates and returns the next available stream ID for outgoing
    /// communication.
    pub fn next_stream_id(&self, kind: YamuxStreamKind, incoming: bool) -> Option<StreamId> {
        if self.init && self.terminated.is_none() {
            Some(kind.stream_id(incoming))
        } else {
            None
        }
    }

    pub fn consume(&mut self, len: usize) {
        // does not need to do anything;
        // we will update the stream window later when we process the `IncomingData' action
        let _ = len;
    }

    pub fn limit(&self) -> usize {
        const SIZE_OF_HEADER: usize = 12;
        let headers = self.streams.len() * 2 + 1;

        let windows = self
            .streams
            .values()
            .map(|s| s.window_ours as usize)
            .sum::<usize>();

        windows + headers * SIZE_OF_HEADER
    }
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

bitflags::bitflags! {
    #[derive(Serialize, Deserialize, Debug, Default, Clone)]
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
    /// Unknown version.
    Version(u8),
    /// Unknown flags.
    Flags(u16),
    /// Unknown type.
    Type(u8),
    /// Unknown error code.
    ErrorCode(u32),
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

#[derive(Debug)]
pub enum YamuxStreamKind {
    Rpc,
    Gossipsub,
    Kademlia,
    Identify,
}

impl YamuxStreamKind {
    pub fn stream_id(self, incoming: bool) -> StreamId {
        (self as StreamId) * 2 + 1 + (incoming as StreamId)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn yamux_stream_id() {
        use super::YamuxStreamKind::*;
        assert_eq!(Rpc.stream_id(false), 1);
        assert_eq!(Rpc.stream_id(true), 2);
        assert_eq!(Kademlia.stream_id(false), 5);
        assert_eq!(Kademlia.stream_id(true), 6);
    }
}
