use redux::ActionMeta;

use crate::P2pPeerReadyAction;
use crate::{connection::P2pConnectionService, webrtc};

use super::{
    P2pConnectionOutgoingAnswerRecvErrorAction, P2pConnectionOutgoingAnswerRecvPendingAction,
    P2pConnectionOutgoingAnswerRecvSuccessAction, P2pConnectionOutgoingErrorAction,
    P2pConnectionOutgoingFinalizeErrorAction, P2pConnectionOutgoingFinalizePendingAction,
    P2pConnectionOutgoingFinalizeSuccessAction, P2pConnectionOutgoingInitAction,
    P2pConnectionOutgoingOfferReadyAction, P2pConnectionOutgoingOfferSdpCreatePendingAction,
    P2pConnectionOutgoingOfferSdpCreateSuccessAction, P2pConnectionOutgoingOfferSendSuccessAction,
    P2pConnectionOutgoingRandomInitAction, P2pConnectionOutgoingReconnectAction,
    P2pConnectionOutgoingSuccessAction,
};

impl P2pConnectionOutgoingRandomInitAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionOutgoingInitAction: redux::EnablingCondition<S>,
    {
        let peers = store.state().initial_unused_peers();
        let picked_peer = store.service().random_pick(&peers);
        store.dispatch(P2pConnectionOutgoingInitAction {
            opts: picked_peer,
            rpc_id: None,
        });
    }
}

impl P2pConnectionOutgoingInitAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionOutgoingOfferSdpCreatePendingAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.opts.peer_id;
        store.service().outgoing_init(peer_id);
        store.dispatch(P2pConnectionOutgoingOfferSdpCreatePendingAction { peer_id });
    }
}

impl P2pConnectionOutgoingReconnectAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionOutgoingOfferSdpCreatePendingAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.opts.peer_id;
        store.service().outgoing_init(peer_id);
        store.dispatch(P2pConnectionOutgoingOfferSdpCreatePendingAction { peer_id });
    }
}

impl P2pConnectionOutgoingOfferSdpCreateSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionOutgoingOfferReadyAction: redux::EnablingCondition<S>,
    {
        let offer = webrtc::Offer {
            sdp: self.sdp,
            identity_pub_key: store.state().config.identity_pub_key.clone(),
            target_peer_id: self.peer_id,
        };
        store.dispatch(P2pConnectionOutgoingOfferReadyAction {
            peer_id: self.peer_id,
            offer,
        });
    }
}

impl P2pConnectionOutgoingOfferReadyAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionOutgoingOfferSendSuccessAction: redux::EnablingCondition<S>,
    {
        let (state, service) = store.state_and_service();
        let Some(peer) = state.peers.get(&self.peer_id) else { return };
        // TODO(binier): use dial_opts from outgoing state instead.
        let signaling_method = &peer.dial_opts.signaling;
        match signaling_method {
            webrtc::SignalingMethod::Http(_) | webrtc::SignalingMethod::Https(_) => {
                service.set_offer(self.offer.clone());
                let Some(url) = signaling_method.http_url() else { return };
                service.http_signal_send(url, self.offer.into());
            }
        }
        store.dispatch(P2pConnectionOutgoingOfferSendSuccessAction {
            peer_id: self.peer_id,
        });
    }
}

impl P2pConnectionOutgoingOfferSendSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionOutgoingAnswerRecvPendingAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionOutgoingAnswerRecvPendingAction {
            peer_id: self.peer_id,
        });
    }
}

impl P2pConnectionOutgoingAnswerRecvErrorAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionOutgoingErrorAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionOutgoingErrorAction {
            peer_id: self.peer_id,
            error: self.error,
        });
    }
}

impl P2pConnectionOutgoingAnswerRecvSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionOutgoingFinalizePendingAction: redux::EnablingCondition<S>,
    {
        store.service().set_answer(self.answer.clone());
        store.dispatch(P2pConnectionOutgoingFinalizePendingAction {
            peer_id: self.peer_id,
        });
    }
}

impl P2pConnectionOutgoingFinalizeErrorAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionOutgoingErrorAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionOutgoingErrorAction {
            peer_id: self.peer_id,
            error: self.error,
        });
    }
}

impl P2pConnectionOutgoingFinalizeSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionOutgoingSuccessAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionOutgoingSuccessAction {
            peer_id: self.peer_id,
        });
    }
}

impl P2pConnectionOutgoingSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pPeerReadyAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        store.dispatch(P2pPeerReadyAction { peer_id });
    }
}
