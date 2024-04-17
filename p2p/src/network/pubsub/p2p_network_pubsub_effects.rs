use super::P2pNetworkPubsubAction;

impl P2pNetworkPubsubAction {
    pub fn effects<Store, S>(self, meta: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
    {
        let _ = (meta, store);
        match self {
            Self::NewStream {
                incoming,
                peer_id,
                protocol,
            } => {
                dbg!((peer_id, protocol.name_str(), incoming));
            }
            Self::IncomingData { peer_id, data } => {
                dbg!((peer_id, data));
            }
            Self::Broadcast { data, topic } => {
                let _ = (data, topic);
            }
        }
    }
}
