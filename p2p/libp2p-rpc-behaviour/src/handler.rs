use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    io,
    sync::Arc,
    task::{Context, Poll, Waker},
    time::Duration,
};

use libp2p::{
    core::upgrade::ReadyUpgrade,
    swarm::{
        handler::{ConnectionEvent, InboundUpgradeSend},
        ConnectionHandler, ConnectionHandlerEvent, KeepAlive, SubstreamProtocol,
    },
    StreamProtocol,
};

use super::{
    behaviour::{Event, StreamId},
    stream::{Stream, StreamEvent},
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
    const PROTOCOL_NAME: &'static str = "coda/rpcs/0.0.1";

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

    fn add_stream(
        &mut self,
        incoming: bool,
        io: <ReadyUpgrade<StreamProtocol> as InboundUpgradeSend>::Output,
    ) {
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
    type FromBehaviour = Command;
    type ToBehaviour = Event;
    type Error = io::Error;
    type InboundProtocol = ReadyUpgrade<StreamProtocol>;
    type OutboundProtocol = ReadyUpgrade<StreamProtocol>;
    type OutboundOpenInfo = ();
    type InboundOpenInfo = ();

    fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol, Self::InboundOpenInfo> {
        SubstreamProtocol::new(
            ReadyUpgrade::new(StreamProtocol::new(Self::PROTOCOL_NAME)),
            (),
        )
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
            Self::ToBehaviour,
            Self::Error,
        >,
    > {
        for stream_id in &self.failed {
            self.streams.remove(stream_id);
        }
        self.failed.clear();

        let outbound_request = ConnectionHandlerEvent::OutboundSubstreamRequest {
            protocol: SubstreamProtocol::new(
                ReadyUpgrade::new(StreamProtocol::new(Self::PROTOCOL_NAME)),
                (),
            ),
        };
        for (stream_id, stream) in &mut self.streams {
            match stream.poll_stream(*stream_id, cx) {
                Poll::Pending => {}
                Poll::Ready(Ok(StreamEvent::Request(id))) => {
                    self.last_outgoing_id.push_back(id);
                    return Poll::Ready(outbound_request);
                }
                Poll::Ready(Ok(StreamEvent::Event(event))) => {
                    return Poll::Ready(ConnectionHandlerEvent::NotifyBehaviour(event));
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

    fn on_behaviour_event(&mut self, event: Self::FromBehaviour) {
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
