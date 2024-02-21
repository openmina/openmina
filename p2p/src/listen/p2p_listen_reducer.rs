use crate::{P2pListenerState, P2pListenersState};

use super::P2pListenActionWithMetaRef;

impl P2pListenersState {
    pub fn reducer(&mut self, action: P2pListenActionWithMetaRef) {
        let (action, _meta) = action.split();

        match action {
            super::P2pListenAction::New { listener_id, addr } => {
                if let P2pListenerState::Open { addrs, .. } =
                    self.0.entry(listener_id.clone()).or_default()
                {
                    addrs.insert(addr.clone());
                }
            }
            super::P2pListenAction::Expired { listener_id, addr } => {
                if let Some(P2pListenerState::Open { addrs, .. }) = self.0.get_mut(&listener_id) {
                    addrs.remove(&addr);
                }
            }
            super::P2pListenAction::Error { listener_id, error } => {
                if let P2pListenerState::Open { errors, .. } =
                    self.0.entry(listener_id.clone()).or_default()
                {
                    errors.push(error.clone());
                }
            }
            super::P2pListenAction::Closed { listener_id, error } => {
                let new_state = error
                    .clone()
                    .map_or(P2pListenerState::Closed, P2pListenerState::ClosedWithError);
                self.0.insert(listener_id.clone(), new_state);
            }
        }
    }
}
