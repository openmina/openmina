use crate::P2pNetworkYamuxAction;

use super::P2pNetworkPubsubAction;

impl P2pNetworkPubsubAction {
    pub fn effects<Store, S>(self, _meta: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
    {
        match self {
            Self::NewStream {
                incoming,
                peer_id,
                protocol,
                ..
            } => {
                dbg!((peer_id, protocol.name_str(), incoming));
            }
            Self::IncomingData { peer_id, data } => {
                dbg!((peer_id, data));
            }
            Self::Broadcast { data, topic } => {
                let peers = store
                    .state()
                    .network
                    .scheduler
                    .broadcast_state
                    .clients
                    .iter()
                    .filter(|(_, state)| state.topics.contains(&topic))
                    .map(|(v, _)| *v)
                    .collect::<Vec<_>>();
                for peer_id in peers {
                    store.dispatch(Self::OutgoingData {
                        data: data.clone(),
                        peer_id,
                    });
                }
            }
            Self::OutgoingData { data, peer_id } => {
                let Some(state) = store
                    .state()
                    .network
                    .scheduler
                    .broadcast_state
                    .clients
                    .get(&peer_id)
                else {
                    return;
                };
                store.dispatch(P2pNetworkYamuxAction::OutgoingData {
                    addr: state.addr,
                    stream_id: state.stream_id,
                    data,
                    fin: false,
                });
            }
        }
    }
}
