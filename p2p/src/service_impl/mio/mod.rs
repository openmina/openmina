mod token;
use self::token::{Token, TokenRegistry};

use std::{
    collections::{BTreeMap, VecDeque},
    io::{self, Read, Write},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr},
    process,
    sync::mpsc,
    thread,
};

use mio::net::{TcpListener, TcpStream};

use thiserror::Error;

use crate::{MioEvent, P2pEvent};

/// The state machine sends commands to the service.
enum Cmd {
    /// Bind a new listener to a new socket on the interface
    /// that previously reported by an `InterfaceAppear` event with the ip.
    ListenOn(SocketAddr),
    /// Accept an incoming connection that previously reported
    /// by an `IncomingConnectionIsReady` event with the listener address.
    Accept(SocketAddr),
    /// Refuse to connect the incoming connection previously reported.
    Refuse(SocketAddr),
    /// Create a new outgoing connection to the socket.
    Connect(SocketAddr),
    /// Receive some data from the connection in the buffer.
    Recv(SocketAddr, Box<[u8]>),
    /// Send the data in the connection.
    Send(SocketAddr, Box<[u8]>),
    /// Disconnect the remote peer.
    Disconnect(SocketAddr),
}

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
    cmd_sender: mpsc::Sender<Cmd>,
    waker: mio::Waker,
}

impl MioService {
    pub fn run<E>(event_sender: mpsc::Sender<E>) -> Self
    where
        E: 'static + Send + From<P2pEvent>,
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
            inner.send(MioEvent::InterfaceDetected(IpAddr::V6(
                Ipv6Addr::UNSPECIFIED,
            )));

            let mut events = mio::Events::with_capacity(1024);

            loop {
                inner.run(&mut events);
            }
        });

        MioService {
            cmd_sender: tx,
            waker,
        }
    }

    pub fn listen_on(&self, addr: SocketAddr) {
        self.cmd_sender
            .send(Cmd::ListenOn(addr))
            .unwrap_or_default();
        self.waker.wake().unwrap_or_default();
    }

    pub fn accept(&self, addr: SocketAddr) {
        self.cmd_sender.send(Cmd::Accept(addr)).unwrap_or_default();
        self.waker.wake().unwrap_or_default();
    }

    pub fn refuse(&self, addr: SocketAddr) {
        self.cmd_sender.send(Cmd::Refuse(addr)).unwrap_or_default();
        self.waker.wake().unwrap_or_default();
    }

    pub fn connect(&self, addr: SocketAddr) {
        self.cmd_sender.send(Cmd::Connect(addr)).unwrap_or_default();
        self.waker.wake().unwrap_or_default();
    }

    pub fn recv(&self, addr: SocketAddr, data: Box<[u8]>) {
        self.cmd_sender
            .send(Cmd::Recv(addr, data))
            .unwrap_or_default();
        self.waker.wake().unwrap_or_default();
    }

    pub fn send(&self, addr: SocketAddr, data: Box<[u8]>) {
        self.cmd_sender
            .send(Cmd::Send(addr, data))
            .unwrap_or_default();
        self.waker.wake().unwrap_or_default();
    }

    pub fn disconnect(&self, addr: SocketAddr) {
        self.cmd_sender
            .send(Cmd::Disconnect(addr))
            .unwrap_or_default();
        self.waker.wake().unwrap_or_default();
    }
}

struct MioServiceInner<E> {
    poll: mio::Poll,
    event_sender: mpsc::Sender<E>,
    cmd_receiver: mpsc::Receiver<Cmd>,
    tokens: TokenRegistry,
    listeners: BTreeMap<SocketAddr, TcpListener>,
    connections: BTreeMap<SocketAddr, Connection>,
}

struct Connection {
    stream: TcpStream,
    transmits: VecDeque<(Box<[u8]>, usize)>,
    connected: bool,
}

