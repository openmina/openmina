use openmina_core::ActionEvent;
use p2p::{
    channels::{
        rpc::{P2pRpcId, P2pRpcRequest, P2pRpcResponse},
        streaming_rpc::P2pStreamingRpcResponseFull,
    },
    PeerId,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(level = debug)]
pub enum P2pCallbacksAction {
    P2pChannelsRpcReady {
        peer_id: PeerId,
    },
    P2pChannelsRpcTimeout {
        peer_id: PeerId,
        id: P2pRpcId,
    },
    P2pChannelsRpcResponseReceived {
        peer_id: PeerId,
        id: P2pRpcId,
        response: Option<Box<P2pRpcResponse>>,
    },
    P2pChannelsRpcRequestReceived {
        peer_id: PeerId,
        id: P2pRpcId,
        request: Box<P2pRpcRequest>,
    },

    P2pChannelsStreamingRpcReady,
    P2pChannelsStreamingRpcTimeout {
        peer_id: PeerId,
        id: P2pRpcId,
    },
    P2pChannelsStreamingRpcResponseReceived {
        peer_id: PeerId,
        id: P2pRpcId,
        response: Option<P2pStreamingRpcResponseFull>,
    },
}

impl redux::EnablingCondition<crate::State> for P2pCallbacksAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        true
    }
}
