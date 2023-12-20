mod token;
use self::token::{Token, TokenRegistry};

use std::{
    collections::BTreeMap,
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
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
    /// by an `IncomingConnectionIsReady` event with the address.
    Accept(SocketAddr),
    /// Refuse to connect the incoming connection previously reported.
    Refuse(SocketAddr),
    /// Create a new outgoing connection to the socket.
    Connect(SocketAddr),
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
    #[error("mio failed to accept connection from {0:?} on {1}, error: {2}")]
    Accept(Option<SocketAddr>, SocketAddr, io::Error),
    #[error("mio failed register stream {0}, error: {1}")]
    RegisterStream(SocketAddr, io::Error),
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
                std::process::exit(1);
            }
        };

        let mut tokens = TokenRegistry::default();
        let waker = match mio::Waker::new(poll.registry(), tokens.register(Token::Waker)) {
            Ok(v) => v,
            Err(err) => {
                MioError::Waker(err).report();
                std::process::exit(1);
            }
        };

        let (tx, rx) = mpsc::channel();

        let mut inner = MioServiceInner {
            poll,
            event_sender,
            cmd_receiver: rx,
            tokens,
            listeners: BTreeMap::default(),
            streams: BTreeMap::default(),
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
    streams: BTreeMap<SocketAddr, TcpStream>,
}

impl<E> MioServiceInner<E>
where
    E: From<P2pEvent>,
{
    fn run(&mut self, events: &mut mio::Events) {
        if let Err(err) = self.poll.poll(events, None) {
            MioError::Poll(err).report();
        }

        for event in events.iter() {
            match self.tokens.get(&event.token()) {
                None => {}
                Some(Token::Waker) => {
                    while let Ok(cmd) = self.cmd_receiver.try_recv() {
                        self.handle(cmd);
                    }
                }
                Some(Token::Listener(addr)) => {
                    if event.is_readable() {
                        self.send(MioEvent::IncomingConnectionIsReady(addr));
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
                Some(Token::Connection(addr)) => {
                    if event.is_readable() {
                        self.send(MioEvent::IncomingDataIsReady(addr));
                    }
                    if event.is_writable() {
                        //
                    }
                    if let Some(stream) = self.streams.get_mut(&addr) {
                        if let Err(err) = self.poll.registry().reregister(
                            stream,
                            event.token(),
                            mio::Interest::READABLE | mio::Interest::WRITABLE,
                        ) {
                            self.streams.remove(&addr);
                            self.send(MioEvent::ConnectionDidClose(addr, Err(err.to_string())));
                            MioError::RegisterStream(addr, err).report();
                        }
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
            Cmd::Accept(addr) => {
                if let Some(listener) = self.listeners.get_mut(&addr) {
                    match listener.accept() {
                        Ok((mut stream, new_addr)) => {
                            if let Err(err) = self.poll.registry().register(
                                &mut stream,
                                self.tokens.register(Token::Connection(addr)),
                                mio::Interest::READABLE | mio::Interest::WRITABLE,
                            ) {
                                self.send(MioEvent::IncomingConnectionDidAccept(
                                    Some(new_addr),
                                    Err(err.to_string()),
                                ));
                                MioError::Accept(Some(new_addr), addr, err).report()
                            } else {
                                self.send(MioEvent::IncomingConnectionDidAccept(
                                    Some(new_addr),
                                    Ok(()),
                                ));
                                self.streams.insert(addr, stream);
                            }
                        }
                        Err(err) => {
                            self.send(MioEvent::IncomingConnectionDidAccept(
                                None,
                                Err(err.to_string()),
                            ));
                            self.listeners.remove(&addr);
                            MioError::Accept(None, addr, err).report()
                        }
                    }
                }
            }
            _ => todo!(),
        }
    }

    pub fn send(&self, event: MioEvent) {
        self.event_sender
            .send(P2pEvent::MioEvent(event).into())
            .unwrap_or_default();
    }
}
