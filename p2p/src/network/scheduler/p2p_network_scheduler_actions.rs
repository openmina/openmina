use std::net::{IpAddr, SocketAddr};

use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use super::{
    super::{
        select::{token, SelectKind},
        Data, Limit,
    },
    p2p_network_scheduler_state::{P2pNetworkConnectionCloseReason, P2pNetworkConnectionError},
};

use crate::{
    disconnection::P2pDisconnectionReason, ConnectionAddr, P2pPeerStatus, P2pState, PeerId,
    StreamId,
};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(ip), display(listener), display(addr), debug(result), select_kind = debug(kind), display(error)))]
pub enum P2pNetworkSchedulerAction {
    InterfaceDetected {
        ip: IpAddr,
    },
    InterfaceExpired {
        ip: IpAddr,
    },
    ListenerReady {
        listener: SocketAddr,
    },
    ListenerError {
        listener: SocketAddr,
        error: String,
    },
    #[action_event(fields(debug(addr), debug(result)))]
    IncomingDidAccept {
        addr: Option<ConnectionAddr>,
        result: Result<(), String>,
    },
    IncomingDataIsReady {
        addr: ConnectionAddr,
    },
    /// Initialize outgoing connection.
    OutgoingConnect {
        addr: SocketAddr,
    },
    /// Outgoint TCP stream is established.
    OutgoingDidConnect {
        addr: ConnectionAddr,
        result: Result<(), String>,
    },
    IncomingDataDidReceive {
        addr: ConnectionAddr,
        result: Result<Data, String>,
    },
    SelectDone {
        addr: ConnectionAddr,
        kind: SelectKind,
        protocol: Option<token::Protocol>,
        incoming: bool,
        expected_peer_id: Option<PeerId>,
    },
    SelectError {
        addr: ConnectionAddr,
        kind: SelectKind,
        error: String,
    },
    YamuxDidInit {
        addr: ConnectionAddr,
        peer_id: PeerId,
        message_size_limit: Limit<usize>,
    },

    /// Action that initiate the specified peer disconnection.
    Disconnect {
        /// Connection address.
        addr: ConnectionAddr,
        /// Reason why disconneciton is triggered.
        reason: P2pDisconnectionReason,
    },

    /// Fatal connection error.
    Error {
        /// Connection address.
        addr: ConnectionAddr,
        /// Reason why disconneciton is triggered.
        error: P2pNetworkConnectionError,
    },

    /// Remote address is disconnected.
    ///
    /// Action that signals that the peer is disconnected.
    Disconnected {
        /// Connection address.
        addr: ConnectionAddr,
        /// Reason why the peer disconnected.
        reason: P2pNetworkConnectionCloseReason,
    },

    /// Prune connection.
    Prune {
        /// Connection address.
        addr: ConnectionAddr,
    },
    /// Prune streams.
    PruneStreams {
        peer_id: PeerId,
    },
    /// Prune streams.
    PruneStream {
        peer_id: PeerId,
        stream_id: StreamId,
    },
}

impl From<P2pNetworkSchedulerAction> for crate::P2pAction {
    fn from(value: P2pNetworkSchedulerAction) -> Self {
        crate::P2pAction::Network(value.into())
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkSchedulerAction {
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
        match self {
            P2pNetworkSchedulerAction::InterfaceDetected { .. }
            | P2pNetworkSchedulerAction::InterfaceExpired { .. }
            | P2pNetworkSchedulerAction::ListenerReady { .. }
            | P2pNetworkSchedulerAction::ListenerError { .. }
            | P2pNetworkSchedulerAction::SelectDone { .. }
            | P2pNetworkSchedulerAction::SelectError { .. } => true,
            P2pNetworkSchedulerAction::IncomingDidAccept { addr, .. } => {
                addr.as_ref().map_or(false, |addr| {
                    !state.network.scheduler.connections.contains_key(addr)
                })
            }
            P2pNetworkSchedulerAction::OutgoingConnect { addr } => !state
                .network
                .scheduler
                .connections
                .contains_key(&ConnectionAddr {
                    sock_addr: *addr,
                    incoming: false,
                }),
            P2pNetworkSchedulerAction::OutgoingDidConnect { addr, .. } => state
                .network
                .scheduler
                .connections
                .get(addr)
                .map_or(false, |conn_state| !conn_state.incoming),
            P2pNetworkSchedulerAction::IncomingDataDidReceive { addr, .. }
            | P2pNetworkSchedulerAction::IncomingDataIsReady { addr }
            | P2pNetworkSchedulerAction::YamuxDidInit { addr, .. } => {
                state.network.scheduler.connections.contains_key(addr)
            }
            P2pNetworkSchedulerAction::Disconnect { addr, .. }
            | P2pNetworkSchedulerAction::Error { addr, .. } => state
                .network
                .scheduler
                .connections
                .get(addr)
                .map_or(false, |conn_state| conn_state.closed.is_none()),
            P2pNetworkSchedulerAction::Disconnected { addr, reason } => state
                .network
                .scheduler
                .connections
                .get(addr)
                .map_or(false, |conn_state| {
                    conn_state.closed.as_ref() == Some(reason)
                }),
            // TODO: introduce state for closed connection
            P2pNetworkSchedulerAction::Prune { addr } => state
                .network
                .scheduler
                .connections
                .get(addr)
                .map_or(false, |conn_state| conn_state.closed.is_some()),
            P2pNetworkSchedulerAction::PruneStreams { peer_id } => {
                state.peers.get(peer_id).map_or(false, |peer_state| {
                    peer_state.status.is_error()
                        || matches!(peer_state.status, P2pPeerStatus::Disconnected { .. })
                })
            }
            P2pNetworkSchedulerAction::PruneStream { peer_id, stream_id } => state
                .network
                .scheduler
                .find_peer(peer_id)
                .and_then(|(_, conn_state)| conn_state.streams.get(stream_id))
                .is_some(),
        }
    }
}
