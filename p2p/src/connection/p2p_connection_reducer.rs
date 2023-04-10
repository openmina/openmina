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
            if let P2pConnectionOutgoingAction::Reconnect(a) = action {
                state.status = P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::Init {
                        time: meta.time(),
                        opts: a.opts.clone(),
                        rpc_id: a.rpc_id,
                    },
                ));
            }
            let P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(state)) = &mut state.status else { return };
            state.reducer(meta.with_action(action));
        }
        P2pConnectionAction::Incoming(action) => {
            if let P2pConnectionIncomingAction::Init(a) = action {
                state.status = P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                    P2pConnectionIncomingState::Init {
                        time: meta.time(),
                        signaling: a.signaling.clone(),
                        offer: a.offer.clone(),
                        rpc_id: a.rpc_id,
                    },
                ))
            }
            let P2pPeerStatus::Connecting(P2pConnectionState::Incoming(state)) = &mut state.status else { return };
            state.reducer(meta.with_action(action));
        }
    }
}