impl<E> MioServiceInner<E>
where
    E: From<P2pEvent>,
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
                    if event.is_readable() {
                        self.send(MioEvent::IncomingConnectionIsReady { listener: addr });
                    }
                    if let Some(listener) = self.listeners.get_mut(&addr) {
                        if let Err(err) = self.poll.registry().reregister(
                            listener,
                            event.token(),
                            mio::Interest::READABLE,
                        ) {
                            self.listeners.remove(&addr);
                            MioError::Listen(addr, err).report();
                        }
                    }
                }
                Some(Token::Connection(mut addr)) => {
                    let mut connection = self.connections.remove(&addr).expect("must be here");
                    if event.is_readable() {
                        self.send(MioEvent::IncomingDataIsReady(addr));
                    }
                    if event.is_writable() {
                        if !connection.connected {
                            match connection.stream.peer_addr() {
                                Ok(new_addr) => {
                                    connection.connected = true;
                                    addr = new_addr;
                                    self.send(MioEvent::OutgoingConnectionDidConnect(addr, Ok(())));
                                }
                                Err(err) if err.kind() == io::ErrorKind::NotConnected => {}
                                #[cfg(unix)]
                                Err(err) if err.raw_os_error() == Some(libc::EINPROGRESS) => {}
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
                                        break;
                                    }
                                    Err(err) => {
                                        self.send(MioEvent::OutgoingDataDidSend(
                                            addr,
                                            Err(err.to_string()),
                                        ));
                                        continue 'events;
                                    }
                                    Ok(len) => {
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
                    if let Err(err) = self.poll.registry().reregister(
                        &mut connection.stream,
                        event.token(),
                        mio::Interest::READABLE | mio::Interest::WRITABLE,
                    ) {
                        self.send(MioEvent::ConnectionDidClose(addr, Err(err.to_string())));
                    } else {
                        self.connections.insert(addr, connection);
                    }
                }
            }
        }
        events.clear();
    }

    fn handle(&mut self, cmd: Cmd) {
        match cmd {
            Cmd::ListenOn(addr) => match TcpListener::bind(addr) {
                Ok(mut listener) => {
                    if let Err(err) = self.poll.registry().register(
                        &mut listener,
                        self.tokens.register(Token::Listener(addr)),
                        mio::Interest::READABLE,
                    ) {
                        MioError::Listen(addr, err).report()
                    } else {
                        self.listeners.insert(addr, listener);
                    }
                }
                Err(err) => MioError::Listen(addr, err).report(),
            },
            Cmd::Accept(listener_addr) => {
                if let Some(listener) = self.listeners.get_mut(&listener_addr) {
                    match listener.accept() {
                        Ok((mut stream, addr)) => {
                            if let Err(err) = self.poll.registry().register(
                                &mut stream,
                                self.tokens.register(Token::Connection(addr)),
                                mio::Interest::READABLE | mio::Interest::WRITABLE,
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
                                };
                                self.connections.insert(addr, connection);
                            }
                        }
                        Err(err) => {
                            self.send(MioEvent::IncomingConnectionDidAccept(
                                None,
                                Err(err.to_string()),
                            ));
                            self.listeners.remove(&listener_addr);
                        }
                    }
                } else {
                    self.send(MioEvent::IncomingConnectionDidAccept(
                        None,
                        Err("no such listener".to_owned()),
                    ));
                }
            }
            Cmd::Refuse(addr) => {
                if let Some(listener) = self.listeners.get_mut(&addr) {
                    if let Ok((stream, _)) = listener.accept() {
                        stream.shutdown(Shutdown::Both).unwrap_or_default();
                    }
                } else {
                    self.send(MioEvent::IncomingConnectionDidAccept(
                        None,
                        Err("no such listener".to_owned()),
                    ));
                }
            }
            Cmd::Connect(addr) => {
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
            Cmd::Recv(addr, mut buf) => {
                if let Some(connection) = self.connections.get_mut(&addr) {
                    let res = connection
                        .stream
                        .read(&mut buf)
                        .map_err(|err| err.to_string())
                        .map(|len| (buf, len));
                    self.send(MioEvent::IncomingDataDidReceive(addr, res));
                } else {
                    self.send(MioEvent::IncomingDataDidReceive(
                        addr,
                        Err("not connected".to_string()),
                    ));
                }
            }
            Cmd::Send(addr, buf) => {
                if let Some(connection) = self.connections.get_mut(&addr) {
                    connection.transmits.push_back((buf, 0));
                } else {
                    self.send(MioEvent::OutgoingDataDidSend(
                        addr,
                        Err("not connected".to_string()),
                    ));
                }
            }
            Cmd::Disconnect(addr) => {
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
        self.event_sender
            .send(P2pEvent::MioEvent(event).into())
            .unwrap_or_default();
    }
}
