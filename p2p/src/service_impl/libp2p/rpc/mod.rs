pub mod handler;
pub(super) mod protocol;

use handler::{RequestProtocol, RequestResponseHandler, RequestResponseHandlerEvent};
use lib::p2p::rpc::{P2pRpcEvent, P2pRpcId, P2pRpcOutgoingError, P2pRpcRequest};
use libp2p::core::{ConnectedPoint, Multiaddr, PeerId};
use libp2p::futures::channel::oneshot;
use libp2p::swarm::{
    ConnectionId,
    DialError, FromSwarm, IntoConnectionHandler, NetworkBehaviour, NetworkBehaviourAction,
    NotifyHandler, PollParameters,
};
use smallvec::SmallVec;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt,
    sync::{atomic::AtomicU64, Arc},
    task::{Context, Poll},
};

pub const RPC_PROTOCOL_NAME: &'static str = "coda/rpcs/0.0.1";

/// An inbound request or response.
#[derive(Debug)]
pub enum RequestResponseMessage<TRequest, TResponse, TChannelResponse = TResponse> {
    /// A request message.
    Request {
        /// The ID of this request.
        request_id: P2pRpcId,
        /// The request message.
        request: TRequest,
        /// The channel waiting for the response.
        ///
        /// If this channel is dropped instead of being used to send a response
        /// via [`RpcBehaviour::send_response`], a [`RequestResponseEvent::InboundFailure`]
        /// with [`InboundFailure::ResponseOmission`] is emitted.
        channel: ResponseChannel<TChannelResponse>,
    },
    /// A response message.
    Response {
        /// The ID of the request that produced this response.
        ///
        /// See [`RpcBehaviour::send_request`].
        request_id: P2pRpcId,
        /// The response message.
        response: TResponse,
    },
}

/// Possible failures occurring in the context of sending
/// an outbound request and receiving the response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutboundFailure {
    /// The connection closed before a response was received.
    ///
    /// It is not known whether the request may have been
    /// received (and processed) by the remote peer.
    ConnectionClosed,
    /// The remote supports none of the requested protocols.
    UnsupportedProtocols,
}

impl fmt::Display for OutboundFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutboundFailure::ConnectionClosed => {
                write!(f, "Connection was closed before a response was received")
            }
            OutboundFailure::UnsupportedProtocols => {
                write!(f, "The remote supports none of the requested protocols")
            }
        }
    }
}

impl std::error::Error for OutboundFailure {}

/// Possible failures occurring in the context of receiving an
/// inbound request and sending a response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InboundFailure {
    /// The connection closed before a response could be send.
    ConnectionClosed,
    /// The local peer supports none of the protocols requested
    /// by the remote.
    UnsupportedProtocols,
    /// The local peer failed to respond to an inbound request
    /// due to the [`ResponseChannel`] being dropped instead of
    /// being passed to [`RpcBehaviour::send_response`].
    ResponseOmission,
}

impl fmt::Display for InboundFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InboundFailure::ConnectionClosed => {
                write!(f, "Connection was closed before a response could be sent")
            }
            InboundFailure::UnsupportedProtocols => write!(
                f,
                "The local peer supports none of the protocols requested by the remote"
            ),
            InboundFailure::ResponseOmission => write!(
                f,
                "The response channel was dropped without sending a response to the remote"
            ),
        }
    }
}

impl std::error::Error for InboundFailure {}

/// A channel for sending a response to an inbound request.
///
/// See [`RpcBehaviour::send_response`].
#[derive(Debug)]
pub struct ResponseChannel<TResponse> {
    sender: oneshot::Sender<TResponse>,
}

impl<TResponse> ResponseChannel<TResponse> {
    /// Checks whether the response channel is still open, i.e.
    /// the `RpcBehaviour` behaviour is still waiting for a
    /// a response to be sent via [`RpcBehaviour::send_response`]
    /// and this response channel.
    ///
    /// If the response channel is no longer open then the inbound
    /// request timed out waiting for the response.
    pub fn is_open(&self) -> bool {
        !self.sender.is_canceled()
    }
}

