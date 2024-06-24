use std::net::SocketAddr;

use mina_p2p_messages::rpc_kernel::{QueryHeader, QueryID, ResponseHeader};
use openmina_core::{action_debug, action_trace, ActionEvent};
use serde::{Deserialize, Serialize};

use super::{super::*, *};
use crate::{P2pState, PeerId};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(addr), display(peer_id), incoming, stream_id, debug(data), fin))]
pub enum P2pNetworkRpcAction {
    Init {
        addr: SocketAddr,
        peer_id: PeerId,
        stream_id: StreamId,
        incoming: bool,
    },
    #[action_event(level = trace)]
    IncomingData {
        addr: SocketAddr,
        peer_id: PeerId,
        stream_id: StreamId,
        data: Data,
    },
    #[action_event(expr(log_message(context, message, addr, peer_id, stream_id)))]
    IncomingMessage {
        addr: SocketAddr,
        peer_id: PeerId,
        stream_id: StreamId,
        message: RpcMessage,
    },
    PrunePending {
        peer_id: PeerId,
        stream_id: StreamId,
    },
    HeartbeatSend {
        addr: SocketAddr,
        peer_id: PeerId,
        stream_id: StreamId,
    },
    OutgoingQuery {
        peer_id: PeerId,
        query: QueryHeader,
        data: Data,
    },
    OutgoingResponse {
        peer_id: PeerId,
        response: ResponseHeader,
        data: Data,
    },
    OutgoingData {
        addr: SocketAddr,
        peer_id: PeerId,
        stream_id: StreamId,
        data: Data,
        fin: bool,
    },
}

pub enum RpcStreamId {
    Exact(StreamId),
    WithQuery(QueryID),
    AnyIncoming,
    AnyOutgoing,
}

impl P2pNetworkRpcAction {
    pub fn stream_id(&self) -> RpcStreamId {
        match self {
            Self::Init { stream_id, .. } => RpcStreamId::Exact(*stream_id),
            Self::IncomingData { stream_id, .. } => RpcStreamId::Exact(*stream_id),
            Self::IncomingMessage { stream_id, .. } => RpcStreamId::Exact(*stream_id),
            Self::PrunePending { stream_id, .. } => RpcStreamId::Exact(*stream_id),
            Self::HeartbeatSend { stream_id, .. } => RpcStreamId::Exact(*stream_id),
            Self::OutgoingQuery { .. } => RpcStreamId::AnyOutgoing,
            Self::OutgoingResponse {
                response: ResponseHeader { id },
                ..
            } => RpcStreamId::WithQuery(*id),
            Self::OutgoingData { stream_id, .. } => RpcStreamId::Exact(*stream_id),
        }
    }

    pub fn peer_id(&self) -> &PeerId {
        match self {
            Self::Init { peer_id, .. } => peer_id,
            Self::IncomingData { peer_id, .. } => peer_id,
            Self::IncomingMessage { peer_id, .. } => peer_id,
            Self::PrunePending { peer_id, .. } => peer_id,
            Self::HeartbeatSend { peer_id, .. } => peer_id,
            Self::OutgoingQuery { peer_id, .. } => peer_id,
            Self::OutgoingResponse { peer_id, .. } => peer_id,
            Self::OutgoingData { peer_id, .. } => peer_id,
        }
    }
}
impl From<P2pNetworkRpcAction> for crate::P2pAction {
    fn from(a: P2pNetworkRpcAction) -> Self {
        Self::Network(a.into())
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkRpcAction {
    fn is_enabled(&self, state: &P2pState, time: redux::Timestamp) -> bool {
        #[allow(unused_variables)]
        match self {
            P2pNetworkRpcAction::Init {
                addr,
                peer_id,
                stream_id,
                incoming,
            } => true,
            P2pNetworkRpcAction::IncomingData {
                addr,
                peer_id,
                stream_id,
                data,
            } => true,
            P2pNetworkRpcAction::IncomingMessage {
                addr,
                peer_id,
                stream_id,
                message,
            } => true,
            P2pNetworkRpcAction::PrunePending { peer_id, stream_id } => true,
            P2pNetworkRpcAction::HeartbeatSend {
                addr,
                peer_id,
                stream_id,
            } => {
                // TODO: if we have an incoming rpc, for which response
                // isn't yet fully flushed to the stream, we will end up
                // adding these heartbeats to the queue. Not necessarily
                // an issue but not a completely correct behavior either.
                state
                    .network
                    .find_rpc_state(self)
                    .map_or(false, |s| s.should_send_heartbeat(time))
            }
            P2pNetworkRpcAction::OutgoingQuery {
                peer_id,
                query,
                data,
            } => true,
            P2pNetworkRpcAction::OutgoingResponse {
                peer_id,
                response,
                data,
            } => true,
            P2pNetworkRpcAction::OutgoingData {
                addr,
                peer_id,
                stream_id,
                data,
                fin,
            } => true,
        }
    }
}

fn log_message<T>(
    context: &T,
    message: &RpcMessage,
    addr: &SocketAddr,
    peer_id: &PeerId,
    stream_id: &u32,
) where
    T: openmina_core::log::EventContext,
{
    match message {
        RpcMessage::Handshake => action_trace!(
            context,
            kind = "P2pNetworkRpcIncomingMessage",
            addr = display(addr),
            peer_id = display(peer_id),
            stream_id,
            message_kind = "handshake"
        ),
        RpcMessage::Heartbeat => action_trace!(
            context,
            kind = "P2pNetworkRpcIncomingMessage",
            addr = display(addr),
            peer_id = display(peer_id),
            stream_id,
            message_kind = "heartbeat"
        ),
        RpcMessage::Query { header, .. } => action_debug!(
            context,
            kind = "P2pNetworkRpcIncomingMessage",
            addr = display(addr),
            peer_id = display(peer_id),
            stream_id,
            message_kind = "query",
            message_header = debug(header)
        ),
        RpcMessage::Response { header, .. } => action_debug!(
            context,
            kind = "P2pNetworkRpcIncomingMessage",
            addr = display(addr),
            peer_id = display(peer_id),
            stream_id,
            message_kind = "response",
            message_header = debug(header)
        ),
    }
}
