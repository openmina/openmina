use std::{
    collections::{VecDeque, BTreeMap, BTreeSet},
    task::{Waker, Context, Poll},
    io,
    time::Duration,
    sync::Arc,
};

use libp2p::{
    swarm::{
        ConnectionHandler, SubstreamProtocol, KeepAlive, ConnectionHandlerEvent,
        handler::ConnectionEvent,
    },
    core::{upgrade::ReadyUpgrade, Negotiated, muxing::SubstreamBox},
};

use super::{
    stream::{Stream, StreamEvent},
    behaviour::{StreamId, Event},
};

#[derive(Debug)]
pub enum Command {
    Send { stream_id: StreamId, bytes: Vec<u8> },
    Open { outgoing_stream_id: u32 },
}

pub struct Handler {
    menu: Arc<BTreeSet<(&'static str, i32)>>,
    streams: BTreeMap<StreamId, Stream>,
    last_outgoing_id: VecDeque<u32>,
    last_incoming_id: u32,

    failed: Vec<StreamId>,

    waker: Option<Waker>,
}

impl Handler {
    const PROTOCOL_NAME: [u8; 15] = *b"coda/rpcs/0.0.1";

    pub fn new(menu: Arc<BTreeSet<(&'static str, i32)>>) -> Self {
        Handler {
            menu,
            streams: BTreeMap::default(),
            last_outgoing_id: VecDeque::default(),
            last_incoming_id: 0,
            failed: Vec::default(),
            waker: None,
        }
    }

    fn add_stream(&mut self, incoming: bool, io: Negotiated<SubstreamBox>) {
        if incoming {
            let id = self.last_incoming_id;
            self.last_incoming_id += 1;
            let mut stream = Stream::new_incoming(self.menu.clone());
            stream.negotiated(io);
            self.streams.insert(StreamId::Incoming(id), stream);
            self.waker.as_ref().map(Waker::wake_by_ref);
        } else if let Some(id) = self.last_outgoing_id.pop_front() {
            if let Some(stream) = self.streams.get_mut(&StreamId::Outgoing(id)) {
                stream.negotiated(io);
                self.waker.as_ref().map(Waker::wake_by_ref);
            }
        }
    }
}

impl ConnectionHandler for Handler {
    type InEvent = Command;
    type OutEvent = Event;
    type Error = io::Error;
    type InboundProtocol = ReadyUpgrade<[u8; 15]>;
    type OutboundProtocol = ReadyUpgrade<[u8; 15]>;
    type OutboundOpenInfo = ();
    type InboundOpenInfo = ();

    fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol, Self::InboundOpenInfo> {
        SubstreamProtocol::new(ReadyUpgrade::new(Self::PROTOCOL_NAME), ())
            .with_timeout(Duration::from_secs(15))
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
            Self::OutEvent,
            Self::Error,
        >,
    > {
        for stream_id in &self.failed {
            self.streams.remove(stream_id);
        }
        self.failed.clear();

        let outbound_request = ConnectionHandlerEvent::OutboundSubstreamRequest {
            protocol: SubstreamProtocol::new(ReadyUpgrade::new(Self::PROTOCOL_NAME), ()),
        };
        for (stream_id, stream) in &mut self.streams {
            match stream.poll_stream(*stream_id, cx) {
                Poll::Pending => {}
                Poll::Ready(Ok(StreamEvent::Request(id))) => {
                    self.last_outgoing_id.push_back(id);
                    return Poll::Ready(outbound_request);
                }
                Poll::Ready(Ok(StreamEvent::Event(event))) => {
                    return Poll::Ready(ConnectionHandlerEvent::Custom(event));
                }
                Poll::Ready(Err(_err)) => {
                    self.failed.push(*stream_id);
                    // return Poll::Ready(ConnectionHandlerEvent::Close(err));
                }
            }
        }

        self.waker = Some(cx.waker().clone());
        Poll::Pending
    }

    fn on_behaviour_event(&mut self, event: Self::InEvent) {
        match event {
            Command::Open { outgoing_stream_id } => {
                self.streams.insert(
                    StreamId::Outgoing(outgoing_stream_id),
                    Stream::new_outgoing(true),
                );
            }
            Command::Send { stream_id, bytes } => {
                if let Some(stream) = self.streams.get_mut(&stream_id) {
                    stream.add(bytes);
                } else if let StreamId::Outgoing(id) = stream_id {
                    // implicitly open outgoing stream
                    self.last_outgoing_id.push_back(id);
                    let mut stream = Stream::new_outgoing(false);
                    stream.add(bytes);
                    self.streams.insert(stream_id, stream);
                }
            }
        }
        self.waker.as_ref().map(Waker::wake_by_ref);
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
            ConnectionEvent::FullyNegotiatedInbound(io) => self.add_stream(true, io.protocol),
            ConnectionEvent::FullyNegotiatedOutbound(io) => self.add_stream(false, io.protocol),
            _ => {}
        }
    }
}
