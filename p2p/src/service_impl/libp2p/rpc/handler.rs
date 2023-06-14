pub use super::protocol::{RequestProtocol, ResponseProtocol};

use libp2p::swarm::handler::ConnectionEvent;

use super::EMPTY_QUEUE_SHRINK_THRESHOLD;

use libp2p::core::upgrade::{NegotiationError, UpgradeError};
use libp2p::futures::{channel::oneshot, future::BoxFuture, prelude::*, stream::FuturesUnordered};
use libp2p::swarm::{
    handler::{ConnectionHandler, ConnectionHandlerEvent, ConnectionHandlerUpgrErr, KeepAlive},
    SubstreamProtocol,
};
use std::time::Duration;
use std::{
    collections::VecDeque,
    fmt, io,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    task::{Context, Poll},
};

use crate::channels::rpc::{P2pRpcId, P2pRpcRequest, P2pRpcResponse};

/// A connection handler of a `RequestResponse` protocol.
#[doc(hidden)]
pub struct RequestResponseHandler {
    /// A pending fatal error that results in the connection being closed.
    pending_error: Option<ConnectionHandlerUpgrErr<io::Error>>,
    /// Queue of events to emit in `poll()`.
    pending_events: VecDeque<RequestResponseHandlerEvent>,
    /// Outbound upgrades waiting to be emitted as an `OutboundSubstreamRequest`.
    outbound: VecDeque<RequestProtocol>,
    /// Inbound upgrades waiting for the incoming request.
    inbound: FuturesUnordered<
        BoxFuture<
            'static,
            Result<((P2pRpcId, P2pRpcRequest), oneshot::Sender<P2pRpcResponse>), oneshot::Canceled>,
        >,
    >,
    inbound_request_id: Arc<AtomicU32>,
}

impl RequestResponseHandler {
    pub(super) fn new(inbound_request_id: Arc<AtomicU32>) -> Self {
        Self {
            outbound: VecDeque::new(),
            inbound: FuturesUnordered::new(),
            pending_events: VecDeque::new(),
            pending_error: None,
            inbound_request_id,
        }
    }
}

/// The events emitted by the [`RequestResponseHandler`].
#[doc(hidden)]
pub enum RequestResponseHandlerEvent {
    /// A request has been received.
    Request {
        request_id: P2pRpcId,
        request: P2pRpcRequest,
        sender: oneshot::Sender<P2pRpcResponse>,
    },
    /// A response has been received.
    Response {
        request_id: P2pRpcId,
        response: Option<P2pRpcResponse>,
    },
    /// A response to an inbound request has been sent.
    ResponseSent(P2pRpcId),
    /// A response to an inbound request was omitted as a result
    /// of dropping the response `sender` of an inbound `Request`.
    ResponseOmission(P2pRpcId),
    /// An outbound request failed to negotiate a mutually supported protocol.
    OutboundUnsupportedProtocols(P2pRpcId),
    /// An inbound request failed to negotiate a mutually supported protocol.
    InboundUnsupportedProtocols(P2pRpcId),
}

impl fmt::Debug for RequestResponseHandlerEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestResponseHandlerEvent::Request {
                request_id,
                request: _,
                sender: _,
            } => f
                .debug_struct("RequestResponseHandlerEvent::Request")
                .field("request_id", request_id)
                .finish(),
            RequestResponseHandlerEvent::Response {
                request_id,
                response: _,
            } => f
                .debug_struct("RequestResponseHandlerEvent::Response")
                .field("request_id", request_id)
                .finish(),
            RequestResponseHandlerEvent::ResponseSent(request_id) => f
                .debug_tuple("RequestResponseHandlerEvent::ResponseSent")
                .field(request_id)
                .finish(),
            RequestResponseHandlerEvent::ResponseOmission(request_id) => f
                .debug_tuple("RequestResponseHandlerEvent::ResponseOmission")
                .field(request_id)
                .finish(),
            RequestResponseHandlerEvent::OutboundUnsupportedProtocols(request_id) => f
                .debug_tuple("RequestResponseHandlerEvent::OutboundUnsupportedProtocols")
                .field(request_id)
                .finish(),
            RequestResponseHandlerEvent::InboundUnsupportedProtocols(request_id) => f
                .debug_tuple("RequestResponseHandlerEvent::InboundUnsupportedProtocols")
                .field(request_id)
                .finish(),
        }
    }
}

impl ConnectionHandler for RequestResponseHandler {
    type InEvent = RequestProtocol;
    type OutEvent = RequestResponseHandlerEvent;
    type Error = ConnectionHandlerUpgrErr<io::Error>;
    type InboundProtocol = ResponseProtocol;
    type OutboundProtocol = RequestProtocol;
    type OutboundOpenInfo = P2pRpcId;
    type InboundOpenInfo = P2pRpcId;

    fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol, Self::InboundOpenInfo> {
        // A channel for notifying the handler when the inbound
        // upgrade received the request.
        let (rq_send, rq_recv) = oneshot::channel();

        // A channel for notifying the inbound upgrade when the
        // response is sent.
        let (rs_send, rs_recv) = oneshot::channel();

        let request_id = self.inbound_request_id.fetch_add(1, Ordering::Relaxed);

        // By keeping all I/O inside the `ResponseProtocol` and thus the
        // inbound substream upgrade via above channels, we ensure that it
        // is all subject to the configured timeout without extra bookkeeping
        // for inbound substreams as well as their timeouts and also make the
        // implementation of inbound and outbound upgrades symmetric in
        // this sense.
        let proto = ResponseProtocol {
            request_sender: rq_send,
            response_receiver: rs_recv,
            request_id,
        };

        // The handler waits for the request to come in. It then emits
        // `RequestResponseHandlerEvent::Request` together with a
        // `ResponseChannel`.
        self.inbound
            .push(rq_recv.map_ok(move |rq| (rq, rs_send)).boxed());

        SubstreamProtocol::new(proto, request_id).with_timeout(Duration::from_secs(15))
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
            ConnectionEvent::FullyNegotiatedInbound(e) => {
                let sent = e.protocol;
                let request_id = e.info;
                if sent {
                    self.pending_events
                        .push_back(RequestResponseHandlerEvent::ResponseSent(request_id))
                } else {
                    self.pending_events
                        .push_back(RequestResponseHandlerEvent::ResponseOmission(request_id))
                }
            }
            ConnectionEvent::FullyNegotiatedOutbound(e) => {
                let response = e.protocol;
                let request_id = e.info;
                self.pending_events
                    .push_back(RequestResponseHandlerEvent::Response {
                        request_id,
                        response,
                    });
            }
            ConnectionEvent::ListenUpgradeError(e) => {
                let info = e.info;
                let error = e.error;
                match error {
                    ConnectionHandlerUpgrErr::Upgrade(UpgradeError::Select(
                        NegotiationError::Failed,
                    )) => {
                        // The local peer merely doesn't support the protocol(s) requested.
                        // This is no reason to close the connection, which may
                        // successfully communicate with other protocols already.
                        // An event is reported to permit user code to react to the fact that
                        // the local peer does not support the requested protocol(s).
                        self.pending_events.push_back(
                            RequestResponseHandlerEvent::InboundUnsupportedProtocols(info),
                        );
                    }
                    _ => {
                        // Anything else is considered a fatal error or misbehaviour of
                        // the remote peer and results in closing the connection.
                        self.pending_error = Some(error);
                    }
                }
            }
            ConnectionEvent::DialUpgradeError(e) => {
                let info = e.info;
                let error = e.error;
                match error {
                    ConnectionHandlerUpgrErr::Upgrade(UpgradeError::Select(
                        NegotiationError::Failed,
                    )) => {
                        // The remote merely doesn't support the protocol(s) we requested.
                        // This is no reason to close the connection, which may
                        // successfully communicate with other protocols already.
                        // An event is reported to permit user code to react to the fact that
                        // the remote peer does not support the requested protocol(s).
                        self.pending_events.push_back(
                            RequestResponseHandlerEvent::OutboundUnsupportedProtocols(info),
                        );
                    }
                    _ => {
                        // Anything else is considered a fatal error or misbehaviour of
                        // the remote peer and results in closing the connection.
                        self.pending_error = Some(error);
                    }
                }
            }
            _ => {}
        }
    }

    fn on_behaviour_event(&mut self, request: Self::InEvent) {
        self.outbound.push_back(request);
    }

    fn connection_keep_alive(&self) -> KeepAlive {
        KeepAlive::Yes
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<ConnectionHandlerEvent<RequestProtocol, P2pRpcId, Self::OutEvent, Self::Error>> {
        // Check for a pending (fatal) error.
        if let Some(err) = self.pending_error.take() {
            // The handler will not be polled again by the `Swarm`.
            return Poll::Ready(ConnectionHandlerEvent::Close(err));
        }

        // Drain pending events.
        if let Some(event) = self.pending_events.pop_front() {
            return Poll::Ready(ConnectionHandlerEvent::Custom(event));
        } else if self.pending_events.capacity() > EMPTY_QUEUE_SHRINK_THRESHOLD {
            self.pending_events.shrink_to_fit();
        }

        // Check for inbound requests.
        while let Poll::Ready(Some(result)) = self.inbound.poll_next_unpin(cx) {
            match result {
                Ok(((id, rq), rs_sender)) => {
                    // We received an inbound request.
                    return Poll::Ready(ConnectionHandlerEvent::Custom(
                        RequestResponseHandlerEvent::Request {
                            request_id: id,
                            request: rq,
                            sender: rs_sender,
                        },
                    ));
                }
                Err(oneshot::Canceled) => {
                    // The inbound upgrade has errored or timed out reading
                    // or waiting for the request. The handler is informed
                    // via `inject_listen_upgrade_error`.
                }
            }
        }

        // Emit outbound requests.
        if let Some(request) = self.outbound.pop_front() {
            let info = request.request_id;
            return Poll::Ready(ConnectionHandlerEvent::OutboundSubstreamRequest {
                protocol: SubstreamProtocol::new(request, info)
                    .with_timeout(Duration::from_secs(15)),
            });
        }

        debug_assert!(self.outbound.is_empty());

        if self.outbound.capacity() > EMPTY_QUEUE_SHRINK_THRESHOLD {
            self.outbound.shrink_to_fit();
        }

        Poll::Pending
    }
}
