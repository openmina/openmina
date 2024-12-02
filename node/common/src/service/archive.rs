use mina_p2p_messages::v2::{self, ArchiveTransitionFronntierDiff};
use node::core::{channels::mpsc, thread};
use std::net::SocketAddr;

use super::NodeService;

pub struct ArchiveService {
    archive_sender: mpsc::UnboundedSender<ArchiveTransitionFronntierDiff>,
}

const ARCHIVE_SEND_RETRIES: u8 = 3;

impl ArchiveService {
    fn new(archive_sender: mpsc::UnboundedSender<ArchiveTransitionFronntierDiff>) -> Self {
        Self { archive_sender }
    }

    fn run(
        mut archive_receiver: mpsc::UnboundedReceiver<ArchiveTransitionFronntierDiff>,
        address: SocketAddr,
    ) {
        while let Some(breadcrumb) = archive_receiver.blocking_recv() {
            println!("[archive-service] Received breadcrumb, sending to archive...");
            let mut retries = ARCHIVE_SEND_RETRIES;
            while retries > 0 {
                // if let Err(e) = rpc::send_diff(address, v2::ArchiveRpc::SendDiff(breadcrumb.clone())) {
                //     node::core::warn!(
                //     node::core::log::system_time();
                //         summary = "Failed sending diff to archive",
                //         error = e.to_string()
                //     );
                //     retries -= 1;
                // } else {
                //     node::core::warn!(
                //         node::core::log::system_time();
                //         summary = "Successfully sent diff to archive",
                //     );
                //     break;
                // }

                match rpc::send_diff(address, v2::ArchiveRpc::SendDiff(breadcrumb.clone())) {
                    Ok(result) => {
                        if result.should_retry() {
                            node::core::warn!(node::core::log::system_time(); summary = "Archive suddenly closed connection, retrying...");
                            retries -= 1;
                        } else {
                            node::core::warn!(node::core::log::system_time(); summary = "Successfully sent diff to archive");
                            break;
                        }
                    }
                    Err(e) => {
                        node::core::warn!(node::core::log::system_time(); summary = "Failed sending diff to archive", error = e.to_string(), retries = retries);
                        retries -= 1;
                    }
                }
            }
            // Sleep for a bit to avoid flooding the archive
            // std::thread::sleep(std::time::Duration::from_millis(1000));
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
            if let Err(e) = archive.archive_sender.send(data.clone()) {
                node::core::warn!(
                    node::core::log::system_time();
                    summary = "Failed sending diff to archive service",
                    error = e.to_string()
                );
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

    pub enum HandleResult {
        MessageSent,
        ConnectionClosed,
        ConnectionAlive,
    }

    impl HandleResult {
        pub fn should_retry(&self) -> bool {
            matches!(self, Self::ConnectionClosed)
        }
    }

    pub fn send_diff(address: SocketAddr, data: v2::ArchiveRpc) -> io::Result<HandleResult> {
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

    fn process_rpc(address: SocketAddr, data: &[u8]) -> io::Result<HandleResult> {
        println!("[archive-service] Data length: {}", data.len());
        let mut poll = Poll::new()?;
        let mut events = Events::with_capacity(128);
        let mut event_count = 0;

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
                event_count += 1;
                println!("[archive-service] Event: {:?}", event_count);
                match event.token() {
                    TOKEN => {
                        match handle_connection_event(
                            poll.registry(),
                            &mut stream,
                            event,
                            data,
                            &mut handshake_received,
                            &mut handshake_sent,
                        )? {
                                HandleResult::MessageSent => return Ok(HandleResult::MessageSent),
                                HandleResult::ConnectionClosed => return Ok(HandleResult::ConnectionClosed),
                                HandleResult::ConnectionAlive => {
                                    continue;
                                },
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
    ) -> io::Result<HandleResult> {
        if event.is_writable() {
            if !*handshake_sent {
                match connection.write(&HANDSHAKE_MSG) {
                    Ok(n) if n < HANDSHAKE_MSG.len() => return Err(io::ErrorKind::WriteZero.into()),
                    Ok(_) => {
                        println!("[archive-service] Handshake sent");
                        *handshake_sent = true;
                        registry.reregister(connection, event.token(), Interest::READABLE)?;
                    }
                    Err(ref err) if would_block(err) => {}
                    Err(ref err) if interrupted(err) => {
                        println!("[archive-service] Handshake interrupted");
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
                return Ok(HandleResult::ConnectionAlive);
            }

            if *handshake_received && *handshake_sent {
                match connection.write_all(data) {
                    Ok(_) => {
                        println!("[archive-service] Message sent");
                        connection.flush()?;
                        // registry.deregister(connection)?;
                        // connection.shutdown(std::net::Shutdown::Both)?;
                        registry.reregister(connection, event.token(), Interest::READABLE)?;
                        return Ok(HandleResult::ConnectionAlive);
                    }
                    Err(ref err) if would_block(err) => {
                        println!("[archive-service] Message would block");
                    }
                    // Try again if interrupted
                    Err(ref err) if interrupted(err) => {
                        println!("[archive-service] Message interrupted");
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
                                println!("[archive-service] Handshake received");
                                registry.reregister(
                                    connection,
                                    event.token(),
                                    Interest::WRITABLE,
                                )?;
                                break;
                            }
                        } else if bytes_read >= 13 {
                            // or is this heartbeat?
                            let close_msg = [5, 0, 0, 0, 0, 0, 0, 0, 2, 1, 0, 1, 0];
                            if received_data[..13] == close_msg {
                                println!("[archive-service] Received close message");
                                registry.deregister(connection)?;
                                connection.shutdown(std::net::Shutdown::Both)?;
                                return Ok(HandleResult::MessageSent);
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

            println!("[archive-service] Received data: {:?}", received_data);

            if connection_closed {
                registry.deregister(connection)?;
                println!("[archive-service] Remote closed connection");
                connection.shutdown(std::net::Shutdown::Both)?;
                return Ok(HandleResult::ConnectionClosed);
            }
        }

        Ok(HandleResult::ConnectionAlive)
    }

    fn would_block(err: &io::Error) -> bool {
        err.kind() == io::ErrorKind::WouldBlock
    }

    fn interrupted(err: &io::Error) -> bool {
        err.kind() == io::ErrorKind::Interrupted
    }
}
