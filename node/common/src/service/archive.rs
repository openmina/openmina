use mina_p2p_messages::v2::{self, ArchiveTransitionFronntierDiff};
use node::core::{channels::mpsc, thread};
use std::net::SocketAddr;

use super::NodeService;

pub struct ArchiveService {
    archive_sender: mpsc::UnboundedSender<ArchiveTransitionFronntierDiff>,
}

impl ArchiveService {
    fn new(archive_sender: mpsc::UnboundedSender<ArchiveTransitionFronntierDiff>) -> Self {
        Self { archive_sender }
    }

    fn run(
        mut archive_receiver: mpsc::UnboundedReceiver<ArchiveTransitionFronntierDiff>,
        address: SocketAddr,
    ) {
        while let Some(breadcrumb) = archive_receiver.blocking_recv() {
            if let Err(e) = rpc::send_diff(address, v2::ArchiveRpc::SendDiff(breadcrumb)) {
                node::core::warn!(
                    node::core::log::system_time();
                    summary = "Failed sending diff to archive",
                    error = e.to_string()
                )
            }
        }
    }

    pub fn start(address: SocketAddr) -> Self {
        let (archive_sender, archive_receiver) =
            mpsc::unbounded_channel::<ArchiveTransitionFronntierDiff>();

        thread::Builder::new()
            .name("openmina_archive".to_owned())
            .spawn(move || {
                Self::run(archive_receiver, address);
            })
            .unwrap();

        Self::new(archive_sender)
    }
}

impl node::transition_frontier::archive::archive_service::ArchiveService for NodeService {
    fn send_to_archive(&mut self, data: ArchiveTransitionFronntierDiff) {
        if let Some(archive) = self.archive.as_mut() {
            if let Err(e) = archive.archive_sender.send(data) {
                node::core::warn!(
                    node::core::log::system_time();
                    summary = "Failed sending diff to archive",
                    error = e.to_string()
                )
            }
        }
    }
}

// We need to replicate the ocaml node's RPC like interface
mod rpc {
    use binprot::BinProtWrite;
    use mina_p2p_messages::rpc_kernel::{Message, NeedsLength, Query, RpcMethod};
    use mina_p2p_messages::v2::{self, ArchiveRpc};
    use mio::event::Event;
    use mio::net::TcpStream;
    use mio::{Events, Interest, Poll, Registry, Token};
    use std::io::{self, Read, Write};
    use std::net::SocketAddr;

    const HANDSHAKE_MSG: [u8; 15] = [7, 0, 0, 0, 0, 0, 0, 0, 2, 253, 82, 80, 67, 0, 1];

    pub fn send_diff(address: SocketAddr, data: v2::ArchiveRpc) -> io::Result<()> {
        let rpc = encode_to_rpc(data);
        process_rpc(address, &rpc)
    }

    fn encode_to_rpc(data: ArchiveRpc) -> Vec<u8> {
        type Method = mina_p2p_messages::rpc::SendArchiveDiffUnversioned;
        let mut v = vec![0; 8];

        if let Err(e) = Message::Query(Query {
            tag: Method::NAME.into(),
            version: Method::VERSION,
            id: 1,
            data: NeedsLength(data),
        })
        .binprot_write(&mut v)
        {
            node::core::warn!(
                node::core::log::system_time();
                summary = "Failed binprot serializastion",
                error = e.to_string()
            )
        }

        let payload_length = (v.len() - 8) as u64;
        v[..8].copy_from_slice(&payload_length.to_le_bytes());
        v.splice(0..0, [1, 0, 0, 0, 0, 0, 0, 0, 0].iter().cloned());

        v
    }

