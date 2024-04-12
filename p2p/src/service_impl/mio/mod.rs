mod token;
use self::token::{Token, TokenRegistry};

use std::{
    collections::{BTreeMap, VecDeque},
    io::{self, Read, Write},
    net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr},
    process,
    sync::mpsc,
    thread,
};

use mio::net::{TcpListener, TcpStream};

use thiserror::Error;

use crate::{MioCmd, MioEvent, P2pMioService};

#[derive(Debug, Error)]
enum MioError {
    #[error("mio failed to create poll instance, fatal error: {0}")]
    New(io::Error),
    #[error("mio failed to create waker instance, fatal error: {0}")]
    Waker(io::Error),
    #[error("mio failed to poll events, error: {0}")]
    Poll(io::Error),
    #[error("mio failed to start listening on {0}, error: {1}")]
    Listen(SocketAddr, io::Error),
}

impl MioError {
    fn report(self) {
        openmina_core::log::error!(
            openmina_core::log::system_time();
            kind = "MioError",
            summary = self.to_string(),
        );
    }
}

pub struct MioService {
    cmd_sender: mpsc::Sender<MioCmd>,
    waker: Option<mio::Waker>,
}

impl redux::TimeService for MioService {}

impl redux::Service for MioService {}

impl P2pMioService for MioService {
    fn send_mio_cmd(&mut self, cmd: MioCmd) {
        self.cmd_sender.send(cmd).unwrap_or_default();
        if let Some(w) = self.waker.as_ref() {
            w.wake().unwrap_or_default()
        }
    }
}

impl MioService {
    pub fn mocked() -> Self {
        MioService {
            cmd_sender: mpsc::channel().0,
            waker: None,
        }
    }

    pub fn run<F>(event_sender: F) -> Self
    where
        F: 'static + Send + Sync + Fn(MioEvent),
    {
        let poll = match mio::Poll::new() {
            Ok(v) => v,
            Err(err) => {
                MioError::New(err).report();
                process::exit(1);
            }
        };

        let mut tokens = TokenRegistry::default();
        let waker = match mio::Waker::new(poll.registry(), tokens.register(Token::Waker)) {
            Ok(v) => v,
            Err(err) => {
                MioError::Waker(err).report();
                process::exit(1);
            }
        };

        let (tx, rx) = mpsc::channel();

        let mut inner = MioServiceInner {
            poll,
            event_sender,
            cmd_receiver: rx,
            tokens,
            listeners: BTreeMap::default(),
            connections: BTreeMap::default(),
        };

        thread::spawn(move || {
            // fake interfaces, TODO: detect interfaces properly
            inner.send(MioEvent::InterfaceDetected(IpAddr::V4(
                Ipv4Addr::UNSPECIFIED,
            )));

            let mut events = mio::Events::with_capacity(1024);

            loop {
                inner.run(&mut events);
            }
        });

        MioService {
            cmd_sender: tx,
            waker: Some(waker),
        }
    }
}

struct MioServiceInner<F> {
    poll: mio::Poll,
    event_sender: F,
    cmd_receiver: mpsc::Receiver<MioCmd>,
    tokens: TokenRegistry,
    listeners: BTreeMap<SocketAddr, Listener>,
    connections: BTreeMap<SocketAddr, Connection>,
}

struct Listener {
    inner: TcpListener,
    incomind_ready: bool,
}

struct Connection {
    stream: TcpStream,
    transmits: VecDeque<(Box<[u8]>, usize)>,
    connected: bool,
    incoming_ready: bool,
}

