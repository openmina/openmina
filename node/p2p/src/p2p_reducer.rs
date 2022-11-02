use crate::{P2pAction, P2pActionWithMetaRef, P2pState};

impl P2pState {
    pub fn reducer(&mut self, action: P2pActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        // match action {
        // }
    }
}
