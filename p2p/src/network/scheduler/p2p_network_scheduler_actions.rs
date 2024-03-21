use std::net::{IpAddr, SocketAddr};

use serde::{Deserialize, Serialize};

use super::super::{
    select::{token, SelectKind},
    Data,
};

use crate::{P2pState, PeerId};

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
}

impl From<P2pNetworkSchedulerAction> for crate::P2pAction {
    fn from(value: P2pNetworkSchedulerAction) -> Self {
        crate::P2pAction::Network(value.into())
    }
}

impl redux::EnablingCondition<P2pState> for P2pNetworkSchedulerAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
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
        }
    }
}
