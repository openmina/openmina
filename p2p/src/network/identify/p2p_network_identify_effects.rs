use crate::P2pNetworkService;

use super::P2pNetworkIdentifyAction;
use redux::ActionMeta;

impl P2pNetworkIdentifyAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store) -> Result<(), String>
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pNetworkService,
    {
        match self {
            crate::network::identify::P2pNetworkIdentifyAction::Stream(action) => {
                action.effects(meta, store)
            }
        }
    }
}
