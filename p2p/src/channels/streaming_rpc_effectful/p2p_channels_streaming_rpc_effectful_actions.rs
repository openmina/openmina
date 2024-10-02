use crate::{
    channels::{
        streaming_rpc::{
            P2pStreamingRpcId, P2pStreamingRpcRequest, P2pStreamingRpcResponse,
            P2pStreamingRpcResponseFull,
        },
        P2pChannelsEffectfulAction,
    },
    P2pState, PeerId,
};
use openmina_core::ActionEvent;
use redux::Timestamp;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(peer_id)))]
pub enum P2pChannelsStreamingRpcEffectfulAction {
    Init {
        peer_id: PeerId,
    },
    RequestSend {
        peer_id: PeerId,
        id: P2pStreamingRpcId,
        request: Box<P2pStreamingRpcRequest>,
        on_init: Option<redux::Callback<(PeerId, P2pStreamingRpcId, P2pStreamingRpcRequest)>>,
    },
    ResponseNextPartGet {
        peer_id: PeerId,
        id: P2pStreamingRpcId,
    },
    ResponseSendInit {
        peer_id: PeerId,
        id: P2pStreamingRpcId,
        response: Option<P2pStreamingRpcResponseFull>,
    },
    ResponsePartSend {
        peer_id: PeerId,
        id: P2pStreamingRpcId,
        response: Box<P2pStreamingRpcResponse>,
    },
}

impl redux::EnablingCondition<P2pState> for P2pChannelsStreamingRpcEffectfulAction {
    fn is_enabled(&self, _state: &P2pState, _time: Timestamp) -> bool {
        true
    }
}

impl From<P2pChannelsStreamingRpcEffectfulAction> for crate::P2pAction {
    fn from(a: P2pChannelsStreamingRpcEffectfulAction) -> crate::P2pAction {
        crate::P2pAction::ChannelsEffectful(P2pChannelsEffectfulAction::StreamingRpc(a))
    }
}
