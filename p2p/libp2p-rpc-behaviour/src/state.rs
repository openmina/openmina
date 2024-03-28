use std::{
    collections::{BTreeSet, VecDeque},
    io,
    pin::Pin,
    sync::Arc,
    task::{self, Context, Poll},
};

use libp2p::futures::{AsyncRead, AsyncWrite};

use mina_p2p_messages::{
    binprot::{self, BinProtRead, BinProtWrite},
    rpc_kernel::RpcTag,
};

use mina_p2p_messages::{
    rpc::VersionedRpcMenuV1,
    rpc_kernel::{
        Message, MessageHeader, NeedsLength, Query, QueryHeader, Response, ResponseHeader,
        ResponsePayload, RpcMethod, RpcResult,
    },
};

#[derive(Debug)]
pub enum Received {
    Query {
        header: QueryHeader,
        bytes: Vec<u8>,
    },
    Response {
        header: ResponseHeader,
        bytes: Vec<u8>,
    },
    Menu(Vec<(String, u32)>),
    HandshakeDone,
    // SentConfirmation(i64),
}

pub struct Inner {
    menu: Arc<BTreeSet<(RpcTag, u32)>>,
    command_queue: VecDeque<(usize, Vec<u8>)>,
    buffer: Buffer,
    ask_menu: bool,
}

impl Inner {
    pub fn new(menu: Arc<BTreeSet<(RpcTag, u32)>>, ask_menu: bool) -> Self {
        Inner {
            menu,
            command_queue: {
                if ask_menu {
                    let msg = Message::<<VersionedRpcMenuV1 as RpcMethod>::Query>::Query(Query {
                        tag: <VersionedRpcMenuV1 as RpcMethod>::NAME.into(),
                        version: <VersionedRpcMenuV1 as RpcMethod>::VERSION,
                        id: 0,
                        data: NeedsLength(()),
                    });
                    let mut bytes = vec![0; 8];
                    msg.binprot_write(&mut bytes).expect("valid constant");
                    let len = (bytes.len() - 8) as u64;
                    bytes[..8].clone_from_slice(&len.to_le_bytes());
                    [(0, Self::HANDSHAKE_MSG.to_vec()), (0, bytes)]
                        .into_iter()
                        .collect()
                } else {
                    [(0, Self::HANDSHAKE_MSG.to_vec())].into_iter().collect()
                }
            },
            buffer: Buffer::default(),
            ask_menu,
        }
    }
}

struct Buffer {
    offset: usize,
    buf: Vec<u8>,
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer {
            offset: 0,
            buf: vec![0; Self::INITIAL_SIZE],
        }
    }
}

impl Buffer {
    const INITIAL_SIZE: usize = 0x1000;

    pub fn poll_fill<T>(&mut self, cx: &mut Context<'_>, io: &mut T) -> Poll<io::Result<usize>>
    where
        T: AsyncRead + Unpin,
    {
        loop {
            let read =
                task::ready!(Pin::new(&mut *io).poll_read(cx, &mut self.buf[self.offset..]))?;
            self.offset += read;
            if self.offset < self.buf.len() {
                return Poll::Ready(Ok(read));
            } else {
                self.buf.resize(2 * self.buf.len(), 0);
            }
        }
    }

    pub fn try_cut(&mut self) -> Option<Result<(MessageHeader, Vec<u8>), binprot::Error>> {
        if self.offset >= 8 {
            let msg_len = u64::from_le_bytes(
                self.buf[..8]
                    .try_into()
                    .expect("cannot fail, offset is >= 8"),
            ) as usize;
            if self.offset >= 8 + msg_len {
                self.offset -= 8 + msg_len;
                let mut all_bytes = &self.buf[8..(8 + msg_len)];
                let header = match MessageHeader::binprot_read(&mut all_bytes) {
                    Ok(v) => v,
                    Err(err) => return Some(Err(err)),
                };
                let bytes = all_bytes.to_vec();
                self.buf = self.buf[(8 + msg_len)..].to_vec();
                let new_len = self.buf.len().next_power_of_two().max(Self::INITIAL_SIZE);
                self.buf.resize(new_len, 0);
                return Some(Ok((header, bytes)));
            }
        }

        None
    }
}

impl Inner {
    const HANDSHAKE_MSG: [u8; 15] = *b"\x07\x00\x00\x00\x00\x00\x00\x00\x02\xfdRPC\x00\x01";

