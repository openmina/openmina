use crate::{
    channels::P2pChannelsEffectfulAction, connection::P2pConnectionEffectfulAction,
    P2pEffectfulAction, P2pStore,
};
use redux::ActionMeta;

impl P2pEffectfulAction {
    pub fn effects<Store, S>(self, meta: ActionMeta, store: &mut Store)
    where
        Store: P2pStore<S>,
        Store::Service: crate::P2pService,
    {
        match self {
            P2pEffectfulAction::Initialize => {}
            P2pEffectfulAction::Channels(action) => match action {
                P2pChannelsEffectfulAction::BestTip(action) => action.effects(&meta, store),
                P2pChannelsEffectfulAction::Transaction(action) => action.effects(&meta, store),
                P2pChannelsEffectfulAction::StreamingRpc(action) => action.effects(&meta, store),
                P2pChannelsEffectfulAction::SnarkJobCommitment(action) => {
                    action.effects(&meta, store)
                }
                P2pChannelsEffectfulAction::Rpc(action) => action.effects(&meta, store),
                P2pChannelsEffectfulAction::Snark(action) => action.effects(&meta, store),
            },
            P2pEffectfulAction::Connection(action) => match action {
                P2pConnectionEffectfulAction::Outgoing(action) => action.effects(&meta, store),
                P2pConnectionEffectfulAction::Incoming(action) => action.effects(&meta, store),
            },
            P2pEffectfulAction::Disconnection(action) => action.effects(&meta, store),
            #[cfg(feature = "p2p-libp2p")]
            P2pEffectfulAction::Network(action) => action.effects(&meta, store),
            #[cfg(not(feature = "p2p-libp2p"))]
            P2pEffectfulAction::Network(action) => {}
        }
    }
}
