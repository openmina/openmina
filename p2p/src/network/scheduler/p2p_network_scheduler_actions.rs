use std::net::{IpAddr, SocketAddr};

use openmina_core::{action_debug, action_info, action_warn, log::ActionEvent};
use serde::{Deserialize, Serialize};

use super::{
    super::{
        select::{token, SelectKind},
        Data,
    },
    p2p_network_scheduler_state::{P2pNetworkConnectionCloseReason, P2pNetworkConnectionError},
};

use crate::{disconnection::P2pDisconnectionReason, P2pState, PeerId};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pNetworkSchedulerAction {
    InterfaceDetected {
        ip: IpAddr,
    },
    InterfaceExpired {
        ip: IpAddr,
    },
    IncomingConnectionIsReady {
        listener: SocketAddr,
    },
    IncomingDidAccept {
        addr: Option<SocketAddr>,
        result: Result<(), String>,
    },
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

    /// Action that signals that the peer is disconnected.
    Disconnected {
        /// Connection address.
        addr: SocketAddr,
        /// Reason why the peer disconnected.
        reason: P2pNetworkConnectionCloseReason,
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
            P2pNetworkSchedulerAction::IncomingConnectionIsReady { listener } => true,
            P2pNetworkSchedulerAction::IncomingDidAccept { addr, result } => true,
            P2pNetworkSchedulerAction::OutgoingDidConnect { addr, result } => true,
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
        }
    }
}

impl ActionEvent for P2pNetworkSchedulerAction {
    fn action_event<T>(&self, context: &T)
    where
        T: openmina_core::log::EventContext,
    {
        match self {
            P2pNetworkSchedulerAction::InterfaceDetected { ip } => {
                action_debug!(context, ip = display(ip))
            }
            P2pNetworkSchedulerAction::InterfaceExpired { ip } => {
                action_debug!(context, ip = display(ip))
            }
            P2pNetworkSchedulerAction::IncomingConnectionIsReady { listener } => {
                action_debug!(context, listener = display(listener))
            }
            P2pNetworkSchedulerAction::IncomingDidAccept {
                addr,
                result: Ok(_),
            } => action_debug!(context, addr = debug(addr)),
            P2pNetworkSchedulerAction::IncomingDidAccept {
                addr,
                result: Err(error),
            } => action_debug!(context, addr = debug(addr), error = display(error)),
            P2pNetworkSchedulerAction::OutgoingDidConnect {
                addr,
                result: Ok(_),
            } => action_debug!(context, addr = display(addr)),
            P2pNetworkSchedulerAction::OutgoingDidConnect {
                addr,
                result: Err(error),
            } => action_debug!(context, addr = display(addr), error = display(error)),
            P2pNetworkSchedulerAction::IncomingDataIsReady { addr } => {
                action_debug!(context, addr = display(addr))
            }
            P2pNetworkSchedulerAction::IncomingDataDidReceive {
                addr,
                result: Ok(data),
            } => action_debug!(context, addr = display(addr), data = debug(data)),
            P2pNetworkSchedulerAction::IncomingDataDidReceive {
                addr,
                result: Err(error),
            } => action_debug!(context, addr = display(addr), error = display(error)),
            P2pNetworkSchedulerAction::SelectDone {
                addr,
                kind,
                protocol,
                incoming,
            } => action_debug!(
                context,
                addr = display(addr),
                select_kind = debug(kind),
                protocol = debug(protocol),
                incoming = incoming
            ),
            P2pNetworkSchedulerAction::SelectError { addr, kind, error } => action_warn!(
                context,
                addr = display(addr),
                select_kind = debug(kind),
                error = display(error)
            ),
            P2pNetworkSchedulerAction::YamuxDidInit { addr, peer_id } => {
                action_debug!(context, addr = display(addr), peer_id = display(peer_id))
            }
            P2pNetworkSchedulerAction::Disconnect { addr, reason } => {
                action_info!(context, addr = display(addr), reason = display(reason))
            }
            P2pNetworkSchedulerAction::Error { addr, error } => {
                action_warn!(context, addr = display(addr), error = display(error))
            }
            P2pNetworkSchedulerAction::Disconnected { addr, reason } => {
                action_info!(context, addr = display(addr), reason = display(reason))
            }
        }
    }
}
