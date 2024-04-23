use std::net::{IpAddr, SocketAddr};

use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use super::{
    super::{
        select::{token, SelectKind},
        Data,
    },
    p2p_network_scheduler_state::{P2pNetworkConnectionCloseReason, P2pNetworkConnectionError},
};

use crate::{disconnection::P2pDisconnectionReason, P2pState, PeerId};

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
    IncomingConnectionIsReady {
        listener: SocketAddr,
    },
    #[action_event(fields(debug(addr), debug(result)))]
    IncomingDidAccept {
        addr: Option<SocketAddr>,
        result: Result<(), String>,
    },
    /// Initialize outgoing connection.
    OutgoingConnect {
        addr: SocketAddr,
    },
    /// Outgoint TCP stream is established.
    OutgoingDidConnect {
        addr: SocketAddr,
        result: Result<(), String>,
    },
    IncomingDataIsReady {
        addr: SocketAddr,
    },
    IncomingDataDidReceive {
        addr: SocketAddr,
        result: Result<Data, String>,
    },
    SelectDone {
        addr: SocketAddr,
        kind: SelectKind,
        protocol: Option<token::Protocol>,
        incoming: bool,
    },
    SelectError {
        addr: SocketAddr,
        kind: SelectKind,
        error: String,
    },
    YamuxDidInit {
        addr: SocketAddr,
        peer_id: PeerId,
    },

    /// Action that initiate the specified peer disconnection.
    Disconnect {
        /// Connection address.
        addr: SocketAddr,
        /// Reason why disconneciton is triggered.
        reason: P2pDisconnectionReason,
    },

    /// Fatal connection error.
    Error {
        /// Connection address.
        addr: SocketAddr,
        /// Reason why disconneciton is triggered.
        error: P2pNetworkConnectionError,
    },

    /// Remote address is disconnected.
    ///
    /// Action that signals that the peer is disconnected.
    Disconnected {
        /// Connection address.
        addr: SocketAddr,
        /// Reason why the peer disconnected.
        reason: P2pNetworkConnectionCloseReason,
    },

    /// Prune connection.
    Prune {
        /// Connection address.
        addr: SocketAddr,
    },
}

impl From<P2pNetworkSchedulerAction> for crate::P2pAction {
    fn from(value: P2pNetworkSchedulerAction) -> Self {
        crate::P2pAction::Network(value.into())
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkSchedulerAction {
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
        #[allow(unused_variables)]
        match self {
            P2pNetworkSchedulerAction::InterfaceDetected { ip } => true,
            P2pNetworkSchedulerAction::InterfaceExpired { ip } => true,
            P2pNetworkSchedulerAction::ListenerReady { listener } => true,
            P2pNetworkSchedulerAction::ListenerError { listener, error } => true,
            P2pNetworkSchedulerAction::IncomingConnectionIsReady { listener } => true,
            P2pNetworkSchedulerAction::IncomingDidAccept { addr, result } => {
                addr.as_ref().map_or(false, |addr| {
                    state.network.scheduler.connections.get(addr).is_none()
                })
            }
            P2pNetworkSchedulerAction::OutgoingConnect { addr } => {
                state.network.scheduler.connections.get(addr).is_none()
            }
            P2pNetworkSchedulerAction::OutgoingDidConnect { addr, result } => state
                .network
                .scheduler
                .connections
                .get(addr)
                .map_or(false, |conn_state| !conn_state.incoming),
            P2pNetworkSchedulerAction::IncomingDataIsReady { addr } => true,
            P2pNetworkSchedulerAction::IncomingDataDidReceive { addr, result } => true,
            P2pNetworkSchedulerAction::SelectDone {
                addr,
                kind,
                protocol,
                incoming,
            } => true,
            P2pNetworkSchedulerAction::SelectError { addr, kind, error } => true,
            P2pNetworkSchedulerAction::YamuxDidInit { addr, peer_id } => true,
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
        }
    }
}
