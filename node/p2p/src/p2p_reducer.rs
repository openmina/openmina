use crate::connection::{P2pConnectionAction, P2pConnectionState};
use crate::{P2pAction, P2pActionWithMetaRef, P2pPeerState, P2pPeerStatus, P2pState};

impl P2pState {
    pub fn reducer(&mut self, action: P2pActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            P2pAction::Connection(action) => {
                let peer = if action.should_create_peer() {
                    self.peers
                        .entry(*action.peer_id())
                        .or_insert_with(|| P2pPeerState {
                            status: P2pPeerStatus::Connecting(match &action {
                                P2pConnectionAction::Outgoing(_) => {
                                    P2pConnectionState::Outgoing(Default::default())
                                }
                            }),
                        })
                } else {
                    match self.peers.get_mut(action.peer_id()) {
                        Some(v) => v,
                        None => return,
                    }
                };
                let P2pPeerStatus::Connecting(state) = &mut peer.status else { return };
                state.reducer(meta.with_action(action));
            }
            P2pAction::Pubsub(_) => {}
        }
    }
}
