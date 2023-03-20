use crate::{P2pPeerState, P2pPeerStatus};

use super::{
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
    }
}
