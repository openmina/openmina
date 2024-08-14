use openmina_core::ActionEvent;
use redux::Timestamp;
use serde::{Deserialize, Serialize};

use crate::{P2pState, PeerId};

use super::{P2pChannelsRpcState, P2pRpcId, P2pRpcLocalState, P2pRpcRequest, P2pRpcResponse};

pub type P2pChannelsRpcActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pChannelsRpcAction>;

pub const MAX_P2P_RPC_REMOTE_CONCURRENT_REQUESTS: usize = 5;

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(peer_id)))]
pub enum P2pChannelsRpcAction {
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
        id: P2pRpcId,
        request: Box<P2pRpcRequest>,
    },
    Timeout {
        peer_id: PeerId,
        id: P2pRpcId,
    },
    ResponseReceived {
        peer_id: PeerId,
        id: P2pRpcId,
        response: Option<Box<P2pRpcResponse>>,
    },
    RequestReceived {
        peer_id: PeerId,
        id: P2pRpcId,
        request: Box<P2pRpcRequest>,
    },
    /// Response for the request sent by peer is pending. Dispatched when
    /// we need data from an async component, like ledger, for constructing
    /// the response.
    ResponsePending {
        peer_id: PeerId,
        id: P2pRpcId,
    },
    ResponseSend {
        peer_id: PeerId,
        id: P2pRpcId,
        response: Option<Box<P2pRpcResponse>>,
    },
}

impl P2pChannelsRpcAction {
    pub fn peer_id(&self) -> &PeerId {
        match self {
            Self::Init { peer_id }
            | Self::Pending { peer_id }
            | Self::Ready { peer_id }
            | Self::RequestSend { peer_id, .. }
            | Self::Timeout { peer_id, .. }
            | Self::ResponseReceived { peer_id, .. }
            | Self::RequestReceived { peer_id, .. }
            | Self::ResponsePending { peer_id, .. }
            | Self::ResponseSend { peer_id, .. } => peer_id,
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pChannelsRpcAction {
    fn is_enabled(&self, state: &P2pState, time: Timestamp) -> bool {
        match self {
            P2pChannelsRpcAction::Init { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(p.channels.rpc, P2pChannelsRpcState::Enabled)
                })
            },
            P2pChannelsRpcAction::Pending { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(p.channels.rpc, P2pChannelsRpcState::Init { .. })
                })
            },
            P2pChannelsRpcAction::Ready { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(p.channels.rpc, P2pChannelsRpcState::Pending { .. })
                })
            },
            P2pChannelsRpcAction::RequestSend { peer_id, id, request } => {
                state.peers.get(peer_id)
                    .filter(|p| !p.is_libp2p() || request.kind().supported_by_libp2p())
                    .and_then(|p| p.status.as_ready())
                    .map_or(false, |p| matches!(
                        &p.channels.rpc,
                        P2pChannelsRpcState::Ready { local: P2pRpcLocalState::WaitingForRequest { .. } | P2pRpcLocalState::Responded { .. }, .. } if p.channels.next_local_rpc_id() == *id
                    ))
            },
            P2pChannelsRpcAction::Timeout { peer_id, id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| matches!(&p.channels.rpc, P2pChannelsRpcState::Ready { local: P2pRpcLocalState::Requested { id: rpc_id, .. }, .. } if rpc_id == id))
                    && state.is_peer_rpc_timed_out(peer_id, *id, time)
            },
            P2pChannelsRpcAction::ResponseReceived { peer_id, id, .. } => {
                // TODO(binier): use consensus to enforce that peer doesn't send
                // us inferior block than it has in the past.
                state.get_ready_peer(peer_id).map_or(false, |p| match &p.channels.rpc {
                    P2pChannelsRpcState::Ready { local, .. } => {
                        // TODO(binier): validate that response corresponds to request.
                        matches!(local, P2pRpcLocalState::Requested { id: rpc_id, .. } if rpc_id == id)
                    },
                    _ => false,
                })
            },
            P2pChannelsRpcAction::RequestReceived { peer_id, id, .. } => {
                state.get_ready_peer(peer_id).map_or(false, |p| match &p.channels.rpc {
                    P2pChannelsRpcState::Ready { remote, .. } => {
                        remote.pending_requests.len() < MAX_P2P_RPC_REMOTE_CONCURRENT_REQUESTS &&
                        remote.pending_requests.iter().all(|v| v.id != *id)
                    },
                    _ => false,
                })
            },
            P2pChannelsRpcAction::ResponsePending { peer_id, id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| match &p.channels.rpc {
                    P2pChannelsRpcState::Ready { remote, .. } => {
                        remote.pending_requests.iter().any(|v| v.id == *id && !v.is_pending)
                    },
                    _ => false,
                })
            },
            P2pChannelsRpcAction::ResponseSend { peer_id, id, response: _response } => {
                #[cfg(feature = "p2p-libp2p")]
                if state.is_libp2p_peer(peer_id) {
                    let Some(response) = _response.as_ref() else {
                        return false;
                    };
                    return if !response.kind().supported_by_libp2p() {
                        false
                    } else if let Some(streams) = state
                        .network
                        .scheduler
                        .rpc_incoming_streams
                        .get(peer_id)
                    {
                        !streams.is_empty()
                    } else {
                        false
                    };
                }

                state.get_ready_peer(peer_id).map_or(false, |p| match &p.channels.rpc {
                    P2pChannelsRpcState::Ready { remote, .. } => {
                        // TODO(binier): validate that response corresponds to request.
                        remote.pending_requests.iter().any(|v| v.id == *id)
                    },
                    _ => false,
                })
            },
        }
    }
}

use crate::channels::P2pChannelsAction;

impl From<P2pChannelsRpcAction> for crate::P2pAction {
    fn from(a: P2pChannelsRpcAction) -> Self {
        Self::Channels(P2pChannelsAction::Rpc(a))
    }
}
