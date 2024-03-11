use std::{
    collections::BTreeSet,
    io,
    sync::Arc,
    task::{self, Context, Poll},
};

use libp2p::{core::upgrade::ReadyUpgrade, swarm::handler::InboundUpgradeSend, StreamProtocol};

use super::{
    behaviour::{Event, StreamId},
    state,
};

pub struct Stream {
    opening_state: Option<OpeningState>,
    inner_state: state::Inner,
}

enum OpeningState {
    Requested,
    Negotiated {
        io: <ReadyUpgrade<StreamProtocol> as InboundUpgradeSend>::Output,
    },
}

pub enum StreamEvent {
    Request(u32),
    Event(Event),
}

impl Stream {
    pub fn new_outgoing(ask_menu: bool) -> Self {
        Stream {
            opening_state: None,
            // empty menu for outgoing stream
            inner_state: state::Inner::new(Arc::new(BTreeSet::default()), ask_menu),
        }
    }

    pub fn new_incoming(menu: Arc<BTreeSet<(&'static str, i32)>>) -> Self {
        Stream {
            opening_state: None,
            inner_state: state::Inner::new(menu, false),
        }
    }

    pub fn negotiated(&mut self, io: <ReadyUpgrade<StreamProtocol> as InboundUpgradeSend>::Output) {
        self.opening_state = Some(OpeningState::Negotiated { io });
    }

    pub fn add(&mut self, bytes: Vec<u8>) {
        self.inner_state.add(bytes);
    }

    pub fn poll_stream(
        &mut self,
        stream_id: StreamId,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<StreamEvent>> {
        match &mut self.opening_state {
            None => {
                if let StreamId::Outgoing(id) = stream_id {
                    self.opening_state = Some(OpeningState::Requested);
                    Poll::Ready(Ok(StreamEvent::Request(id)))
                } else {
                    Poll::Pending
                }
            }
            Some(OpeningState::Requested) => Poll::Pending,
            Some(OpeningState::Negotiated { io }) => {
                let received = match task::ready!(self.inner_state.poll(cx, io)) {
                    Err(err) => {
                        if err.kind() == io::ErrorKind::UnexpectedEof {
                            if let StreamId::Outgoing(id) = stream_id {
                                log::warn!("reopen stream");
                                self.opening_state = Some(OpeningState::Requested);
                                return Poll::Ready(Ok(StreamEvent::Request(id)));
                            } else {
                                return Poll::Ready(Err(err));
                            }
                        } else {
                            return Poll::Ready(Err(err));
                        }
                    }
                    Ok(v) => v,
                };
                Poll::Ready(Ok(StreamEvent::Event(Event::Stream {
                    stream_id,
                    received,
                })))
            }
        }
    }
}