    pub fn add(&mut self, bytes: Vec<u8>) {
        self.command_queue.push_back((0, bytes));
    }

    pub fn poll<T>(&mut self, cx: &mut Context<'_>, io: &mut T) -> Poll<io::Result<Received>>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        let mut send_pending = false;
        let mut recv_pending = false;

        loop {
            if !send_pending && !self.command_queue.is_empty() {
                match self.poll_send(cx, io) {
                    Poll::Pending => send_pending = true,
                    Poll::Ready(r) => r?,
                }
            }

            if !recv_pending {
                match self.poll_recv(cx, io) {
                    Poll::Pending => {
                        recv_pending = true;
                        if self.command_queue.is_empty() {
                            return Poll::Pending;
                        }
                    }
                    Poll::Ready(r) => return Poll::Ready(r),
                }
            }

            if (send_pending || self.command_queue.is_empty()) && recv_pending {
                return Poll::Pending;
            }
        }
    }

    pub fn poll_recv<T>(
        &mut self,
        cx: &mut Context<'_>,
        mut io: &mut T,
    ) -> Poll<io::Result<Received>>
    where
        T: AsyncRead + Unpin,
    {
        let h_id = u64::from_le_bytes(*b"RPC\x00\x00\x00\x00\x00");
        while let Some(v) = self.buffer.try_cut() {
            // TODO: proper error type
            let (header, bytes) = v.map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
            match header {
                MessageHeader::Heartbeat => {
                    // TODO: handle heartbeat properly
                    self.add(b"\x01\x00\x00\x00\x00\x00\x00\x00\x00".to_vec());
                }
                MessageHeader::Response(ResponseHeader { id }) if id == h_id => {
                    return Poll::Ready(Ok(Received::HandshakeDone));
                }
                MessageHeader::Response(ResponseHeader { id }) if id == 0 && self.ask_menu => {
                    let mut bytes_slice = bytes.as_slice();
                    type P = ResponsePayload<<VersionedRpcMenuV1 as RpcMethod>::Response>;
                    let menu = P::binprot_read(&mut bytes_slice)
                        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?
                        .0
                        .ok()
                        .map(|NeedsLength(x)| x)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|(tag, version)| (tag.to_string_lossy(), version))
                        .collect();
                    return Poll::Ready(Ok(Received::Menu(menu)));
                }
                MessageHeader::Response(header) => {
                    return Poll::Ready(Ok(Received::Response { header, bytes }))
                }
                MessageHeader::Query(QueryHeader { tag, version, id })
                    if &tag == VersionedRpcMenuV1::NAME
                        && version == VersionedRpcMenuV1::VERSION =>
                {
                    let msg = Message::<<VersionedRpcMenuV1 as RpcMethod>::Response>::Response(
                        Response {
                            id,
                            data: RpcResult(Ok(NeedsLength(
                                self.menu
                                    .iter()
                                    .cloned()
                                    .map(|(tag, version)| (tag.into(), version))
                                    .collect(),
                            ))),
                        },
                    );
                    let mut bytes = vec![0; 8];
                    msg.binprot_write(&mut bytes).unwrap();
                    let len = (bytes.len() - 8) as u64;
                    bytes[..8].clone_from_slice(&len.to_le_bytes());

                    self.add(bytes);
                }
                MessageHeader::Query(header) => {
                    return Poll::Ready(Ok(Received::Query { header, bytes }))
                }
            };
        }

        if task::ready!(self.buffer.poll_fill(cx, &mut io))? != 0 {
            self.poll_recv(cx, io)
        } else {
            Poll::Ready(Err(io::ErrorKind::UnexpectedEof.into()))
        }
    }

    pub fn poll_send<T>(&mut self, cx: &mut Context<'_>, mut io: &mut T) -> Poll<io::Result<()>>
    where
        T: AsyncWrite + Unpin,
    {
        while let Some((offset, buf)) = self.command_queue.front_mut() {
            if *offset < buf.len() {
                let written = task::ready!(Pin::new(&mut io).poll_write(cx, &buf[*offset..]))?;
                *offset += written;
                if *offset >= buf.len() {
                    self.command_queue.pop_front();
                }
            }
        }

        Poll::Ready(Ok(()))
    }
}
