use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::Arc,
    task::{Context, Poll, Waker},
};

use libp2p::{
    core::Endpoint,
    swarm::{
        derive_prelude::ConnectionEstablished, ConnectionClosed, ConnectionDenied, ConnectionId,
        FromSwarm, NetworkBehaviour, NotifyHandler, PollParameters, THandler, THandlerInEvent,
        THandlerOutEvent, ToSwarm,
    },
    Multiaddr, PeerId,
};

use mina_p2p_messages::binprot::{self, BinProtWrite};
use mina_p2p_messages::rpc_kernel::{
    Error, Message, NeedsLength, Query, Response, RpcMethod, RpcResult,
};

use super::{
    handler::{Command, Handler},
    state::Received,
};

#[derive(Default)]
pub struct BehaviourBuilder {
    menu: BTreeSet<(&'static str, i32)>,
}

impl BehaviourBuilder {
    pub fn register_method<M>(mut self) -> Self
    where
        M: RpcMethod,
    {
        self.menu.insert((M::NAME, M::VERSION));
        self
    }

    pub fn build(self) -> Behaviour {
        Behaviour {
            menu: Arc::new(self.menu),
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct Behaviour {
    menu: Arc<BTreeSet<(&'static str, i32)>>,
    peers: BTreeMap<PeerId, ConnectionId>,
    queue: VecDeque<ToSwarm<(PeerId, Event), Command>>,
    pending: BTreeMap<PeerId, VecDeque<Command>>,
    waker: Option<Waker>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StreamId {
    Incoming(u32),
    Outgoing(u32),
}

#[derive(Debug)]
pub enum Event {
    ConnectionEstablished,
    ConnectionClosed,
    Stream {
        stream_id: StreamId,
        received: Received,
    },
}

impl Behaviour {
    fn dispatch_command(&mut self, peer_id: PeerId, command: Command) {
        if let Some(connection_id) = self.peers.get(&peer_id) {
            self.queue.push_back(ToSwarm::NotifyHandler {
                peer_id,
                handler: NotifyHandler::One(*connection_id),
                event: command,
            });
            self.waker.as_ref().map(Waker::wake_by_ref);
        } else {
            self.pending.entry(peer_id).or_default().push_back(command);
        }
    }

    pub fn open(&mut self, peer_id: PeerId, outgoing_stream_id: u32) {
        self.dispatch_command(peer_id, Command::Open { outgoing_stream_id })
    }

    pub fn respond<M>(
        &mut self,
        peer_id: PeerId,
        stream_id: StreamId,
        id: i64,
        response: Result<M::Response, Error>,
    ) -> Result<(), binprot::Error>
    where
        M: RpcMethod,
    {
        let data = RpcResult(response.map(NeedsLength));
        let msg = Message::<M::Response>::Response(Response { id, data });
        let mut bytes = vec![0; 8];
        msg.binprot_write(&mut bytes)?;
        let len = (bytes.len() - 8) as u64;
        bytes[..8].clone_from_slice(&len.to_le_bytes());

        self.dispatch_command(peer_id, Command::Send { stream_id, bytes });

        Ok(())
    }

    pub fn query<M>(
        &mut self,
        peer_id: PeerId,
        stream_id: StreamId,
        id: i64,
        query: M::Query,
    ) -> Result<(), binprot::Error>
    where
        M: RpcMethod,
    {
        let msg = Message::<M::Query>::Query(Query {
            tag: M::NAME.into(),
            version: M::VERSION,
            id,
            data: NeedsLength(query),
        });
        let mut bytes = vec![0; 8];
        msg.binprot_write(&mut bytes)?;
        let len = (bytes.len() - 8) as u64;
        bytes[..8].clone_from_slice(&len.to_le_bytes());

        self.dispatch_command(peer_id, Command::Send { stream_id, bytes });

        Ok(())
    }
}

impl NetworkBehaviour for Behaviour {
    type ConnectionHandler = Handler;
    type OutEvent = (PeerId, Event);

    fn handle_established_inbound_connection(
        &mut self,
        connection_id: ConnectionId,
        peer: PeerId,
        _local_addr: &Multiaddr,
        _remote_addr: &Multiaddr,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        self.peers.insert(peer, connection_id);
        Ok(Handler::new(self.menu.clone()))
    }

    fn handle_established_outbound_connection(
        &mut self,
        connection_id: ConnectionId,
        peer: PeerId,
        _addr: &Multiaddr,
        _role_override: Endpoint,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        self.peers.insert(peer, connection_id);
        Ok(Handler::new(self.menu.clone()))
    }

    fn on_swarm_event(&mut self, event: FromSwarm<Self::ConnectionHandler>) {
        match event {
            FromSwarm::ConnectionEstablished(ConnectionEstablished {
                peer_id,
                connection_id,
                ..
            }) => {
                self.peers.insert(peer_id, connection_id);
                self.queue.push_back(ToSwarm::GenerateEvent((
                    peer_id,
                    Event::ConnectionEstablished,
                )));
                if let Some(queue) = self.pending.remove(&peer_id) {
                    for command in queue {
                        self.queue.push_back(ToSwarm::NotifyHandler {
                            peer_id,
                            handler: NotifyHandler::One(connection_id),
                            event: command,
                        });
                    }
                }
                self.waker.as_ref().map(Waker::wake_by_ref);
            }
            FromSwarm::ConnectionClosed(ConnectionClosed {
                peer_id,
                connection_id,
                ..
            }) => {
                if self.peers.get(&peer_id) == Some(&connection_id) {
                    self.peers.remove(&peer_id);
                }
                self.queue
                    .push_back(ToSwarm::GenerateEvent((peer_id, Event::ConnectionClosed)));
                self.waker.as_ref().map(Waker::wake_by_ref);
            }
            _ => {}
        }
    }

    fn on_connection_handler_event(
        &mut self,
        peer_id: PeerId,
        connection_id: ConnectionId,
        event: THandlerOutEvent<Self>,
    ) {
        self.peers.insert(peer_id, connection_id);
        self.queue
            .push_back(ToSwarm::GenerateEvent((peer_id, event)));
        self.waker.as_ref().map(Waker::wake_by_ref);
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
        _params: &mut impl PollParameters,
    ) -> Poll<ToSwarm<Self::OutEvent, THandlerInEvent<Self>>> {
        if let Some(event) = self.queue.pop_front() {
            Poll::Ready(event)
        } else {
            self.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
