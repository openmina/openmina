use std::net::SocketAddr;

use crate::{ConnectionAddr, P2pState, PeerId, StreamId};
use mina_core::ActionEvent;
use redux::EnablingCondition;
use serde::{Deserialize, Serialize};

/// Identify stream effectful actions.
#[derive(Debug, Clone, Serialize, Deserialize, ActionEvent)]
pub enum P2pNetworkIdentifyStreamEffectfulAction {
    GetListenAddresses {
        addr: ConnectionAddr,
        peer_id: PeerId,
        stream_id: StreamId,
        addresses: Vec<SocketAddr>,
    },
}

impl EnablingCondition<P2pState> for P2pNetworkIdentifyStreamEffectfulAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        true
    }
}

impl From<P2pNetworkIdentifyStreamEffectfulAction> for crate::P2pEffectfulAction {
    fn from(value: P2pNetworkIdentifyStreamEffectfulAction) -> crate::P2pEffectfulAction {
        crate::P2pEffectfulAction::Network(crate::P2pNetworkEffectfulAction::Identify(
            crate::network::identify::P2pNetworkIdentifyEffectfulAction::Stream(value),
        ))
    }
}
