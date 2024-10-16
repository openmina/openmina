use openmina_core::ActionEvent;
use redux::Timestamp;
use serde::{Deserialize, Serialize};

use crate::{
    channels::{
        rpc::{P2pRpcId, P2pRpcRequest, P2pRpcResponse},
        P2pChannelsEffectfulAction,
    },
    P2pState, PeerId,
};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(peer_id)))]
pub enum P2pChannelsRpcEffectfulAction {
    Init {
        peer_id: PeerId,
    },
    RequestSend {
        peer_id: PeerId,
        id: P2pRpcId,
        request: Box<P2pRpcRequest>,
        on_init: Option<redux::Callback<(PeerId, P2pRpcId, P2pRpcRequest)>>,
    },
    ResponseSend {
        peer_id: PeerId,
        id: P2pRpcId,
        response: Option<Box<P2pRpcResponse>>,
    },
}

impl redux::EnablingCondition<P2pState> for P2pChannelsRpcEffectfulAction {
    fn is_enabled(&self, _state: &P2pState, _time: Timestamp) -> bool {
        true
    }
}

impl From<P2pChannelsRpcEffectfulAction> for crate::P2pEffectfulAction {
    fn from(a: P2pChannelsRpcEffectfulAction) -> crate::P2pEffectfulAction {
        crate::P2pEffectfulAction::Channels(P2pChannelsEffectfulAction::Rpc(a))
    }
}
