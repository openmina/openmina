use binprot::BinProtWrite;
use mina_p2p_messages::v2::{self, ArchiveTransitionFronntierDiff};
use node::core::{channels::mpsc, thread};

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
        address: &str,
    ) {
        while let Some(breadcrumb) = archive_receiver.blocking_recv() {
            println!("Sending data to archive");
            if let v2::ArchiveTransitionFronntierDiff::BreadcrumbAdded {
                block: (_, (_, ref state_hash)),
                ..
            } = breadcrumb
            {
                let filename = format!("{}-breadcrumb.bin", state_hash);
                let mut buff = Vec::new();
                breadcrumb.binprot_write(&mut buff).unwrap();
                std::fs::write(filename, buff).unwrap();
            }
            if let Err(e) = rpc::send_diff(address, v2::ArchiveRpc::SendDiff(breadcrumb)) {
                println!("Error sending to archive: {:?}", e);
            }
        }
    }

    pub fn start(address: &str) -> Self {
        let (archive_sender, archive_receiver) =
            mpsc::unbounded_channel::<ArchiveTransitionFronntierDiff>();

        let address = address.to_string();
        thread::Builder::new()
            .name("openmina_archive".to_owned())
            .spawn(move || {
                Self::run(archive_receiver, &address);
            })
            .unwrap();

        Self::new(archive_sender)
    }
}

impl node::transition_frontier::archive::archive_service::ArchiveService for NodeService {
    fn send_to_archive(&mut self, data: ArchiveTransitionFronntierDiff) {
        if let Some(archive) = self.archive.as_mut() {
            let _ = archive.archive_sender.send(data);
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
    use std::str::from_utf8;

    const HANDSHAKE_MSG: [u8; 15] = [7, 0, 0, 0, 0, 0, 0, 0, 2, 253, 82, 80, 67, 0, 1];

    pub fn send_diff(address: &str, data: v2::ArchiveRpc) -> io::Result<()> {
        let rpc = encode_to_rpc(data);
        process_rpc(address, &rpc)
    }

    pub fn encode_to_rpc(data: ArchiveRpc) -> Vec<u8> {
        type Method = mina_p2p_messages::rpc::SendArchiveDiffUnversioned;
        let mut v = vec![0; 8];

        Message::Query(Query {
            tag: Method::NAME.into(),
            version: Method::VERSION,
            id: 1,
            data: NeedsLength(data),
        })
        .binprot_write(&mut v)
        .unwrap();

        let payload_length = (v.len() - 8) as u64;
        v[..8].copy_from_slice(&payload_length.to_le_bytes());
        v.splice(0..0, [1, 0, 0, 0, 0, 0, 0, 0, 0].iter().cloned());

        v
    }

    fn process_rpc(url: &str, data: &[u8]) -> io::Result<()> {
        let mut poll = Poll::new().unwrap();
        let mut events = Events::with_capacity(128);

        // We still need a token even for one connection
        const TOKEN: Token = Token(0);

        let addr = url.parse().unwrap();
        let mut stream = TcpStream::connect(addr)?;

        let mut handshake_received = false;
        let mut handshake_sent = false;

        poll.registry()
            .register(&mut stream, TOKEN, Interest::WRITABLE)?;

        loop {
            if let Err(e) = poll.poll(&mut events, None) {
                if interrupted(&e) {
                    continue;
                }
                println!("Error polling: {:?}", e);
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
            // We can (maybe) write to the connection.

            println!("[event] writable, Handshake {handshake_sent} - {handshake_received}");

            if !*handshake_sent {
                println!("Sending handshake message");
                match connection.write(&HANDSHAKE_MSG) {
                    Ok(n) if n < HANDSHAKE_MSG.len() => return Err(io::ErrorKind::WriteZero.into()),
                    Ok(n) => {
                        println!("Wrote {} bytes", n);
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
                println!("Sent handshake message");
                return Ok(false);
            }

            if *handshake_received && *handshake_sent {
                match connection.write(data) {
                    Ok(n) if n < data.len() => return Err(io::ErrorKind::WriteZero.into()),
                    Ok(n) => {
                        println!("Wrote {} bytes", n);
                        registry.deregister(connection)?;
                        connection.shutdown(std::net::Shutdown::Both)?;
                        return Ok(true);
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
            }
        }

        if event.is_readable() {
            let mut connection_closed = false;
            let mut received_data = vec![0; 4096];
            let mut bytes_read = 0;

            println!("[event] readable, Handshake {handshake_sent} - {handshake_received}");
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
                                // Respond with the same heartbeat message
                                // connection.write_all(&handshake)?;
                                *handshake_received = true;
                                println!("Handshake received");
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

            if bytes_read != 0 {
                let received_data = &received_data[..bytes_read];
                if let Ok(str_buf) = from_utf8(received_data) {
                    println!("Received data: {}", str_buf.trim_end());
                } else {
                    println!("Received (none UTF-8) data: {:?}", received_data);
                }
            }

            if connection_closed {
                println!("Connection closed");
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

// TODO(adonagy): Remove
mod test {
    use binprot::BinProtRead;
    use ledger::AccountId;
    // const URL: &str = "65.21.205.249:3086";
    use mina_p2p_messages::v2;

    // #[test]
    // fn test_rpc() {
    //     let data = include_bytes!("../../../../tests/files/archive-breadcrumb/3NK56ZbCS31qb8SvCtCCYza4beRDtKgXA2JL6s3evKouG2KkKtiy.bin");
    //     let diff = v2::ArchiveTransitionFronntierDiff::binprot_read(&mut data.as_ref()).unwrap();
    //     super::rpc::send_diff(URL, v2::ArchiveRpc::SendDiff(diff)).unwrap();
    // }

    #[test]
    fn test() {
        // let data = include_bytes!("../../../../breadcrumbs/3NK2vJduqXCdaNGX1S9oWn5akwhLxgceBh8TxDGqnDzzKMF8YDcb-breadcrumb.bin");
        let data = include_bytes!("../../../../breadcrumbs/3NL22dMkBSnAc9cJq2mTEiuVxuE8TTzDV1a4hwxoWMEt31zhFoGC-breadcrumb.bin");
        let diff = v2::ArchiveTransitionFronntierDiff::binprot_read(&mut data.as_ref()).unwrap();
        if let v2::ArchiveTransitionFronntierDiff::BreadcrumbAdded { tokens_used, .. } = diff {
            let tokens_used = tokens_used
                .iter()
                .map(|(id, owner)| {
                    (
                        id.to_decimal(),
                        owner.as_ref().map(|owner| owner.0.to_string()),
                    )
                })
                .collect::<Vec<_>>();
            println!("Tokens used: {:#?}", tokens_used);
        }
    }
}
