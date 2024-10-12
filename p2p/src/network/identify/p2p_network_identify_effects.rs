use super::p2p_network_identify_actions::P2pNetworkIdentifyEffectfulAction;
use crate::P2pNetworkService;
use redux::ActionMeta;

impl P2pNetworkIdentifyEffectfulAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pNetworkService,
    {
        match self {
            P2pNetworkIdentifyEffectfulAction::Stream(action) => action.effects(meta, store),
        }
    }
}
