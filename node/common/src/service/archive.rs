use bitflags::bitflags;
use mina_p2p_messages::v2::{self, ArchiveTransitionFronntierDiff};
use node::core::{channels::mpsc, thread};
use node::ledger::write::BlockApplyResult;
use std::env;
use std::net::SocketAddr;

use super::NodeService;

const ARCHIVE_SEND_RETRIES: u8 = 5;
const MAX_EVENT_COUNT: u64 = 100;
const RETRY_INTERVAL_MS: u64 = 1000;

bitflags! {
    #[derive(Debug, Clone, Default)]
    pub struct ArchiveStorageOptions: u8 {
        const ARCHIVER_PROCESS = 0b0001;
        const LOCAL_PRECOMPUTED_STORAGE = 0b0010;
        const GCP_PRECOMPUTED_STORAGE = 0b0100;
        const AWS_PRECOMPUTED_STORAGE = 0b1000;
    }
}

impl ArchiveStorageOptions {
    pub fn is_enabled(&self) -> bool {
        self.is_empty()
    }

    pub fn requires_precomputed_block(&self) -> bool {
        self.uses_aws_precomputed_storage()
            || self.uses_gcp_precomputed_storage()
            || self.uses_local_precomputed_storage()
    }

    pub fn validate_env_vars(&self) -> Result<(), String> {
        if self.contains(ArchiveStorageOptions::ARCHIVER_PROCESS)
            && env::var("OPENMINA_ARCHIVE_ADDRESS").is_err()
        {
            return Err(
                "OPENMINA_ARCHIVE_ADDRESS is required when ARCHIVER_PROCESS is enabled".to_string(),
            );
        }

        if self.uses_aws_precomputed_storage() {
            if env::var("AWS_ACCESS_KEY_ID").is_err() {
                return Err(
                    "AWS_ACCESS_KEY_ID is required when AWS_PRECOMPUTED_STORAGE is enabled"
                        .to_string(),
                );
            }
            if env::var("AWS_SECRET_ACCESS_KEY").is_err() {
                return Err(
                    "AWS_SECRET_ACCESS_KEY is required when AWS_PRECOMPUTED_STORAGE is enabled"
                        .to_string(),
                );
            }
            if env::var("AWS_SESSION_TOKEN").is_err() {
                return Err(
                    "AWS_SESSION_TOKEN is required when AWS_PRECOMPUTED_STORAGE is enabled"
                        .to_string(),
                );
            }
        }

        // TODO(adonagy): Add GCP precomputed storage validation

        Ok(())
    }

    pub fn uses_local_precomputed_storage(&self) -> bool {
        self.contains(ArchiveStorageOptions::LOCAL_PRECOMPUTED_STORAGE)
    }

    pub fn uses_archiver_process(&self) -> bool {
        self.contains(ArchiveStorageOptions::ARCHIVER_PROCESS)
    }

    pub fn uses_gcp_precomputed_storage(&self) -> bool {
        self.contains(ArchiveStorageOptions::GCP_PRECOMPUTED_STORAGE)
    }

    pub fn uses_aws_precomputed_storage(&self) -> bool {
        self.contains(ArchiveStorageOptions::AWS_PRECOMPUTED_STORAGE)
    }
}

pub struct ArchiveService {
    archive_sender: mpsc::UnboundedSender<BlockApplyResult>,
}

struct ArchiveServiceClients {
    aws_client: Option<aws_sdk_s3::Client>,
    gcp_client: Option<()>,
}

impl ArchiveServiceClients {
    async fn new(options: &ArchiveStorageOptions) -> Self {
        let aws_client = if options.uses_aws_precomputed_storage() {
            let config = aws_config::load_from_env().await;
            Some(aws_sdk_s3::Client::new(&config))
        } else {
            None
        };
        Self {
            aws_client,
            gcp_client: None,
        }
    }

    pub fn aws_client(&self) -> Option<&aws_sdk_s3::Client> {
        self.aws_client.as_ref()
    }

