use crate::{P2pPeerState, P2pPeerStatus};

use super::{
    incoming::{P2pConnectionIncomingAction, P2pConnectionIncomingState},
    outgoing::{P2pConnectionOutgoingAction, P2pConnectionOutgoingState},
    P2pConnectionAction, P2pConnectionActionWithMetaRef, P2pConnectionState,
};

pub fn p2p_connection_reducer(
    state: &mut P2pPeerState,
    action: P2pConnectionActionWithMetaRef<'_>,
) {
    let (action, meta) = action.split();
    match action {
        P2pConnectionAction::Outgoing(action) => {
            if let P2pConnectionOutgoingAction::Reconnect { opts, rpc_id } = action {
                state.status = P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::Init {
                        time: meta.time(),
                        opts: opts.clone(),
                        rpc_id: *rpc_id,
                    },
                ));
            }
            let P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(state)) = &mut state.status
            else {
                return;
            };
            state.reducer(meta.with_action(action));
        }
        P2pConnectionAction::Incoming(action) => {
            if let P2pConnectionIncomingAction::Init { opts, rpc_id } = action {
                state.status = P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionIncomingState::Init {
                        time: meta.time(),
                        signaling: opts.signaling.clone(),
                        offer: opts.offer.clone(),
                        rpc_id: *rpc_id,
                    },
                ))
            } else if let P2pConnectionIncomingAction::Libp2pReceived { .. } = action {
                state.status = P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionIncomingState::Libp2pReceived { time: meta.time() },
                ))
            }
            let P2pPeerStatus::Connecting(P2pConnectionState::Incoming(state)) = &mut state.status
            else {
                return;
            };
            state.reducer(meta.with_action(action));
        }
    }
}
