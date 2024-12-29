use redux::ActionMeta;

use crate::disconnection::P2pDisconnectionAction;

use super::{P2pDisconnectionEffectfulAction, P2pDisconnectionService};

impl P2pDisconnectionEffectfulAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pDisconnectionService,
    {
        match self {
            P2pDisconnectionEffectfulAction::Init { peer_id } => {
                if store.service().disconnect(peer_id) {
                    // peer was already disconnected, so dispatch finish.
                    store.dispatch(P2pDisconnectionAction::Finish { peer_id });
                }
            }
        }
    }
}
