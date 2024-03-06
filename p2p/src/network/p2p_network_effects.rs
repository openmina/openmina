use crate::channels::rpc::P2pChannelsRpcAction;

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
            Self::Rpc(v) => v.effects(meta, store),
        }
    }
}