pub struct RpcBehaviour {
    /// The next (inbound) request ID.
    next_inbound_id: Arc<AtomicU64>,
    /// Pending events to return from `poll`.
    pending_events: VecDeque<NetworkBehaviourAction<P2pRpcEvent, RequestResponseHandler>>,
    /// The currently connected peers, their pending outbound and inbound responses and their known,
    /// reachable addresses, if any.
    connected: HashMap<PeerId, SmallVec<[Connection; 2]>>,
}

impl RpcBehaviour {
    pub fn new() -> Self {
        Self {
            next_inbound_id: Arc::new(AtomicU64::new(1)),
            pending_events: VecDeque::new(),
            connected: HashMap::new(),
        }
    }

    pub fn send_request(&mut self, peer: PeerId, id: P2pRpcId, request: P2pRpcRequest) {
        let request = RequestProtocol {
            request_id: id,
            request,
        };
        self.try_send_request(peer, request);
    }

    /// Checks whether a peer is currently connected.
    pub fn is_connected(&self, peer: &PeerId) -> bool {
        if let Some(connections) = self.connected.get(peer) {
            !connections.is_empty()
        } else {
            false
        }
    }

    /// Checks whether an outbound request to the peer with the provided
    /// [`PeerId`] initiated by [`RpcBehaviour::send_request`] is still
    /// pending, i.e. waiting for a response.
    pub fn is_pending_outbound(&self, peer: &PeerId, request_id: &P2pRpcId) -> bool {
        // Check if request is already sent on established connection.
        let est_conn = self
            .connected
            .get(peer)
            .map(|cs| {
                cs.iter()
                    .any(|c| c.pending_inbound_responses.contains(request_id))
            })
            .unwrap_or(false);

        est_conn
    }

    /// Checks whether an inbound request from the peer with the provided
    /// [`PeerId`] is still pending, i.e. waiting for a response by the local
    /// node through [`RpcBehaviour::send_response`].
    pub fn is_pending_inbound(&self, peer: &PeerId, request_id: &P2pRpcId) -> bool {
        self.connected
            .get(peer)
            .map(|cs| {
                cs.iter()
                    .any(|c| c.pending_outbound_responses.contains(request_id))
            })
            .unwrap_or(false)
    }

    fn try_send_request(
        &mut self,
        peer: PeerId,
        request: RequestProtocol,
    ) -> Option<RequestProtocol> {
        if let Some(connections) = self.connected.get_mut(&peer) {
            if connections.is_empty() {
                return Some(request);
            }
            let ix = (request.request_id.locator() as usize) % connections.len();
            let conn = &mut connections[ix];
            conn.pending_inbound_responses.insert(request.request_id);
            self.pending_events
                .push_back(NetworkBehaviourAction::NotifyHandler {
                    peer_id: peer,
                    handler: NotifyHandler::One(conn.id),
                    event: request,
                });
            None
        } else {
            Some(request)
        }
    }

    /// Remove pending outbound response for the given peer and connection.
    ///
    /// Returns `true` if the provided connection to the given peer is still
    /// alive and the [`P2pRpcId`] was previously present and is now removed.
    /// Returns `false` otherwise.
    fn remove_pending_outbound_response(
        &mut self,
        peer: &PeerId,
        connection: ConnectionId,
        request: P2pRpcId,
    ) -> bool {
        self.get_connection_mut(peer, connection)
            .map(|c| c.pending_outbound_responses.remove(&request))
            .unwrap_or(false)
    }

    /// Remove pending inbound response for the given peer and connection.
    ///
    /// Returns `true` if the provided connection to the given peer is still
    /// alive and the [`P2pRpcId`] was previously present and is now removed.
    /// Returns `false` otherwise.
    fn remove_pending_inbound_response(
        &mut self,
        peer: &PeerId,
        connection: ConnectionId,
        request: &P2pRpcId,
    ) -> bool {
        self.get_connection_mut(peer, connection)
            .map(|c| c.pending_inbound_responses.remove(request))
            .unwrap_or(false)
    }

