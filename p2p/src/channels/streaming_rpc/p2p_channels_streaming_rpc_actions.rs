use openmina_core::ActionEvent;
use redux::Timestamp;
use serde::{Deserialize, Serialize};

use crate::{P2pState, PeerId};

use super::{
    P2pChannelsStreamingRpcState, P2pStreamingRpcId, P2pStreamingRpcLocalState,
    P2pStreamingRpcRemoteState, P2pStreamingRpcRequest, P2pStreamingRpcResponse,
    P2pStreamingRpcResponseFull,
};

pub type P2pChannelsStreamingRpcActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a P2pChannelsStreamingRpcAction>;

pub const MAX_P2P_RPC_REMOTE_CONCURRENT_REQUESTS: usize = 5;

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(peer_id)))]
pub enum P2pChannelsStreamingRpcAction {
    Init {
        peer_id: PeerId,
    },
    Pending {
        peer_id: PeerId,
    },
    Ready {
        peer_id: PeerId,
    },
    RequestSend {
        peer_id: PeerId,
        id: P2pStreamingRpcId,
        request: Box<P2pStreamingRpcRequest>,
    },
    Timeout {
        peer_id: PeerId,
        id: P2pStreamingRpcId,
    },
    ResponseNextPartGet {
        peer_id: PeerId,
        id: P2pStreamingRpcId,
    },
    ResponsePartReceived {
        peer_id: PeerId,
        id: P2pStreamingRpcId,
        response: P2pStreamingRpcResponse,
    },
    ResponseReceived {
        peer_id: PeerId,
        id: P2pStreamingRpcId,
        response: Option<P2pStreamingRpcResponseFull>,
    },
    RequestReceived {
        peer_id: PeerId,
        id: P2pStreamingRpcId,
        request: Box<P2pStreamingRpcRequest>,
    },
    /// Response for the request sent by peer is pending. Dispatched when
    /// we need data from an async component, like ledger, for constructing
    /// the response.
    ResponsePending {
        peer_id: PeerId,
        id: P2pStreamingRpcId,
    },
    ResponseSendInit {
        peer_id: PeerId,
        id: P2pStreamingRpcId,
        response: Option<P2pStreamingRpcResponseFull>,
    },
    ResponsePartNextSend {
        peer_id: PeerId,
        id: P2pStreamingRpcId,
    },
    ResponsePartSend {
        peer_id: PeerId,
        id: P2pStreamingRpcId,
        response: Box<P2pStreamingRpcResponse>,
    },
    ResponseSent {
        peer_id: PeerId,
        id: P2pStreamingRpcId,
    },
}

