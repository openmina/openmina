use openmina_core::error;

use super::*;

impl P2pNetworkAction {
    pub fn effects<Store, S>(self, meta: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pMioService + P2pCryptoService + P2pNetworkService,
    {
        match self {
            P2pNetworkAction::Identify(v) => match v.effects(meta, store) {
                Ok(_) => {}
                Err(e) => error!(meta.time(); "error dispatching Identify stream action: {e}"),
            },
            P2pNetworkAction::Pnet(_)
            | P2pNetworkAction::Select(_)
            | P2pNetworkAction::Noise(_)
            | P2pNetworkAction::Yamux(_)
            | P2pNetworkAction::Kad(_)
            | P2pNetworkAction::Pubsub(_)
            | P2pNetworkAction::Rpc(_)
            | P2pNetworkAction::Scheduler(_) => {
                // handled by reducer
            }
            P2pNetworkAction::SchedulerEffectful(v) => v.effects(meta, store),
            P2pNetworkAction::PubsubEffectful(v) => v.effects(meta, store),
            P2pNetworkAction::PnetEffectful(v) => v.effects(meta, store),
        }
    }
}