    pub fn gcp_client(&self) -> Option<&()> {
        self.gcp_client.as_ref()
    }
}

impl ArchiveService {
    fn new(archive_sender: mpsc::UnboundedSender<BlockApplyResult>) -> Self {
        Self { archive_sender }
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn run(
        mut archive_receiver: mpsc::UnboundedReceiver<BlockApplyResult>,
        address: SocketAddr,
        options: ArchiveStorageOptions,
    ) {
        use mina_p2p_messages::v2::PrecomputedBlock;

        let clients = ArchiveServiceClients::new(&options).await;

        while let Some(breadcrumb) = archive_receiver.blocking_recv() {
            if options.uses_archiver_process() {
                let mut retries = ARCHIVE_SEND_RETRIES;

                let archive_transition_frontier_diff: v2::ArchiveTransitionFronntierDiff =
                    breadcrumb.clone().try_into().unwrap();

                while retries > 0 {
                    match rpc::send_diff(
                        address,
                        v2::ArchiveRpc::SendDiff(archive_transition_frontier_diff.clone()),
                    ) {
                        Ok(result) => {
                            if result.should_retry() {
                                node::core::warn!(
                                    summary = "Archive suddenly closed connection, retrying..."
                                );
                                retries -= 1;
                                std::thread::sleep(std::time::Duration::from_millis(
                                    RETRY_INTERVAL_MS,
                                ));
                            } else {
                                node::core::warn!(summary = "Successfully sent diff to archive");
                                break;
                            }
                        }
                        Err(e) => {
                            node::core::warn!(
                                summary = "Failed sending diff to archive",
                                error = e.to_string(),
                                retries = retries
                            );
                            retries -= 1;
                            std::thread::sleep(std::time::Duration::from_millis(RETRY_INTERVAL_MS));
                        }
                    }
                }
            }

            if options.requires_precomputed_block() {
                // let key =
                // TODO(adonagy)
                let precomputed_block: PrecomputedBlock =
                    if let Ok(precomputed_block) = breadcrumb.try_into() {
                        precomputed_block
                    } else {
                        node::core::warn!(
                            summary = "Failed to convert breadcrumb to precomputed block"
                        );
                        continue;
                    };

                if options.uses_local_precomputed_storage() {
                    // TODO(adonagy): Implement local precomputed storage
                }

                if options.uses_gcp_precomputed_storage() {
                    // TODO(adonagy): Implement GCP precomputed storage
                }

                if options.uses_aws_precomputed_storage() {
                    let client = clients.aws_client().unwrap();
                    // put
                }
            }
        }
    }

    // Note: Placeholder for the wasm implementation, if we decide to include an archive mode in the future
    #[cfg(target_arch = "wasm32")]
    fn run(
        mut archive_receiver: mpsc::UnboundedReceiver<ArchiveTransitionFronntierDiff>,
        address: SocketAddr,
        options: ArchiveStorageOptions,
    ) {
        unimplemented!()
    }

    pub fn start(address: SocketAddr, options: ArchiveStorageOptions) -> Self {
        let (archive_sender, archive_receiver) = mpsc::unbounded_channel::<BlockApplyResult>();

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        thread::Builder::new()
            .name("openmina_archive".to_owned())
            .spawn(move || {
                runtime.block_on(Self::run(archive_receiver, address, options));
            })
            .unwrap();

        Self::new(archive_sender)
    }
}

impl node::transition_frontier::archive::archive_service::ArchiveService for NodeService {
    fn send_to_archive(&mut self, data: BlockApplyResult) {
        if let Some(archive) = self.archive.as_mut() {
            if let Err(e) = archive.archive_sender.send(data) {
                node::core::warn!(
                    summary = "Failed sending diff to archive service",
                    error = e.to_string()
                );
            }
        }
    }
}

// We need to replicate the ocaml node's RPC like interface
#[cfg(not(target_arch = "wasm32"))]
mod rpc {
    use binprot::BinProtWrite;
    use mina_p2p_messages::rpc_kernel::{Message, NeedsLength, Query, RpcMethod};
    use mina_p2p_messages::v2::{self, ArchiveRpc};
    use mio::event::Event;
    use mio::net::TcpStream;
    use mio::{Events, Interest, Poll, Registry, Token};
    use std::io::{self, Read, Write};
    use std::net::SocketAddr;

