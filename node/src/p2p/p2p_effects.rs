use super::P2pActionWithMeta;
use crate::{P2pAction, Service, Store};
use p2p::{
    channels::P2pChannelsEffectfulAction, connection::P2pConnectionEffectfulAction,
    P2pInitializeAction,
};

pub fn node_p2p_effects<S: Service>(store: &mut Store<S>, action: P2pActionWithMeta) {
    let (action, meta) = action.split();

    match action {
        P2pAction::Initialization(P2pInitializeAction::Initialize { .. }) => {
            #[cfg(feature = "p2p-libp2p")]
            if store.state().p2p.ready().is_some() {
                store.service().start_mio();
            }
        }
        P2pAction::DisconnectionEffectful(action) => action.effects(&meta, store),
        P2pAction::ChannelsEffectful(action) => match action {
            P2pChannelsEffectfulAction::BestTip(action) => action.effects(&meta, store),
            P2pChannelsEffectfulAction::Transaction(action) => action.effects(&meta, store),
            P2pChannelsEffectfulAction::StreamingRpc(action) => action.effects(&meta, store),
            P2pChannelsEffectfulAction::Snark(action) => action.effects(&meta, store),
            P2pChannelsEffectfulAction::Rpc(action) => action.effects(&meta, store),
            P2pChannelsEffectfulAction::SnarkJobCommitment(action) => action.effects(&meta, store),
        },
        P2pAction::Network(_action) => {
            #[cfg(feature = "p2p-libp2p")]
            _action.effects(&meta, store);
        }
        P2pAction::ConnectionEffectful(action) => match action {
            P2pConnectionEffectfulAction::Outgoing(action) => action.effects(&meta, store),
            P2pConnectionEffectfulAction::Incoming(action) => action.effects(&meta, store),
        },
        P2pAction::Peer(_)
        | P2pAction::Channels(_)
        | P2pAction::Connection(_)
        | P2pAction::Disconnection(_)
        | P2pAction::Identify(_) => {
            // handled by reducer
        }
    }
}
