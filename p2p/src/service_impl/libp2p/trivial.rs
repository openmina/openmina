//! Trivial behavior. Gives access for state machine to a specific yamux stream.

use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    io,
    task::{Context, Poll, Waker},
};

use libp2p::{
    core::{upgrade::ReadyUpgrade, Endpoint, UpgradeInfo},
    futures::{future, io::AsyncWriteExt, AsyncReadExt},
    swarm::{
        handler::{
            ConnectionEvent, ConnectionHandler, DialUpgradeError, FullyNegotiatedInbound,
            FullyNegotiatedOutbound, ListenUpgradeError,
        },
        ConnectionDenied, ConnectionHandlerEvent, ConnectionId, FromSwarm, KeepAlive,
        NetworkBehaviour, NotifyHandler, PollParameters, Stream, StreamUpgradeError,
        SubstreamProtocol, THandler, THandlerInEvent, THandlerOutEvent, ToSwarm,
    },
    InboundUpgrade, Multiaddr, PeerId, StreamProtocol,
};
use tokio::sync::mpsc;
use void::Void;

#[derive(Debug)]
pub enum Event {
    StreamOpenError {
        protocol: StreamProtocol,
        error: StreamUpgradeError<Void>,
    },
    StreamOpened {
        protocol: StreamProtocol,
        incoming: bool,
    },
    MessageReceived(StreamProtocol, Box<[u8]>),
    StreamClosed(StreamProtocol, Option<io::Error>),
    ConnectionClosed,
}

#[derive(Debug)]
pub enum Command {
    OpenStream(StreamProtocol),
    SendMessage(StreamProtocol, Box<[u8]>),
    CloseStream(StreamProtocol),
    CloseConnection,
}

pub struct Behaviour<const N: usize> {
    protocols: [StreamProtocol; N],
    waker: Option<Waker>,
    queue: VecDeque<(PeerId, Event)>,
    command_queue: VecDeque<(PeerId, Command)>,
    handlers: BTreeMap<PeerId, ConnectionId>,
}

impl<const N: usize> Behaviour<N> {
    pub fn new(protocols: [StreamProtocol; N]) -> Self {
        Behaviour {
            protocols,
            waker: None,
            queue: VecDeque::default(),
            command_queue: VecDeque::default(),
            handlers: BTreeMap::default(),
        }
    }

    pub fn open(&mut self, peer_id: PeerId, protocol: StreamProtocol) {
        self.command_queue
            .push_back((peer_id, Command::OpenStream(protocol)));
        self.waker.take().map(Waker::wake);
    }

    pub fn send(&mut self, peer_id: PeerId, protocol: StreamProtocol, data: Box<[u8]>) {
        self.command_queue
            .push_back((peer_id, Command::SendMessage(protocol, data)));
        self.waker.take().map(Waker::wake);
    }
}

impl<const N: usize> NetworkBehaviour for Behaviour<N> {
    type ConnectionHandler = Handler<N>;

    type ToSwarm = (PeerId, Event);

    fn handle_established_outbound_connection(
        &mut self,
        connection_id: ConnectionId,
        peer_id: PeerId,
        _addr: &Multiaddr,
        _role_override: Endpoint,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        self.handlers.insert(peer_id, connection_id);
        Ok(Handler::new(self.protocols.clone()))
    }

    fn handle_established_inbound_connection(
        &mut self,
        connection_id: ConnectionId,
        peer_id: PeerId,
        _local_addr: &Multiaddr,
        _remote_addr: &Multiaddr,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        if self.handlers.contains_key(&peer_id) {
            return Err(ConnectionDenied::new(
                "only one connection with the same peer allowed",
            ));
        }
        self.handlers.insert(peer_id, connection_id);
        Ok(Handler::new(self.protocols.clone()))
    }

    fn on_swarm_event(&mut self, event: FromSwarm<Self::ConnectionHandler>) {
        // nothing to do
        let _ = event;
    }

    fn on_connection_handler_event(
        &mut self,
        peer_id: PeerId,
        _connection_id: ConnectionId,
        event: THandlerOutEvent<Self>,
    ) {
        self.queue.push_back((peer_id, event));
        self.waker.take().map(Waker::wake);
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
        _params: &mut impl PollParameters,
    ) -> Poll<ToSwarm<Self::ToSwarm, THandlerInEvent<Self>>> {
        if let Some((peer_id, command)) = self.command_queue.pop_front() {
            if let Some(connection_id) = self.handlers.get(&peer_id) {
                return Poll::Ready(ToSwarm::NotifyHandler {
                    peer_id,
                    handler: NotifyHandler::One(*connection_id),
                    event: command,
                });
            }
        }

        if let Some(event) = self.queue.pop_front() {
            return Poll::Ready(ToSwarm::GenerateEvent(event));
        }

        self.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}

pub struct Handler<const N: usize> {
    protocols: [StreamProtocol; N],
    stream: HashMap<StreamProtocol, mpsc::UnboundedSender<Box<[u8]>>>,
    tx: mpsc::UnboundedSender<ThisConnectionHandlerEvent>,
    rx: mpsc::UnboundedReceiver<ThisConnectionHandlerEvent>,
}

type ThisConnectionHandlerEvent =
    ConnectionHandlerEvent<ReadyUpgrade<StreamProtocol>, StreamProtocol, Event, io::Error>;

impl<const N: usize> Handler<N> {
    pub fn new(protocols: [StreamProtocol; N]) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Handler {
            protocols,
            stream: HashMap::default(),
            tx,
            rx,
        }
    }

    fn notify_behaviour(&self, event: Event) {
        self.tx
            .send(ConnectionHandlerEvent::NotifyBehaviour(event))
            .unwrap_or_default();
    }
}

