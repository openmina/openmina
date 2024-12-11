use mina_p2p_messages::v2::{self, ArchiveTransitionFronntierDiff};
use node::core::{channels::mpsc, thread};
use std::net::SocketAddr;

use super::NodeService;

pub struct ArchiveService {
    archive_sender: mpsc::UnboundedSender<ArchiveTransitionFronntierDiff>,
}

const ARCHIVE_SEND_RETRIES: u8 = 5;

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
                match rpc::send_diff(address, v2::ArchiveRpc::SendDiff(breadcrumb.clone())) {
                    Ok(result) => {
                        if result.should_retry() {
                            node::core::warn!(node::core::log::system_time(); summary = "Archive suddenly closed connection, retrying...");
                            retries -= 1;
                            std::thread::sleep(std::time::Duration::from_millis(1000));
                        } else {
                            node::core::warn!(node::core::log::system_time(); summary = "Successfully sent diff to archive");
                            break;
                        }
                    }
                    Err(e) => {
                        node::core::warn!(node::core::log::system_time(); summary = "Failed sending diff to archive", error = e.to_string(), retries = retries);
                        retries -= 1;
                        std::thread::sleep(std::time::Duration::from_millis(1000));
                    }
                }
            }
            // Sleep to avoid overloading the archive during catchup
            // TODO: remove this after debugging, this is a temporary fix
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

    const HEADER_MSG: [u8; 7] = [2, 253, 82, 80, 67, 0, 1];
    const OK_MSG: [u8; 5] = [2, 1, 0, 1, 0];
    // Note: this is the close message that the ocaml node receives
    const CLOSE_MSG: [u8; 7] = [2, 254, 167, 7, 0, 1, 0];
    const HEARTBEAT_MSG: [u8; 1] = [0];

    fn prepend_length(message: &[u8]) -> Vec<u8> {
        let length = message.len() as u64;
        let mut length_bytes = length.to_le_bytes().to_vec();
        length_bytes.append(&mut message.to_vec());
        length_bytes
    }
    pub enum HandleResult {
        MessageSent,
        ConnectionClosed,
        ConnectionAlive,
        MessageWouldBlock,
    }

    impl HandleResult {
        pub fn should_retry(&self) -> bool {
            matches!(self, Self::ConnectionClosed)
        }
    }

    pub fn send_diff(address: SocketAddr, data: v2::ArchiveRpc) -> io::Result<HandleResult> {
        let rpc = encode_to_rpc(data)?;
        process_rpc(address, &rpc)
    }

    fn encode_to_rpc(data: ArchiveRpc) -> io::Result<Vec<u8>> {
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
            );
            return Err(e);
        }

        let payload_length = (v.len() - 8) as u64;
        v[..8].copy_from_slice(&payload_length.to_le_bytes());
        // Bake in the heartbeat message
        v.splice(0..0, prepend_length(&HEARTBEAT_MSG).iter().cloned());
        // also add the heartbeat message to the end of the message
        v.extend_from_slice(&prepend_length(&HEARTBEAT_MSG));

        Ok(v)
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
        let mut message_sent = false;
        let mut first_heartbeat_received = false;
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
                println!(
                    "[archive-service] Event: {:?} [R/W:{}/{}]",
                    event_count,
                    event.is_readable(),
                    event.is_writable()
                );
                // TODO: remove this after debugging
                if event_count > 100 {
                    panic!("FAILSAFE triggered, event count: {}", event_count);
                }
                match event.token() {
                    TOKEN => {
                        match handle_connection_event(
                            poll.registry(),
                            &mut stream,
                            event,
                            data,
                            &mut handshake_received,
                            &mut handshake_sent,
                            &mut message_sent,
                            &mut first_heartbeat_received,
                        )? {
                            HandleResult::MessageSent => return Ok(HandleResult::MessageSent),
                            HandleResult::ConnectionClosed => {
                                return Ok(HandleResult::ConnectionClosed)
                            }
                            HandleResult::MessageWouldBlock => {
                                // do nothing, wait for the next event
                                continue;
                            }
                            HandleResult::ConnectionAlive => {
                                // keep swapping between readable and writable until we successfully send the message, then keep in read mode.
                                if message_sent {
                                    poll.registry().reregister(
                                        &mut stream,
                                        TOKEN,
                                        Interest::READABLE,
                                    )?;
                                    continue;
                                }

                                if event.is_writable() {
                                    poll.registry().reregister(
                                        &mut stream,
                                        TOKEN,
                                        Interest::READABLE,
                                    )?;
                                } else {
                                    poll.registry().reregister(
                                        &mut stream,
                                        TOKEN,
                                        Interest::WRITABLE,
                                    )?;
                                }
                                continue;
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    fn send_heartbeat(connection: &mut TcpStream) -> io::Result<HandleResult> {
        match connection.write_all(&HEARTBEAT_MSG) {
            Ok(_) => {
                connection.flush()?;
                Ok(HandleResult::ConnectionAlive)
            }
            Err(ref err) if would_block(err) => {
                println!("[archive-service] Heartbeat would block");
                Ok(HandleResult::MessageWouldBlock)
            }
            Err(ref err) if interrupted(err) => {
                println!("[archive-service] Heartbeat interrupted");
                Ok(HandleResult::MessageWouldBlock)
            }
            Err(err) => Err(err),
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
        message_sent: &mut bool,
        first_heartbeat_received: &mut bool,
    ) -> io::Result<HandleResult> {
        if event.is_writable() {
            if !*handshake_sent {
                let msg = prepend_length(&HEADER_MSG);
                match connection.write(&msg) {
                    Ok(n) if n < msg.len() => return Err(io::ErrorKind::WriteZero.into()),
                    Ok(_) => {
                        println!("[archive-service] Handshake sent");
                        *handshake_sent = true;
                    }
                    Err(ref err) if would_block(err) => {
                        println!("[archive-service] Handshake would block");
                        return Ok(HandleResult::MessageWouldBlock);
                    }
                    Err(ref err) if interrupted(err) => {
                        println!("[archive-service] Handshake interrupted");
                        return handle_connection_event(
                            registry,
                            connection,
                            event,
                            data,
                            handshake_received,
                            handshake_sent,
                            message_sent,
                            first_heartbeat_received,
                        );
                    }
                    // Other errors we'll consider fatal.
                    Err(err) => return Err(err),
                }
                return Ok(HandleResult::ConnectionAlive);
            }

            if *handshake_received && *handshake_sent && !*message_sent && *first_heartbeat_received
            {
                match connection.write(data) {
                    Ok(n) if n < data.len() => {
                        let remaining_data = data[n..].to_vec();
                        // TODO: add recursion guard
                        return handle_connection_event(
                            registry,
                            connection,
                            event,
                            &remaining_data,
                            handshake_received,
                            handshake_sent,
                            message_sent,
                            first_heartbeat_received,
                        );
                    }
                    Ok(_) => {
                        println!("[archive-service] Message sent");
                        connection.flush()?;
                        *message_sent = true;
                        // return send_heartbeat(connection);
                        return Ok(HandleResult::ConnectionAlive);
                    }
                    Err(ref err) if would_block(err) => {
                        println!("[archive-service] Message would block");
                        return Ok(HandleResult::MessageWouldBlock);
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
                            message_sent,
                            first_heartbeat_received,
                        );
                    }
                    // Other errors we'll consider fatal.
                    Err(err) => return Err(err),
                }
            } else {
                // Send a heartbeat to keep the connection alive until we receive the closing message
                // match connection.write_all(&HEARTBEAT_MSG) {
                //     Ok(_) => {
                //         println!("[archive-service] Heartbeat sent");
                //         connection.flush()?;
                //         return Ok(HandleResult::ConnectionAlive);
                //     }
                //     Err(ref err) if would_block(err) => {
                //         println!("[archive-service] Heartbeat would block");
                //     }
                //     // Try again if interrupted
                //     Err(ref err) if interrupted(err) => {
                //         println!("[archive-service] Heartbeat interrupted");
                //         return handle_connection_event(
                //             registry,
                //             connection,
                //             event,
                //             data,
                //             handshake_received,
                //             handshake_sent,
                //             message_sent,
                //         );
                //     }
                //     // Other errors we'll consider fatal.
                //     Err(err) => return Err(err),
                // }
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
                println!("[archive-service] Remote closed connection");
                connection.shutdown(std::net::Shutdown::Both)?;
                return Ok(HandleResult::ConnectionClosed);
            }

            if bytes_read < 8 {
                // malformed message, at least the length should be present
                return Ok(HandleResult::ConnectionAlive);
            }

            let raw_message = RawMessage::from_bytes(&received_data[..bytes_read]);
            let messages = raw_message.parse_raw()?;

            for message in messages {
                match message {
                    ParsedMessage::Header => {
                        *handshake_received = true;
                        println!("[archive-service] Handshake received");
                    }
                    ParsedMessage::Ok | ParsedMessage::Close => {
                        println!("[archive-service] Received close message");
                        connection.flush()?;
                        registry.deregister(connection)?;
                        connection.shutdown(std::net::Shutdown::Both)?;
                        return Ok(HandleResult::MessageSent);
                    }
                    ParsedMessage::Heartbeat => {
                        println!("[archive-service] Received heartbeat message");
                        *first_heartbeat_received = true;
                    }
                    ParsedMessage::Unknown(unknown) => {
                        println!("[archive-service] Received unknown message: {:?}", unknown);
                        registry.deregister(connection)?;
                        connection.shutdown(std::net::Shutdown::Both)?;
                        return Ok(HandleResult::ConnectionClosed);
                    }
                }
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

    enum ParsedMessage {
        Heartbeat,
        Ok,
        Close,
        Header,
        Unknown(Vec<u8>),
    }

    struct RawMessage {
        length: usize,
        data: Vec<u8>,
    }

    impl RawMessage {
        fn from_bytes(bytes: &[u8]) -> Self {
            println!("[archive-service-parser] Raw message: {:?}", bytes);
            Self {
                length: bytes.len(),
                data: bytes.to_vec(),
            }
        }

        fn parse_raw(&self) -> io::Result<Vec<ParsedMessage>> {
            let mut parsed_bytes: usize = 0;

            // more than one message can be sent in a single packet
            let mut messages = Vec::new();

            while parsed_bytes < self.length {
                // first 8 bytes are the length in little endian
                let length = u64::from_le_bytes(
                    self.data[parsed_bytes..parsed_bytes + 8]
                        .try_into()
                        .unwrap(),
                ) as usize;
                parsed_bytes += 8;

                println!("[archive-service-parser] Parsed length: {}", length);
                println!(
                    "[archive-service-parser] Parsed bytes: {:?}",
                    &self.data[parsed_bytes..parsed_bytes + length]
                );

                if parsed_bytes + length > self.length {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Message length exceeds raw message length",
                    ));
                }

                if length == HEADER_MSG.len()
                    && self.data[parsed_bytes..parsed_bytes + length] == HEADER_MSG
                {
                    messages.push(ParsedMessage::Header);
                } else if length == OK_MSG.len()
                    && self.data[parsed_bytes..parsed_bytes + length] == OK_MSG
                {
                    messages.push(ParsedMessage::Ok);
                } else if length == HEARTBEAT_MSG.len()
                    && self.data[parsed_bytes..parsed_bytes + length] == HEARTBEAT_MSG
                {
                    messages.push(ParsedMessage::Heartbeat);
                } else if length == CLOSE_MSG.len()
                    && self.data[parsed_bytes..parsed_bytes + length] == CLOSE_MSG
                {
                    messages.push(ParsedMessage::Close);
                } else {
                    messages.push(ParsedMessage::Unknown(
                        self.data[parsed_bytes..parsed_bytes + length].to_vec(),
                    ));
                }

                parsed_bytes += length;
            }
            Ok(messages)
        }
    }
}
