use crate::{ConnectionAddr, P2pAction, P2pState, PeerId, StreamId};
use openmina_core::ActionEvent;
use redux::EnablingCondition;
use serde::{Deserialize, Serialize};

/// Identify stream related actions.
#[derive(Debug, Clone, Serialize, Deserialize, ActionEvent)]
pub enum P2pNetworkIdentifyStreamEffectfulAction {
    /// Creates a new stream state.
    SendIdentify {
        addr: ConnectionAddr,
        peer_id: PeerId,
        stream_id: StreamId,
    },
}

macro_rules! enum_field {
    ($field:ident: $field_type:ty) => {
        pub fn $field(&self) -> &$field_type {
            match self {
                P2pNetworkIdentifyStreamEffectfulAction::SendIdentify { $field, .. } => $field,
            }
        }
    };
}

impl P2pNetworkIdentifyStreamEffectfulAction {
    enum_field!(addr: ConnectionAddr);
    enum_field!(peer_id: PeerId);
    enum_field!(stream_id: StreamId);
}

impl EnablingCondition<P2pState> for P2pNetworkIdentifyStreamEffectfulAction {
    fn is_enabled(&self, _state: &P2pState, _time: redux::Timestamp) -> bool {
        // TODO
        true
    }
}

impl From<P2pNetworkIdentifyStreamEffectfulAction> for P2pAction {
    fn from(value: P2pNetworkIdentifyStreamEffectfulAction) -> Self {
        P2pAction::Network(super::super::P2pNetworkIdentifyAction::StreamEffectful(value).into())
    }
}
