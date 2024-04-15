use super::P2pNetworkFloodsubAction;
use redux::ActionMeta;

impl P2pNetworkFloodsubAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store) -> Result<(), String>
    where
        Store: crate::P2pStore<S>,
    {
        match self {
            crate::network::floodsub::P2pNetworkFloodsubAction::Stream(action) => {
                action.effects(meta, store)
            }
        }
    }
}
