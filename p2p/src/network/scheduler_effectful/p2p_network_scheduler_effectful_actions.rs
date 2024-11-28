use std::net::{IpAddr, SocketAddr};

use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::{ConnectionAddr, P2pNetworkConnectionCloseReason, P2pState};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(ip), display(listener), display(addr), debug(result), select_kind = debug(kind), display(error)))]
pub enum P2pNetworkSchedulerEffectfulAction {
    InterfaceDetected {
        ip: IpAddr,
        port: u16,
    },
    IncomingConnectionIsReady {
        listener: SocketAddr,
        should_accept: bool,
    },
    #[action_event(fields(debug(addr), debug(result)))]
    IncomingDidAccept {
        addr: ConnectionAddr,
        result: Result<(), String>,
    },
    /// Initialize outgoing connection.
    OutgoingConnect {
        addr: SocketAddr,
    },
    /// Outgoing TCP stream is established.
    OutgoingDidConnect {
        addr: ConnectionAddr,
    },
    IncomingDataIsReady {
        addr: ConnectionAddr,
        limit: usize,
    },
    NoiseSelectDone {
        addr: ConnectionAddr,
        incoming: bool,
    },
    /// Action that initiate the specified peer disconnection.
    Disconnect {
        /// Connection address.
        addr: ConnectionAddr,
        /// Reason why disconnection is triggered.
        reason: P2pNetworkConnectionCloseReason,
    },
}

impl From<P2pNetworkSchedulerEffectfulAction> for crate::P2pEffectfulAction {
    fn from(value: P2pNetworkSchedulerEffectfulAction) -> crate::P2pEffectfulAction {
        crate::P2pEffectfulAction::Network(crate::P2pNetworkEffectfulAction::Scheduler(value))
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkSchedulerEffectfulAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}
