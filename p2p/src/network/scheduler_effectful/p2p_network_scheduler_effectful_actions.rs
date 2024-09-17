use std::net::{IpAddr, SocketAddr};

use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::{
    disconnection::P2pDisconnectionReason,
    select::{token, SelectKind},
    ConnectionAddr, P2pNetworkConnectionError, P2pState, PeerId,
};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(ip), display(listener), display(addr), debug(result), select_kind = debug(kind), display(error)))]
pub enum P2pNetworkSchedulerEffectfulAction {
    InterfaceDetected {
        ip: IpAddr,
    },
    IncomingConnectionIsReady {
        listener: SocketAddr,
    },
    #[action_event(fields(debug(addr), debug(result)))]
    IncomingDidAccept {
        addr: Option<ConnectionAddr>,
        result: Result<(), String>,
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
    IncomingDataIsReady {
        addr: ConnectionAddr,
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
}

impl From<P2pNetworkSchedulerEffectfulAction> for crate::P2pAction {
    fn from(value: P2pNetworkSchedulerEffectfulAction) -> Self {
        crate::P2pAction::Network(value.into())
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkSchedulerEffectfulAction {
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
        let conn = |addr| state.network.scheduler.connections.get(addr);

        match self {
            P2pNetworkSchedulerEffectfulAction::InterfaceDetected { .. } => true,
            P2pNetworkSchedulerEffectfulAction::IncomingConnectionIsReady { .. } => true,
            P2pNetworkSchedulerEffectfulAction::IncomingDidAccept { addr, .. } => {
                addr.as_ref().map_or(false, |addr| {
                    state.network.scheduler.connections.contains_key(addr)
                })
            }
            P2pNetworkSchedulerEffectfulAction::OutgoingConnect { addr } => conn(&ConnectionAddr {
                sock_addr: *addr,
                incoming: false,
            })
            .is_some(),
            P2pNetworkSchedulerEffectfulAction::OutgoingDidConnect { addr, .. } => state
                .network
                .scheduler
                .connections
                .get(addr)
                .map_or(false, |conn_state| !conn_state.incoming),
            P2pNetworkSchedulerEffectfulAction::IncomingDataIsReady { addr } => {
                conn(addr).is_some()
            }
            P2pNetworkSchedulerEffectfulAction::SelectDone { .. } => true,
            P2pNetworkSchedulerEffectfulAction::SelectError { .. } => true,
            P2pNetworkSchedulerEffectfulAction::Disconnect { addr, .. }
            | P2pNetworkSchedulerEffectfulAction::Error { addr, .. } => state
                .network
                .scheduler
                .connections
                .get(addr)
                .map_or(false, |conn_state| conn_state.closed.is_some()),
        }
    }
}
