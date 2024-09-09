use openmina_core::error;

use super::*;

impl P2pNetworkAction {
    pub fn effects<Store, S>(self, meta: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService + P2pCryptoService + P2pNetworkService,
    {
        match self {
            P2pNetworkAction::Scheduler(v) => v.effects(meta, store),
            P2pNetworkAction::Pnet(v) => v.effects(meta, store),
            P2pNetworkAction::Select(v) => v.effects(meta, store),

            P2pNetworkAction::Identify(v) => match v.effects(meta, store) {
                Ok(_) => {}
                Err(e) => error!(meta.time(); "error dispatching Identify stream action: {e}"),
            },
            P2pNetworkAction::Noise(_) | P2pNetworkAction::Yamux(_) | P2pNetworkAction::Kad(_) => {
                // handled by reducer
            }
            P2pNetworkAction::Pubsub(v) => v.effects(meta, store),
            P2pNetworkAction::Rpc(v) => v.effects(meta, store),
        }
    }
}
