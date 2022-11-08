use super::{P2pConnectionAction, P2pConnectionActionWithMetaRef, P2pConnectionState};

impl P2pConnectionState {
    pub fn reducer(&mut self, action: P2pConnectionActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            P2pConnectionAction::Outgoing(action) => {
                let P2pConnectionState::Outgoing(state) = self else { return };
                state.reducer(meta.with_action(action));
            }
        }
    }
}