/// Select one of the list of protocols
/// `InboundUpgrade` will return the stream along with the name of the protocol negotiated
pub struct UpgradeInfoList<const N: usize>([StreamProtocol; N]);

impl<const N: usize> UpgradeInfo for UpgradeInfoList<N> {
    type Info = StreamProtocol;

    type InfoIter = [StreamProtocol; N];

    fn protocol_info(&self) -> Self::InfoIter {
        self.0.clone()
    }
}

impl<const N: usize, C> InboundUpgrade<C> for UpgradeInfoList<N> {
    type Output = (C, StreamProtocol);
    type Error = Void;
    type Future = future::Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_inbound(self, socket: C, info: Self::Info) -> Self::Future {
        future::ready(Ok((socket, info)))
    }
}

impl<const N: usize> ConnectionHandler for Handler<N> {
    type FromBehaviour = Command;

    type ToBehaviour = Event;

    type Error = io::Error;

    type InboundProtocol = UpgradeInfoList<N>;

    type OutboundProtocol = ReadyUpgrade<StreamProtocol>;

    type InboundOpenInfo = ();

    type OutboundOpenInfo = StreamProtocol;

    fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol, Self::InboundOpenInfo> {
        SubstreamProtocol::new(UpgradeInfoList(self.protocols.clone()), ())
    }

    fn connection_keep_alive(&self) -> KeepAlive {
        KeepAlive::Yes
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<
        ConnectionHandlerEvent<
            Self::OutboundProtocol,
            Self::OutboundOpenInfo,
            Self::ToBehaviour,
            Self::Error,
        >,
    > {
        self.rx.poll_recv(cx).map(|x| {
            x.unwrap_or_else(|| ConnectionHandlerEvent::Close(io::ErrorKind::Other.into()))
        })
    }

    fn on_behaviour_event(&mut self, event: Self::FromBehaviour) {
        match event {
            Command::OpenStream(protocol) => {
                let protocol =
                    SubstreamProtocol::new(ReadyUpgrade::new(protocol.clone()), protocol);
                self.tx
                    .send(ConnectionHandlerEvent::OutboundSubstreamRequest { protocol })
                    .unwrap_or_default();
            }
            Command::SendMessage(protocol, message) => {
                if let Some(tx) = self.stream.get(&protocol) {
                    tx.send(message).unwrap_or_default();
                } else {
                    // no such stream, the message is lost
                }
            }
            Command::CloseStream(protocol) => drop(self.stream.remove(&protocol)),
            Command::CloseConnection => {
                self.tx
                    .send(ConnectionHandlerEvent::Close(io::ErrorKind::Other.into()))
                    .unwrap_or_default();
            }
        }
    }

    fn on_connection_event(
        &mut self,
        event: ConnectionEvent<
            Self::InboundProtocol,
            Self::OutboundProtocol,
            Self::InboundOpenInfo,
            Self::OutboundOpenInfo,
        >,
    ) {
        match event {
            ConnectionEvent::FullyNegotiatedInbound(FullyNegotiatedInbound {
                protocol: (protocol, info),
                info: (),
            }) => {
                let (tx, rx) = mpsc::unbounded_channel();
                self.stream.insert(info.clone(), tx);
                tokio::spawn(handle_stream(info.clone(), protocol, rx, self.tx.clone()));
                self.notify_behaviour(Event::StreamOpened {
                    protocol: info,
                    incoming: true,
                });
            }
            ConnectionEvent::FullyNegotiatedOutbound(FullyNegotiatedOutbound {
                info,
                protocol,
            }) => {
                let (tx, rx) = mpsc::unbounded_channel();
                self.stream.insert(info.clone(), tx);
                tokio::spawn(handle_stream(info.clone(), protocol, rx, self.tx.clone()));
                self.notify_behaviour(Event::StreamOpened {
                    protocol: info,
                    incoming: false,
                });
            }
            ConnectionEvent::DialUpgradeError(DialUpgradeError { error, info }) => {
                self.stream.remove(&info);
                self.notify_behaviour(Event::StreamOpenError {
                    protocol: info,
                    error,
                });
            }
            ConnectionEvent::ListenUpgradeError(ListenUpgradeError { error, info: () }) => {
                let _ = error;
                // cannot happen, the type of `error` is `Void` which cannot be constructed
            }
            _ => (),
        }
    }
}

async fn handle_stream(
    protocol: StreamProtocol,
    stream: Stream,
    mut rx: mpsc::UnboundedReceiver<Box<[u8]>>,
    tx: mpsc::UnboundedSender<ThisConnectionHandlerEvent>,
) {
    let (mut read, mut write) = stream.split();

    tokio::spawn({
        let tx = tx.clone();
        let protocol = protocol.clone();
        async move {
            loop {
                let mut buffer = vec![0; 0x10000];
                match read.read(&mut buffer).await {
                    Ok(0) => break,
                    Ok(len) => {
                        buffer.resize(len, 0);

                        tx.send(ConnectionHandlerEvent::NotifyBehaviour(
                            Event::MessageReceived(protocol.clone(), buffer.into_boxed_slice()),
                        ))
                        .unwrap_or_default()
                    }
                    Err(error) => {
                        tx.send(ConnectionHandlerEvent::NotifyBehaviour(
                            Event::StreamClosed(protocol, Some(error)),
                        ))
                        .unwrap_or_default();
                        break;
                    }
                }
            }
        }
    });

    while let Some(msg) = rx.blocking_recv() {
        write.write_all(&msg).await.unwrap();
    }
    let error = write.close().await.err();
    tx.send(ConnectionHandlerEvent::NotifyBehaviour(
        Event::StreamClosed(protocol, error),
    ))
    .unwrap_or_default();
}
