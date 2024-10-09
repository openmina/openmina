use crate::{
    channels::P2pChannelsEffectfulAction, connection::P2pConnectionEffectfulAction, P2pAction,
    P2pStore,
};
use redux::ActionWithMeta;

pub fn p2p_effects<Store, S>(store: &mut Store, action: ActionWithMeta<P2pAction>)
where
    Store: P2pStore<S>,
    Store::Service: crate::P2pService,
{
    let (action, meta) = action.split();
    match action {
        P2pAction::Initialization(_) => {
            // Noop
        }
        P2pAction::ConnectionEffectful(action) => match action {
            P2pConnectionEffectfulAction::Outgoing(action) => action.effects(&meta, store),
            P2pConnectionEffectfulAction::Incoming(action) => action.effects(&meta, store),
        },
        P2pAction::DisconnectionEffectful(action) => action.effects(&meta, store),

        P2pAction::ChannelsEffectful(action) => match action {
            P2pChannelsEffectfulAction::BestTip(action) => action.effects(&meta, store),
            P2pChannelsEffectfulAction::Transaction(action) => action.effects(&meta, store),
            P2pChannelsEffectfulAction::StreamingRpc(action) => action.effects(&meta, store),
            P2pChannelsEffectfulAction::SnarkJobCommitment(action) => action.effects(&meta, store),
            P2pChannelsEffectfulAction::Rpc(action) => action.effects(&meta, store),
            P2pChannelsEffectfulAction::Snark(action) => action.effects(&meta, store),
        },
        P2pAction::Network(_action) => {
            #[cfg(feature = "p2p-libp2p")]
            _action.effects(&meta, store);
        }
        P2pAction::Connection(_)
        | P2pAction::Channels(_)
        | P2pAction::Disconnection(_)
        | P2pAction::Peer(_)
        | P2pAction::Identify(_) => {
            // handled by reducer
        }
    }
}