impl P2pChannelsStreamingRpcAction {
    pub fn peer_id(&self) -> &PeerId {
        match self {
            Self::Init { peer_id }
            | Self::Pending { peer_id }
            | Self::Ready { peer_id }
            | Self::RequestSend { peer_id, .. }
            | Self::Timeout { peer_id, .. }
            | Self::ResponseNextPartGet { peer_id, .. }
            | Self::ResponsePartReceived { peer_id, .. }
            | Self::ResponseReceived { peer_id, .. }
            | Self::RequestReceived { peer_id, .. }
            | Self::ResponsePending { peer_id, .. }
            | Self::ResponseSendInit { peer_id, .. }
            | Self::ResponsePartNextSend { peer_id, .. }
            | Self::ResponsePartSend { peer_id, .. }
            | Self::ResponseSent { peer_id, .. } => peer_id,
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pChannelsStreamingRpcAction {
    fn is_enabled(&self, state: &P2pState, time: Timestamp) -> bool {
        match self {
            P2pChannelsStreamingRpcAction::Init { peer_id } => {
                state.peers.get(peer_id).filter(|p| !p.is_libp2p())
                    .and_then(|p| p.status.as_ready())
                    .map_or(false, |p| matches!(p.channels.streaming_rpc, P2pChannelsStreamingRpcState::Enabled))
            },
            P2pChannelsStreamingRpcAction::Pending { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(p.channels.streaming_rpc, P2pChannelsStreamingRpcState::Init { .. })
                })
            },
            P2pChannelsStreamingRpcAction::Ready { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(p.channels.streaming_rpc, P2pChannelsStreamingRpcState::Pending { .. })
                })
            },
            P2pChannelsStreamingRpcAction::RequestSend { peer_id, id, .. } => {
                state.get_ready_peer(peer_id)
                    .map_or(false, |p| matches!(
                        &p.channels.streaming_rpc,
                        P2pChannelsStreamingRpcState::Ready { local: P2pStreamingRpcLocalState::WaitingForRequest { .. } | P2pStreamingRpcLocalState::Responded { .. }, .. } if p.channels.next_local_rpc_id() == *id
                    ))
            },
            P2pChannelsStreamingRpcAction::Timeout { peer_id, id } => {
                state.get_ready_peer(peer_id).map_or(false, |p|
                    matches!(&p.channels.streaming_rpc, P2pChannelsStreamingRpcState::Ready { local:
                        P2pStreamingRpcLocalState::Requested { id: rpc_id, .. }, .. } if rpc_id == id))
                    && state.is_peer_streaming_rpc_timed_out(peer_id, *id, time)
            },
            P2pChannelsStreamingRpcAction::ResponseNextPartGet { peer_id, id, .. } => {
                state.get_ready_peer(peer_id).map_or(false, |p| match &p.channels.streaming_rpc {
                    P2pChannelsStreamingRpcState::Ready { local: P2pStreamingRpcLocalState::Requested { id: rpc_id, progress, .. }, .. } => {
                        rpc_id == id && !progress.is_done() && !progress.is_part_pending()
                    },
                    _ => false,
                })
            },
            P2pChannelsStreamingRpcAction::ResponsePartReceived { peer_id, id, response } => {
                state.get_ready_peer(peer_id).map_or(false, |p| match &p.channels.streaming_rpc {
                    P2pChannelsStreamingRpcState::Ready { local: P2pStreamingRpcLocalState::Requested { id: rpc_id, request, .. }, .. } => {
                        rpc_id == id && response.kind() == request.kind()
                    },
                    _ => false,
                })
            },
            P2pChannelsStreamingRpcAction::ResponseReceived { peer_id, id, response } => {
                state.get_ready_peer(peer_id).map_or(false, |p| match &p.channels.streaming_rpc {
                    P2pChannelsStreamingRpcState::Ready { local: P2pStreamingRpcLocalState::Requested { id: rpc_id, request, progress, .. }, .. } => {
                        rpc_id == id && (response.is_none() || progress.is_done()) && response.as_ref().map_or(true, |resp| resp.kind() == request.kind())
                    },
                    _ => false,
                })
            },
            P2pChannelsStreamingRpcAction::RequestReceived { peer_id, .. } => {
                state.get_ready_peer(peer_id).map_or(false, |p| match &p.channels.streaming_rpc {
                    P2pChannelsStreamingRpcState::Ready { remote, .. } => {
                        matches!(remote, P2pStreamingRpcRemoteState::WaitingForRequest { .. } | P2pStreamingRpcRemoteState::Responded { ..})
                    },
                    _ => false,
                })
            },
            P2pChannelsStreamingRpcAction::ResponsePending { peer_id, id } => {
                state.get_ready_peer(peer_id)
                    .and_then(|p| p.channels.streaming_rpc.remote_todo_request())
                    .map_or(false, |(rpc_id, _)| rpc_id == *id)
            },
            P2pChannelsStreamingRpcAction::ResponseSendInit { peer_id, id, response } => {
                state.get_ready_peer(peer_id)
                    .and_then(|p| p.channels.streaming_rpc.remote_pending_request())
                    .map_or(false, |(rpc_id, req)| rpc_id == *id && response.as_ref().map_or(true, |resp| resp.kind() == req.kind()))
            },
            P2pChannelsStreamingRpcAction::ResponsePartNextSend { peer_id, id } => {
                state.get_ready_peer(peer_id)
                    .map_or(false, |p| match &p.channels.streaming_rpc {
                    P2pChannelsStreamingRpcState::Ready { remote: P2pStreamingRpcRemoteState::Requested { id: rpc_id, progress, .. }, .. } => {
                        rpc_id == id && !progress.is_done()
                    }
                    _ => false,
                })
            }
            P2pChannelsStreamingRpcAction::ResponsePartSend { peer_id, id, response } => {
                state.get_ready_peer(peer_id)
                    .map_or(false, |p| match &p.channels.streaming_rpc {
                    P2pChannelsStreamingRpcState::Ready { remote: P2pStreamingRpcRemoteState::Requested { id: rpc_id, request, progress, .. }, .. } => {
                        rpc_id == id && !progress.is_done() && response.kind() == request.kind()
                    }
                    _ => false,
                })
            },
            P2pChannelsStreamingRpcAction::ResponseSent { peer_id, id} => {
                state.get_ready_peer(peer_id)
                    .map_or(false, |p| match &p.channels.streaming_rpc {
                    P2pChannelsStreamingRpcState::Ready { remote: P2pStreamingRpcRemoteState::Requested { id: rpc_id, progress, .. }, .. } => {
                        rpc_id == id && progress.is_done()
                    }
                    _ => false,
                })
            },
        }
    }
}

use crate::channels::P2pChannelsAction;

impl From<P2pChannelsStreamingRpcAction> for crate::P2pAction {
    fn from(a: P2pChannelsStreamingRpcAction) -> Self {
        Self::Channels(P2pChannelsAction::StreamingRpc(a))
    }
}