    /// Returns a mutable reference to the connection in `self.connected`
    /// corresponding to the given [`PeerId`] and [`ConnectionId`].
    fn get_connection_mut(
        &mut self,
        peer: &PeerId,
        connection: ConnectionId,
    ) -> Option<&mut Connection> {
        self.connected
            .get_mut(peer)
            .and_then(|connections| connections.iter_mut().find(|c| c.id == connection))
    }
}

impl NetworkBehaviour for RpcBehaviour {
    type ConnectionHandler = RequestResponseHandler;
    type OutEvent = P2pRpcEvent;

    fn new_handler(&mut self) -> Self::ConnectionHandler {
        RequestResponseHandler::new(self.next_inbound_id.clone())
    }

    fn addresses_of_peer(&mut self, _: &PeerId) -> Vec<Multiaddr> {
        vec![]
    }

    fn handle_established_inbound_connection(
        &mut self,
        peer: &PeerId,
        conn: &ConnectionId,
        endpoint: &ConnectedPoint,
        _errors: Option<&Vec<Multiaddr>>,
        other_established: usize,
    ) {
        self.connected
            .entry(*peer)
            .or_default()
            .push(Connection::new(*conn));
    }

    fn handle_established_outbound_connection(
        &mut self,
        peer: &PeerId,
        conn: &ConnectionId,
        endpoint: &ConnectedPoint,
        _errors: Option<&Vec<Multiaddr>>,
        other_established: usize,
    ) {
        self.connected
            .entry(*peer)
            .or_default()
            .push(Connection::new(*conn));
    }

    fn on_swarm_event(&mut self, event: FromSwarm<Self::ConnectionHandler>) {
        match event {
            FromSwarm::ConnectionClosed(event) => {
                let peer_id = &event.peer_id;
                let conn = &event.connection_id;
                let connections = self
                    .connected
                    .get_mut(peer_id)
                    .expect("Expected some established connection to peer before closing.");

                let connection = connections
                    .iter()
                    .position(|c| &c.id == conn)
                    .map(|p: usize| connections.remove(p))
                    .expect("Expected connection to be established before closing.");

                if connections.is_empty() {
                    self.connected.remove(peer_id);
                }

                for request_id in connection.pending_outbound_responses {
                    // TODO(binier): incoming rpcs
                    // self.pending_events
                    //     .push_back(NetworkBehaviourAction::GenerateEvent(
                    //         RequestResponseEvent::InboundFailure {
                    //             peer: *peer_id,
                    //             request_id,
                    //             error: InboundFailure::ConnectionClosed,
                    //         },
                    //     ));
                }

                for request_id in connection.pending_inbound_responses {
                    self.pending_events
                        .push_back(NetworkBehaviourAction::GenerateEvent(
                            P2pRpcEvent::OutgoingError(
                                *peer_id,
                                request_id,
                                P2pRpcOutgoingError::ConnectionClosed,
                            ),
                        ));
                }
            }
            _ => {}
        }
    }

