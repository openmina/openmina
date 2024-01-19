use redux::{ActionMeta, EnablingCondition};

use crate::{connection::libp2p::P2pConnectionLibP2pService, peer::P2pPeerReadyAction};

use super::P2pConnectionLibP2pIncomingSuccessAction;

impl P2pConnectionLibP2pIncomingSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionLibP2pService,
        P2pPeerReadyAction: EnablingCondition<S>,
    {
        store.dispatch(P2pPeerReadyAction {
            incoming: true,
            peer_id: self.peer_id,
        });
    }
}
