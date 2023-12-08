use crate::{P2pListenerState, P2pListenersState};

use super::P2pListenActionWithMetaRef;

impl P2pListenersState {
    pub fn reducer(&mut self, action: P2pListenActionWithMetaRef) {
        let (action, _meta) = action.split();

        match action {
            super::P2pListenAction::New(action) => {
                if let P2pListenerState::Open { addrs, .. } =
                    self.0.entry(action.listener_id.clone()).or_default()
                {
                    addrs.insert(action.addr.clone());
                }
            }
            super::P2pListenAction::Expired(action) => {
                if let Some(P2pListenerState::Open { addrs, .. }) =
                    self.0.get_mut(&action.listener_id)
                {
                    addrs.remove(&action.addr);
                }
            }
            super::P2pListenAction::Error(action) => {
                if let P2pListenerState::Open { errors, .. } =
                    self.0.entry(action.listener_id.clone()).or_default()
                {
                    errors.push(action.error.clone());
                }
            }
            super::P2pListenAction::Closed(action) => {
                let new_state = action
                    .error
                    .clone()
                    .map_or(P2pListenerState::Closed, P2pListenerState::ClosedWithError);
                self.0.insert(action.listener_id.clone(), new_state);
            }
        }
    }
}
