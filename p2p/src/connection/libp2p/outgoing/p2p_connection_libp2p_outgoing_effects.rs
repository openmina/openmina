use openmina_core::error;
use redux::{ActionMeta, EnablingCondition};

use crate::{
    connection::{libp2p::P2pConnectionLibP2pService, ConnectionState, P2pConnectionState},
    peer::P2pPeerReadyAction,
    P2pLibP2pPeerState,
};

use super::*;

impl P2pConnectionLibP2pOutgoingInitAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionLibP2pService,
        P2pConnectionLibP2pOutgoingFinalizePendingAction: EnablingCondition<S>,
    {
        let Some(P2pLibP2pPeerState { dial_opts, .. }) =
            store.state().get_libp2p_peer(&self.peer_id)
        else {
            error!(meta.time(); "incorrect peer state for peer {}: {:?}", self.peer_id, store.state().peers.get(&self.peer_id));
            return;
        };
        let addrs = dial_opts.clone();
        store.service().outgoing_init(self.peer_id, addrs);
        store.dispatch(P2pConnectionLibP2pOutgoingFinalizePendingAction {
            peer_id: self.peer_id,
        });
    }
}

impl P2pConnectionLibP2pOutgoingFinalizePendingAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionLibP2pService,
    {
        // noop
    }
}

impl P2pConnectionLibP2pOutgoingFinalizeSuccessAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionLibP2pService,
        P2pConnectionLibP2pOutgoingSuccessAction: EnablingCondition<S>,
    {
        let Some(as_connecting) = store
            .state()
            .get_libp2p_peer(&self.peer_id)
            .and_then(|state| state.status.as_connecting())
        else {
            error!(meta.time(); "incorrect peer state for peer {}: {:?}", self.peer_id, store.state().peers.get(&self.peer_id));
            return;
        };
        store.dispatch(P2pConnectionLibP2pOutgoingSuccessAction {
            peer_id: self.peer_id,
            rpc_id: as_connecting.rpc_id(),
        });
    }
}

impl P2pConnectionLibP2pOutgoingFinalizeErrorAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionLibP2pService,
        P2pConnectionLibP2pOutgoingErrorAction: EnablingCondition<S>,
    {
        let Some(as_connecting) = store
            .state()
            .get_libp2p_peer(&self.peer_id)
            .and_then(|state| state.status.as_connecting())
        else {
            error!(meta.time(); "incorrect peer state for peer {}: {:?}", self.peer_id, store.state().peers.get(&self.peer_id));
            return;
        };
        let P2pConnectionState::Outgoing(P2pConnectionLibP2pOutgoingState::Error(s)) =
            as_connecting
        else {
            error!(meta.time(); "incorrect peer connecting state for peer {}: {:?}", self.peer_id, as_connecting);
            return;
        };
        store.dispatch(P2pConnectionLibP2pOutgoingErrorAction {
            peer_id: self.peer_id,
            error: s.error.clone(),
            rpc_id: as_connecting.rpc_id(),
        });
    }
}

impl P2pConnectionLibP2pOutgoingFinalizeTimeoutAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionLibP2pService,
        P2pConnectionLibP2pOutgoingErrorAction: EnablingCondition<S>,
    {
        let Some(as_connecting) = store
            .state()
            .get_libp2p_peer(&self.peer_id)
            .and_then(|state| state.status.as_connecting())
        else {
            error!(meta.time(); "incorrect peer state for peer {}: {:?}", self.peer_id, store.state().peers.get(&self.peer_id));
            return;
        };
        let P2pConnectionState::Outgoing(P2pConnectionLibP2pOutgoingState::Error(s)) =
            as_connecting
        else {
            error!(meta.time(); "incorrect peer connecting state for peer {}: {:?}", self.peer_id, as_connecting);
            return;
        };
        store.dispatch(P2pConnectionLibP2pOutgoingErrorAction {
            peer_id: self.peer_id,
            error: s.error.clone(),
            rpc_id: as_connecting.rpc_id(),
        });
    }
}

impl P2pConnectionLibP2pOutgoingSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionLibP2pService,
        P2pPeerReadyAction: EnablingCondition<S>,
    {
        store.dispatch(P2pPeerReadyAction {
            peer_id: self.peer_id,
            incoming: true,
        });
    }
}

impl P2pConnectionLibP2pOutgoingErrorAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, _: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionLibP2pService,
    {
        // noop
    }
}