    fn process_rpc(address: SocketAddr, data: &[u8]) -> io::Result<()> {
        let mut poll = Poll::new()?;
        let mut events = Events::with_capacity(128);

        // We still need a token even for one connection
        const TOKEN: Token = Token(0);

        let mut stream = TcpStream::connect(address)?;

        let mut handshake_received = false;
        let mut handshake_sent = false;

        poll.registry()
            .register(&mut stream, TOKEN, Interest::WRITABLE)?;

        loop {
            if let Err(e) = poll.poll(&mut events, None) {
                if interrupted(&e) {
                    continue;
                }
                return Err(e);
            }

            for event in events.iter() {
                match event.token() {
                    TOKEN => {
                        if handle_connection_event(
                            poll.registry(),
                            &mut stream,
                            event,
                            data,
                            &mut handshake_received,
                            &mut handshake_sent,
                        )? {
                            return Ok(());
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    /// Returns `true` if the connection is done.
    fn handle_connection_event(
        registry: &Registry,
        connection: &mut TcpStream,
        event: &Event,
        data: &[u8],
        handshake_received: &mut bool,
        handshake_sent: &mut bool,
    ) -> io::Result<bool> {
        if event.is_writable() {
            if !*handshake_sent {
                match connection.write(&HANDSHAKE_MSG) {
                    Ok(n) if n < HANDSHAKE_MSG.len() => return Err(io::ErrorKind::WriteZero.into()),
                    Ok(_) => {
                        registry.reregister(connection, event.token(), Interest::READABLE)?;
                    }
                    Err(ref err) if would_block(err) => {}
                    Err(ref err) if interrupted(err) => {
                        return handle_connection_event(
                            registry,
                            connection,
                            event,
                            data,
                            handshake_received,
                            handshake_sent,
                        )
                    }
                    // Other errors we'll consider fatal.
                    Err(err) => return Err(err),
                }
                *handshake_sent = true;
                return Ok(false);
            }

            if *handshake_received && *handshake_sent {
                match connection.write(data) {
                    Ok(n) if n < data.len() => return Err(io::ErrorKind::WriteZero.into()),
                    Ok(_) => {
                        registry.deregister(connection)?;
                        connection.shutdown(std::net::Shutdown::Both)?;
                        return Ok(true);
                    }
                    Err(ref err) if would_block(err) => {}
                    // Try again if interrupted
                    Err(ref err) if interrupted(err) => {
                        return handle_connection_event(
                            registry,
                            connection,
                            event,
                            data,
                            handshake_received,
                            handshake_sent,
                        )
                    }
                    // Other errors we'll consider fatal.
                    Err(err) => return Err(err),
                }
            }
        }

        if event.is_readable() {
            let mut connection_closed = false;
            let mut received_data = vec![0; 4096];
            let mut bytes_read = 0;

            loop {
                match connection.read(&mut received_data[bytes_read..]) {
                    Ok(0) => {
                        connection_closed = true;
                        break;
                    }
                    Ok(n) => {
                        bytes_read += n;
                        if bytes_read >= 15 {
                            let handshake = [7, 0, 0, 0, 0, 0, 0, 0, 2, 253, 82, 80, 67, 0, 1];
                            if received_data[..15] == handshake {
                                *handshake_received = true;
                                registry.reregister(
                                    connection,
                                    event.token(),
                                    Interest::WRITABLE,
                                )?;
                                break;
                            }
                        }
                        if bytes_read == received_data.len() {
                            received_data.resize(received_data.len() + 1024, 0);
                        }
                    }
                    // Would block "errors" are the OS's way of saying that the
                    // connection is not actually ready to perform this I/O operation.
                    Err(ref err) if would_block(err) => break,
                    Err(ref err) if interrupted(err) => continue,
                    // Other errors we'll consider fatal.
                    Err(err) => return Err(err),
                }
            }

            if connection_closed {
                registry.deregister(connection)?;
                connection.shutdown(std::net::Shutdown::Both)?;
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn would_block(err: &io::Error) -> bool {
        err.kind() == io::ErrorKind::WouldBlock
    }

    fn interrupted(err: &io::Error) -> bool {
        err.kind() == io::ErrorKind::Interrupted
    }
}
