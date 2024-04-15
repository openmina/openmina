use openmina_core::error;

use super::*;

impl P2pNetworkAction {
    pub fn effects<Store, S>(self, meta: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService + P2pCryptoService,
    {
        match self {
            Self::Scheduler(v) => v.effects(meta, store),
            Self::Pnet(v) => v.effects(meta, store),
            Self::Select(v) => v.effects(meta, store),
            Self::Noise(v) => v.effects(meta, store),
            Self::Yamux(v) => v.effects(meta, store),
            Self::Identify(v) => match v.effects(meta, store) {
                Ok(_) => {}
                Err(e) => error!(meta.time(); "error dispatching Identify stream action: {e}"),
            },
            Self::Floodsub(v) => match v.effects(meta, store) {
                Ok(_) => {}
                Err(e) => error!(meta.time(); "error dispatching Floodsub stream action: {e}"),
            },
            Self::Kad(v) => match v.effects(meta, store) {
                Ok(_) => {}
                Err(e) => error!(meta.time(); "error dispatching Kademlia action: {e}"),
            },
            Self::Rpc(v) => v.effects(meta, store),
        }
    }
}