    const MAX_RECURSION_DEPTH: u8 = 25;

    // messages
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
                // Failsafe to prevent infinite loops
                if event_count > super::MAX_EVENT_COUNT {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("FAILSAFE triggered, event count: {}", event_count),
                    ));
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

    fn _send_heartbeat(connection: &mut TcpStream) -> io::Result<HandleResult> {
        match connection.write_all(&HEARTBEAT_MSG) {
            Ok(_) => {
                connection.flush()?;
                Ok(HandleResult::ConnectionAlive)
            }
            Err(ref err) if would_block(err) => Ok(HandleResult::MessageWouldBlock),
            Err(ref err) if interrupted(err) => Ok(HandleResult::MessageWouldBlock),
            Err(err) => Err(err),
        }
    }

    struct RecursionGuard {
        count: u8,
        max_depth: u8,
    }

    impl RecursionGuard {
        fn new(max_depth: u8) -> Self {
            Self {
                count: 0,
                max_depth,
            }
        }

        fn increment(&mut self) -> io::Result<()> {
            self.count += 1;
            if self.count > self.max_depth {
                Err(io::ErrorKind::WriteZero.into())
            } else {
                Ok(())
            }
        }
    }

    fn send_data<F>(
        connection: &mut TcpStream,
        data: &[u8],
        recursion_guard: &mut RecursionGuard,
        // closure that can be called when the data is sent
        on_success: F,
    ) -> io::Result<HandleResult>
    where
        F: FnOnce() -> io::Result<HandleResult>,
    {
        match connection.write(data) {
            Ok(n) if n < data.len() => {
                recursion_guard.increment()?;
                let remaining_data = data[n..].to_vec();
                send_data(connection, &remaining_data, recursion_guard, on_success)
            }
            Ok(_) => {
                connection.flush()?;
                on_success()
            }
            Err(ref err) if would_block(err) => Ok(HandleResult::MessageWouldBlock),
            Err(ref err) if interrupted(err) => {
                recursion_guard
                    .increment()
                    .map_err(|_| io::ErrorKind::Interrupted)?;
                send_data(connection, data, recursion_guard, on_success)
            }
            Err(err) => Err(err),
        }
    }

    #[allow(clippy::too_many_arguments)]
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
                send_data(
                    connection,
                    &msg,
                    &mut RecursionGuard::new(MAX_RECURSION_DEPTH),
                    || {
                        *handshake_sent = true;
                        Ok(HandleResult::ConnectionAlive)
                    },
                )?;
                return Ok(HandleResult::ConnectionAlive);
            }

            if *handshake_received && *handshake_sent && !*message_sent && *first_heartbeat_received
            {
                send_data(
                    connection,
                    data,
                    &mut RecursionGuard::new(MAX_RECURSION_DEPTH),
                    || {
                        *message_sent = true;
                        Ok(HandleResult::ConnectionAlive)
                    },
                )?;
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
                    }
                    ParsedMessage::Ok | ParsedMessage::Close => {
                        connection.flush()?;
                        registry.deregister(connection)?;
                        connection.shutdown(std::net::Shutdown::Both)?;
                        return Ok(HandleResult::MessageSent);
                    }
                    ParsedMessage::Heartbeat => {
                        *first_heartbeat_received = true;
                    }
                    ParsedMessage::Unknown(msg) => {
                        registry.deregister(connection)?;
                        connection.shutdown(std::net::Shutdown::Both)?;
                        node::core::warn!(
                            summary = "Received unknown message",
                            msg = format!("{:?}", msg)
                        );
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

// Note: Placeholder for the wasm implementation, if we decide to include an archive mode in the future
#[cfg(target_arch = "wasm32")]
mod rpc {}