impl<F> MioServiceInner<F>
where
    F: 'static + Send + Sync + Fn(MioEvent),
{
    fn run(&mut self, events: &mut mio::Events) {
        if let Err(err) = self.poll.poll(events, None) {
            MioError::Poll(err).report();
        }

        'events: for event in events.iter() {
            match self.tokens.get(&event.token()) {
                None => {}
                Some(Token::Waker) => {
                    while let Ok(cmd) = self.cmd_receiver.try_recv() {
                        self.handle(cmd);
                    }
                }
                Some(Token::Listener(addr)) => {
                    let Some(mut listener) = self.listeners.remove(&addr) else {
                        continue 'events;
                    };

                    if event.is_readable() && !listener.incomind_ready {
                        self.send(MioEvent::IncomingConnectionIsReady { listener: addr });
                        listener.incomind_ready = true;
                    }
                    self.listeners.insert(addr, listener);
                }
                Some(Token::Connection(mut addr)) => {
                    let Some(mut connection) = self.connections.remove(&addr) else {
                        continue 'events;
                    };
                    if event.is_error() {
                        match connection.stream.take_error() {
                            Ok(Some(e)) => {
                                self.send(MioEvent::ConnectionDidClose(addr, Err(e.to_string())));
                            }
                            Ok(None) => {
                                openmina_core::error!(
                                    openmina_core::log::system_time();
                                    summary = "mio error event without actual error",
                                    addr = openmina_core::log::inner::field::display(addr),
                                );
                            }
                            Err(e) => {
                                openmina_core::error!(
                                    openmina_core::log::system_time();
                                    summary = "error getting mio error",
                                    error = openmina_core::log::inner::field::display(e),
                                    addr = openmina_core::log::inner::field::display(addr),
                                );
                            }
                        }
                        continue 'events;
                    }
                    if event.is_readable() && !connection.incoming_ready {
                        connection.incoming_ready = true;
                        self.send(MioEvent::IncomingDataIsReady(addr));
                    }
                    let mut rereg = false;
                    if event.is_writable() {
                        if !connection.connected {
                            // make network debugger happy
                            let _ = connection.stream.take_error();
                            match connection.stream.peer_addr() {
                                Ok(new_addr) => {
                                    connection.connected = true;
                                    addr = new_addr;
                                    self.send(MioEvent::OutgoingConnectionDidConnect(addr, Ok(())));
                                }
                                Err(err) if err.kind() == io::ErrorKind::NotConnected => {
                                    rereg = true;
                                }
                                #[cfg(unix)]
                                Err(err) if err.raw_os_error() == Some(libc::EINPROGRESS) => {
                                    rereg = true;
                                }
                                Err(err) => {
                                    self.send(MioEvent::OutgoingConnectionDidConnect(
                                        addr,
                                        Err(err.to_string()),
                                    ));
                                    continue;
                                }
                            }
                        } else {
                            while let Some((buf, mut offset)) = connection.transmits.pop_front() {
                                match connection.stream.write(&buf[offset..]) {
                                    Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                                        connection.transmits.push_front((buf, offset));
                                        rereg = true;
                                        break;
                                    }
                                    Err(err) => {
                                        self.send(MioEvent::OutgoingDataDidSend(
                                            addr,
                                            Err(err.to_string()),
                                        ));
                                        // drop the connection
                                        continue 'events;
                                    }
                                    Ok(len) => {
                                        rereg = true;
                                        offset += len;
                                        if offset == buf.len() {
                                            self.send(MioEvent::OutgoingDataDidSend(addr, Ok(())));
                                        } else {
                                            connection.transmits.push_front((buf, offset));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    let interests = if connection.incoming_ready {
                        mio::Interest::WRITABLE
                    } else {
                        mio::Interest::READABLE | mio::Interest::WRITABLE
                    };
                    if rereg {
                        if let Err(err) = self.poll.registry().reregister(
                            &mut connection.stream,
                            event.token(),
                            interests,
                        ) {
                            self.send(MioEvent::ConnectionDidClose(addr, Err(err.to_string())));
                            continue;
                        }
                    }
                    self.connections.insert(addr, connection);
                }
            }
        }
        events.clear();
    }

    fn handle(&mut self, cmd: MioCmd) {
        use self::MioCmd::*;

        match cmd {
            ListenOn(addr) => match TcpListener::bind(addr) {
                Ok(mut listener) => {
                    if let Err(err) = self.poll.registry().register(
                        &mut listener,
                        self.tokens.register(Token::Listener(addr)),
                        mio::Interest::READABLE,
                    ) {
                        MioError::Listen(addr, err).report()
                    } else {
                        self.listeners.insert(
                            addr,
                            Listener {
                                inner: listener,
                                incomind_ready: false,
                            },
                        );
                    }
                }
                Err(err) => MioError::Listen(addr, err).report(),
            },
            Accept(listener_addr) => {
                if let Some(mut listener) = self.listeners.remove(&listener_addr) {
                    match listener.inner.accept() {
                        Ok((mut stream, addr)) => {
                            listener.incomind_ready = false;
                            if let Err(err) = self.poll.registry().register(
                                &mut stream,
                                self.tokens.register(Token::Connection(addr)),
                                mio::Interest::READABLE,
                            ) {
                                self.send(MioEvent::IncomingConnectionDidAccept(
                                    Some(addr),
                                    Err(err.to_string()),
                                ));
                            } else {
                                self.send(MioEvent::IncomingConnectionDidAccept(
                                    Some(addr),
                                    Ok(()),
                                ));
                                let connection = Connection {
                                    stream,
                                    transmits: VecDeque::default(),
                                    connected: true,
                                    incoming_ready: false,
                                };
                                self.connections.insert(addr, connection);
                            }
                            let token = self.tokens.register(Token::Listener(listener_addr));
                            if let Err(err) = self.poll.registry().reregister(
                                &mut listener.inner,
                                token,
                                mio::Interest::READABLE,
                            ) {
                                MioError::Listen(addr, err).report();
                            } else {
                                self.listeners.insert(listener_addr, listener);
                            }
                        }
                        Err(err) => {
                            self.send(MioEvent::IncomingConnectionDidAccept(
                                None,
                                Err(err.to_string()),
                            ));
                        }
                    }
                } else {
                    self.send(MioEvent::IncomingConnectionDidAccept(
                        None,
                        Err("no such listener".to_owned()),
                    ));
                }
            }
            Refuse(addr) => {
                if let Some(listener) = self.listeners.get_mut(&addr) {
                    if let Ok((stream, _)) = listener.inner.accept() {
                        listener.incomind_ready = false;
                        stream.shutdown(Shutdown::Both).unwrap_or_default();
                    }
                } else {
                    self.send(MioEvent::IncomingConnectionDidAccept(
                        None,
                        Err("no such listener".to_owned()),
                    ));
                }
            }
            Connect(addr) => {
                match TcpStream::connect(addr) {
                    Ok(mut stream) => {
                        if let Err(err) = self.poll.registry().register(
                            &mut stream,
                            self.tokens.register(Token::Connection(addr)),
                            mio::Interest::WRITABLE,
                        ) {
                            self.send(MioEvent::OutgoingConnectionDidConnect(
                                addr,
                                Err(err.to_string()),
                            ));
                        } else {
                            self.connections.insert(
                                addr,
                                Connection {
                                    stream,
                                    transmits: VecDeque::default(),
                                    connected: false,
                                    incoming_ready: false,
                                },
                            );
                        }
                    }
                    Err(err) => self.send(MioEvent::OutgoingConnectionDidConnect(
                        addr,
                        Err(err.to_string()),
                    )),
                };
            }
            Recv(addr, mut buf) => {
                if let Some(mut connection) = self.connections.remove(&addr) {
                    let mut keep = false;
                    match connection.stream.read(&mut buf) {
                        Ok(0) => self.send(MioEvent::ConnectionDidClose(addr, Ok(()))),
                        Ok(read) => {
                            self.send(MioEvent::IncomingDataDidReceive(
                                addr,
                                Ok(buf[..read].to_vec().into()),
                            ));
                            self.send(MioEvent::IncomingDataIsReady(addr));
                            keep = true;
                        }
                        Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                            connection.incoming_ready = false;
                            keep = true;
                        }
                        Err(err) => {
                            self.send(MioEvent::IncomingDataDidReceive(addr, Err(err.to_string())));
                            self.send(MioEvent::ConnectionDidClose(addr, Ok(())));
                        }
                    };

                    if keep {
                        let interests =
                            match (connection.incoming_ready, connection.transmits.is_empty()) {
                                (false, false) => {
                                    Some(mio::Interest::READABLE | mio::Interest::WRITABLE)
                                }
                                (false, true) => Some(mio::Interest::READABLE),
                                (true, false) => Some(mio::Interest::WRITABLE),
                                (true, true) => None,
                            };

                        if let Some(interests) = interests {
                            let token = self.tokens.register(Token::Connection(addr));
                            self.poll
                                .registry()
                                .reregister(&mut connection.stream, token, interests)
                                .unwrap();
                        }
                        self.connections.insert(addr, connection);
                    }
                } else {
                    self.send(MioEvent::IncomingDataDidReceive(
                        addr,
                        Err("not connected".to_string()),
                    ));
                }
            }
            Send(addr, buf) => {
                if let Some(connection) = self.connections.get_mut(&addr) {
                    connection.transmits.push_back((buf, 0));
                    let interests =
                        match (connection.incoming_ready, connection.transmits.is_empty()) {
                            (false, false) => {
                                Some(mio::Interest::READABLE | mio::Interest::WRITABLE)
                            }
                            (false, true) => Some(mio::Interest::READABLE),
                            (true, false) => Some(mio::Interest::WRITABLE),
                            (true, true) => None,
                        };
                    if let Some(interests) = interests {
                        let token = self.tokens.register(Token::Connection(addr));
                        self.poll
                            .registry()
                            .reregister(&mut connection.stream, token, interests)
                            .unwrap();
                    }
                } else {
                    self.send(MioEvent::OutgoingDataDidSend(
                        addr,
                        Err("not connected".to_string()),
                    ));
                }
            }
            Disconnect(addr) => {
                if let Some(connection) = self.connections.remove(&addr) {
                    connection
                        .stream
                        .shutdown(Shutdown::Both)
                        .unwrap_or_default();
                }
            }
        }
    }

    pub fn send(&self, event: MioEvent) {
        (self.event_sender)(event);
    }
}
