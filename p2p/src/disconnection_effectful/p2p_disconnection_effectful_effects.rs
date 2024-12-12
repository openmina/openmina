use redux::ActionMeta;

use super::{P2pDisconnectionEffectfulAction, P2pDisconnectionService};

impl P2pDisconnectionEffectfulAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pDisconnectionService,
    {
        match self {
            P2pDisconnectionEffectfulAction::Init { peer_id } => {
                store.service().disconnect(peer_id);
            }
        }
    }
}