    fn on_connection_handler_event(
        &mut self,
        peer: PeerId,
        connection: ConnectionId,
        event: RequestResponseHandlerEvent,
    ) {
        match event {
            RequestResponseHandlerEvent::Response {
                request_id,
                response,
            } => {
                let removed = self.remove_pending_inbound_response(&peer, connection, &request_id);
                debug_assert!(
                    removed,
                    "Expect request_id to be pending before receiving response.",
                );

                self.pending_events
                    .push_back(NetworkBehaviourAction::GenerateEvent(
                        P2pRpcEvent::OutgoingResponse(peer, request_id, response),
                    ));
            }
            RequestResponseHandlerEvent::Request {
                request_id,
                request,
                sender,
            } => {
                // TODO(binier): incoming rpcs
                // let channel = ResponseChannel { sender };
                // let message = RequestResponseMessage::Request {
                //     request_id,
                //     request,
                //     channel,
                // };
                // self.pending_events
                //     .push_back(NetworkBehaviourAction::GenerateEvent(
                //         RequestResponseEvent::Message { peer, message },
                //     ));

                // match self.get_connection_mut(&peer, connection) {
                //     Some(connection) => {
                //         let inserted = connection.pending_outbound_responses.insert(request_id);
                //         debug_assert!(inserted, "Expect id of new request to be unknown.");
                //     }
                //     // Connection closed after `RequestResponseEvent::Request` has been emitted.
                //     None => {
                //         self.pending_events
                //             .push_back(NetworkBehaviourAction::GenerateEvent(
                //                 RequestResponseEvent::InboundFailure {
                //                     peer,
                //                     request_id,
                //                     error: InboundFailure::ConnectionClosed,
                //                 },
                //             ));
                //     }
                // }
            }
            RequestResponseHandlerEvent::ResponseSent(request_id) => {
                // TODO(binier): incoming rpcs
                // let removed = self.remove_pending_outbound_response(&peer, connection, request_id);
                // debug_assert!(
                //     removed,
                //     "Expect request_id to be pending before response is sent."
                // );

                // self.pending_events
                //     .push_back(NetworkBehaviourAction::GenerateEvent(
                //         RequestResponseEvent::ResponseSent { peer, request_id },
                //     ));
            }
            RequestResponseHandlerEvent::ResponseOmission(request_id) => {
                // TODO(binier): incoming rpcs
                // let removed = self.remove_pending_outbound_response(&peer, connection, request_id);
                // debug_assert!(
                //     removed,
                //     "Expect request_id to be pending before response is omitted.",
                // );

                // self.pending_events
                //     .push_back(NetworkBehaviourAction::GenerateEvent(
                //         RequestResponseEvent::InboundFailure {
                //             peer,
                //             request_id,
                //             error: InboundFailure::ResponseOmission,
                //         },
                //     ));
            }
            RequestResponseHandlerEvent::OutboundUnsupportedProtocols(request_id) => {
                let removed = self.remove_pending_inbound_response(&peer, connection, &request_id);
                debug_assert!(
                    removed,
                    "Expect request_id to be pending before failing to connect.",
                );

                self.pending_events
                    .push_back(NetworkBehaviourAction::GenerateEvent(
                        P2pRpcEvent::OutgoingError(
                            peer,
                            request_id,
                            P2pRpcOutgoingError::ProtocolUnsupported,
                        ),
                    ));
            }
            RequestResponseHandlerEvent::InboundUnsupportedProtocols(request_id) => {
                // TODO(binier): incoming rpcs
                // Note: No need to call `self.remove_pending_outbound_response`,
                // `RequestResponseHandlerEvent::Request` was never emitted for this request and
                // thus request was never added to `pending_outbound_responses`.
                // self.pending_events
                //     .push_back(NetworkBehaviourAction::GenerateEvent(
                //         P2pRpcEvent::OutgoingError(
                //             *peer_id,
                //             request_id,
                //             P2pRpcOutgoingError::UnsupportedProtocol,
                //         ),
                //     ));
            }
        }
    }

    fn poll(
        &mut self,
        _: &mut Context<'_>,
        _: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<Self::OutEvent, Self::ConnectionHandler>> {
        if let Some(ev) = self.pending_events.pop_front() {
            return Poll::Ready(ev);
        } else if self.pending_events.capacity() > EMPTY_QUEUE_SHRINK_THRESHOLD {
            self.pending_events.shrink_to_fit();
        }

        Poll::Pending
    }
}

/// Internal threshold for when to shrink the capacity
/// of empty queues. If the capacity of an empty queue
/// exceeds this threshold, the associated memory is
/// released.
const EMPTY_QUEUE_SHRINK_THRESHOLD: usize = 100;

/// Internal information tracked for an established connection.
struct Connection {
    id: ConnectionId,
    /// Pending outbound responses where corresponding inbound requests have
    /// been received on this connection and emitted via `poll` but have not yet
    /// been answered.
    pending_outbound_responses: HashSet<P2pRpcId>,
    /// Pending inbound responses for previously sent requests on this
    /// connection.
    pending_inbound_responses: HashSet<P2pRpcId>,
}

impl Connection {
    fn new(id: ConnectionId) -> Self {
        Self {
            id,
            pending_outbound_responses: Default::default(),
            pending_inbound_responses: Default::default(),
        }
    }
}
