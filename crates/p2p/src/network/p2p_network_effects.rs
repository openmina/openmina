use super::*;

impl P2pNetworkEffectfulAction {
    pub fn effects<Store, S>(self, meta: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService + P2pCryptoService + P2pNetworkService,
    {
        match self {
            P2pNetworkEffectfulAction::Scheduler(a) => a.effects(meta, store),
            P2pNetworkEffectfulAction::Pnet(v) => v.effects(meta, store),
            P2pNetworkEffectfulAction::Pubsub(v) => v.effects(meta, store),
            P2pNetworkEffectfulAction::Identify(v) => v.effects(meta, store),
            P2pNetworkEffectfulAction::Kad(v) => v.effects(meta, store),
        }
    }
}
