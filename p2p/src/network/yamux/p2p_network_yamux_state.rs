use std::collections::{BTreeMap, VecDeque};

use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};

use super::super::*;

pub const INITIAL_RECV_BUFFER_CAPACITY: usize = 0x40000; // 256kb
pub const INITIAL_WINDOW_SIZE: u32 = INITIAL_RECV_BUFFER_CAPACITY as u32;
pub const MAX_WINDOW_SIZE: u32 = INITIAL_RECV_BUFFER_CAPACITY as u32;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct P2pNetworkYamuxState {
    pub message_size_limit: Limit<usize>,
    pub pending_outgoing_limit: Limit<usize>,
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

    pub fn set_err(&mut self, err: YamuxFrameParseError) {
        self.terminated = Some(Err(err));
    }

    pub fn set_res(&mut self, res: Result<(), YamuxSessionError>) {
        self.terminated = Some(Ok(res));
    }

    /// Attempts to parse a Yamux frame from the buffer starting at the given offset.
    /// Returns the number of bytes consumed if a frame was successfully parsed.
    pub fn try_parse_frame(&mut self, offset: usize) -> Option<usize> {
        let buf = &self.buffer[offset..];
        if buf.len() < 12 {
            return None;
        }

        // Version 0 is the only supported version as per Yamux specification.
        // Any other version should be rejected.
        let _version = match buf[0] {
            0 => 0,
            unknown => {
                self.set_err(YamuxFrameParseError::Version(unknown));
                return None;
            }
        };

        let flags = u16::from_be_bytes(buf[2..4].try_into().expect("cannot fail"));
        let Some(flags) = YamuxFlags::from_bits(flags) else {
            self.set_err(YamuxFrameParseError::Flags(flags));
            return None;
        };
        let stream_id = u32::from_be_bytes(buf[4..8].try_into().expect("cannot fail"));
        let b = buf[8..12].try_into().expect("cannot fail");

        match buf[1] {
            // Data frame - contains actual payload data for the stream
            0 => {
                let len = u32::from_be_bytes(b) as usize;
                if len > self.message_size_limit {
                    self.set_res(Err(YamuxSessionError::Internal));
                    return None;
                }
                if buf.len() >= 12 + len {
                    let frame = YamuxFrame {
                        flags,
                        stream_id,
                        inner: YamuxFrameInner::Data(buf[12..(12 + len)].to_vec().into()),
                    };
                    self.incoming.push_back(frame);
                    Some(12 + len)
                } else {
                    None
                }
            }
            // Window Update frame - used for flow control, updates available window size
            1 => {
                let difference = u32::from_be_bytes(b);
                let frame = YamuxFrame {
                    flags,
                    stream_id,
                    inner: YamuxFrameInner::WindowUpdate { difference },
                };
                self.incoming.push_back(frame);
                Some(12)
            }
            // Ping frame - used for keepalive and round-trip time measurements
            2 => {
                let opaque = u32::from_be_bytes(b);
                let frame = YamuxFrame {
                    flags,
                    stream_id,
                    inner: YamuxFrameInner::Ping { opaque },
                };
                self.incoming.push_back(frame);
                Some(12)
            }
            // GoAway frame - signals session termination with optional error code
            3 => {
                let code = u32::from_be_bytes(b);
                let result = match code {
                    0 => Ok(()),                           // Normal termination
                    1 => Err(YamuxSessionError::Protocol), // Protocol error
                    2 => Err(YamuxSessionError::Internal), // Internal error
                    unknown => {
                        self.set_err(YamuxFrameParseError::ErrorCode(unknown));
                        return None;
                    }
                };
                let frame = YamuxFrame {
                    flags,
                    stream_id,
                    inner: YamuxFrameInner::GoAway(result),
                };
                self.incoming.push_back(frame);
                Some(12)
            }
            // Unknown frame type
            unknown => {
                self.set_err(YamuxFrameParseError::Type(unknown));
                None
            }
        }
    }

    /// Attempts to parse all available complete frames from the buffer,
    /// then shifts and compacts the buffer as needed.
    pub fn parse_frames(&mut self) {
        let mut offset = 0;
        while let Some(consumed) = self.try_parse_frame(offset) {
            offset += consumed;
        }
        self.shift_and_compact_buffer(offset);
    }

    pub(crate) fn shift_and_compact_buffer(&mut self, offset: usize) {
        let new_len = self.buffer.len() - offset;
        if self.buffer.capacity() > INITIAL_RECV_BUFFER_CAPACITY * 2
            && new_len < INITIAL_RECV_BUFFER_CAPACITY / 2
        {
            let old_buffer = &self.buffer;
            let mut new_buffer = Vec::with_capacity(INITIAL_RECV_BUFFER_CAPACITY);
            new_buffer.extend_from_slice(&old_buffer[offset..]);
            self.buffer = new_buffer;
        } else {
            self.buffer.copy_within(offset.., 0);
            self.buffer.truncate(new_len);
        }
    }

    /// Extends the internal buffer with new data, ensuring it has appropriate capacity.
    /// On first use, reserves the initial capacity.
    pub fn extend_buffer(&mut self, data: &[u8]) {
        if self.buffer.capacity() == 0 {
            self.buffer.reserve(INITIAL_RECV_BUFFER_CAPACITY);
        }
        self.buffer.extend_from_slice(data);
    }

    /// Returns the number of incoming frames that have been parsed and are ready for processing.
    pub fn incoming_frame_count(&self) -> usize {
        self.incoming.len()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, MallocSizeOf)]
