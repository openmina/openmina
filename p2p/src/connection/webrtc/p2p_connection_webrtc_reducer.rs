use openmina_core::error;
use redux::ActionWithMeta;

use crate::{connection::P2pConnectionState, P2pPeerStatus, P2pWebRTCPeerState};

use super::{
    incoming::{P2pConnectionWebRTCIncomingAction, P2pConnectionWebRTCIncomingState},
    P2pConnectionWebRTCAction,
};

pub fn p2p_connection_webrtc_reducer(
    state: &mut P2pWebRTCPeerState,
    action: ActionWithMeta<&'_ P2pConnectionWebRTCAction>,
) {
    let (action, meta) = action.split();
    match action {
        P2pConnectionWebRTCAction::Outgoing(action) => {
            // if let P2pConnectionWebRTCOutgoingAction::Reconnect(a) = action {
            //     state.status = P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
            //         P2pConnectionWebRTCOutgoingState::Init {
            //             time: meta.time(),
            //             rpc_id: a.rpc_id,
            //         },
            //     ));
            // }
            let P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(state)) = &mut state.status
            else {
                error!(meta.time(); "incorrect peer status: {:?}", state.status);
                return;
            };
            state.reducer(meta.with_action(action));
        }
        P2pConnectionWebRTCAction::Incoming(action) => {
            if let P2pConnectionWebRTCIncomingAction::Init(a) = action {
                state.status = P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionWebRTCIncomingState::Init {
                        time: meta.time(),
                        signaling: a.opts.signaling.clone(),
                        offer: a.opts.offer.clone(),
                        rpc_id: a.rpc_id,
                    },
                ))
            } else if let P2pConnectionWebRTCIncomingAction::Libp2pReceived(_) = action {
                state.status = P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionWebRTCIncomingState::Libp2pReceived { time: meta.time() },
                ))
            }
            let P2pPeerStatus::Connecting(P2pConnectionState::Incoming(state)) = &mut state.status
            else {
                error!(meta.time(); "incorrect peer status: {:?}", state.status);
                return;
            };
            state.reducer(meta.with_action(action));
        }
    }
}