pub struct YamuxStreamState {
    pub incoming: bool,
    pub syn_sent: bool,
    pub established: bool,
    pub readable: bool,
    pub writable: bool,
    pub window_theirs: u32,
    pub window_ours: u32,
    pub max_window_size: u32,
    pub pending: VecDeque<YamuxFrame>,
}

impl Default for YamuxStreamState {
    fn default() -> Self {
        YamuxStreamState {
            incoming: false,
            syn_sent: false,
            established: false,
            readable: false,
            writable: false,
            window_theirs: INITIAL_WINDOW_SIZE,
            window_ours: INITIAL_WINDOW_SIZE,
            max_window_size: INITIAL_WINDOW_SIZE,
            pending: VecDeque::default(),
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

    /// Updates the remote window size and returns any frames that can now be sent
    /// Returns frames that were pending and can now be sent due to increased window size
    pub fn update_remote_window(&mut self, difference: u32) -> VecDeque<YamuxFrame> {
        self.window_theirs = self.window_theirs.saturating_add(difference);
        let mut sendable_frames = VecDeque::new();

        if difference > 0 {
            let mut available_window = self.window_theirs;
            while let Some(frame) = self.pending.pop_front() {
                let frame_len = frame.len_as_u32();
                if frame_len > available_window {
                    // Put frame back and stop
                    self.pending.push_front(frame);
                    break;
                }
                available_window -= frame_len;
                sendable_frames.push_back(frame);
            }
        }

        sendable_frames
    }

    /// Updates the local window size and possibly increases max window size
    pub fn update_local_window(&mut self, difference: u32) {
        self.window_ours = self.window_ours.saturating_add(difference);
        if self.window_ours > self.max_window_size {
            self.max_window_size = self.window_ours.min(MAX_WINDOW_SIZE);
        }
    }

    /// Consumes window space for outgoing data
    /// Returns true if the frame can be sent immediately,
    /// false if it needs to be queued (window too small)
    pub fn try_consume_window(&mut self, frame_len: u32) -> bool {
        if let Some(new_window) = self.window_theirs.checked_sub(frame_len) {
            self.window_theirs = new_window;
            true
        } else {
            false
        }
    }

    /// Checks if window should be updated based on current size
    /// Returns the amount by which the window should be increased, if any
    pub fn should_update_window(&self) -> Option<u32> {
        if self.window_ours < self.max_window_size / 2 {
            Some(self.max_window_size.saturating_mul(2).min(1024 * 1024))
        } else {
            None
        }
    }
}

bitflags::bitflags! {
    #[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
    pub struct YamuxFlags: u16 {
        /// Signals the start of a new stream. May be sent with a data or window update message. Also sent with a ping to indicate outbound.
        const SYN = 0b0001;
        /// Acknowledges the start of a new stream. May be sent with a data or window update message. Also sent with a ping to indicate response.
        const ACK = 0b0010;
        /// Performs a half-close of a stream. May be sent with a data message or window update.
        const FIN = 0b0100;
        /// Reset a stream immediately. May be sent with a data or window update message.
        const RST = 0b1000;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct YamuxPing {
    pub stream_id: StreamId,
    pub opaque: u32,
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

#[derive(Serialize, Deserialize, Debug, Clone, MallocSizeOf)]
pub struct YamuxFrame {
    #[ignore_malloc_size_of = "doesn't allocate"]
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

    pub fn len(&self) -> usize {
        if let YamuxFrameInner::Data(data) = &self.inner {
            data.len()
        } else {
            0
        }
    }

    // When we parse the frame we parse length as u32 and so `data.len()` should always be representable as u32
    pub fn len_as_u32(&self) -> u32 {
        if let YamuxFrameInner::Data(data) = &self.inner {
            u32::try_from(data.len()).unwrap_or(u32::MAX)
        } else {
            0
        }
    }

    /// If this data is bigger then `pos`, keep only first `pos` bytes and return some remaining
    /// otherwise return none
    pub fn split_at(&mut self, pos: usize) -> Option<Self> {
        use std::ops::Sub;

        if let YamuxFrameInner::Data(data) = &mut self.inner {
            if data.len() <= pos {
                return None;
            }
            let (keep, rest) = data.split_at(pos);
            let rest = Data(rest.to_vec().into_boxed_slice());
            *data = Data(keep.to_vec().into_boxed_slice());

            let fin = if self.flags.contains(YamuxFlags::FIN) {
                self.flags.remove(YamuxFlags::FIN);
                YamuxFlags::FIN
            } else {
                YamuxFlags::empty()
            };
            Some(YamuxFrame {
                flags: self.flags.sub(YamuxFlags::SYN | YamuxFlags::ACK) | fin,
                stream_id: self.stream_id,
                inner: YamuxFrameInner::Data(rest),
            })
        } else {
            None
        }
    }

    pub fn is_session_stream(&self) -> bool {
        self.stream_id == 0
    }

    pub fn kind(&self) -> YamuxFrameKind {
        match self.inner {
            YamuxFrameInner::Data(_) => YamuxFrameKind::Data,
            YamuxFrameInner::WindowUpdate { .. } => YamuxFrameKind::WindowUpdate,
            YamuxFrameInner::Ping { .. } => YamuxFrameKind::Ping,
            YamuxFrameInner::GoAway(_) => YamuxFrameKind::GoAway,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum YamuxFrameKind {
    Data,
    WindowUpdate,
    Ping,
    GoAway,
}

#[derive(Serialize, Deserialize, Debug, Clone, MallocSizeOf)]
pub enum YamuxFrameInner {
    Data(Data),
    WindowUpdate { difference: u32 },
    Ping { opaque: u32 },
    GoAway(#[ignore_malloc_size_of = "doesn't allocate"] Result<(), YamuxSessionError>),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
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

mod measurement {
    use std::mem;

    use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};

    use super::{P2pNetworkYamuxState, YamuxFrame};

    impl MallocSizeOf for P2pNetworkYamuxState {
        fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
            self.buffer.capacity()
                + self.incoming.capacity() * mem::size_of::<YamuxFrame>()
                + self
                    .incoming
                    .iter()
                    .map(|frame| frame.size_of(ops))
                    .sum::<usize>()
                + self
                    .streams
                    .iter()
                    .map(|(k, v)| mem::size_of_val(k) + mem::size_of_val(v) + v.size_of(ops))
                    .sum::<usize>()
        }
    }
}
