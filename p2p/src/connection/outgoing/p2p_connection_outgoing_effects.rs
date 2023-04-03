use redux::ActionMeta;

use crate::connection::{P2pConnectionErrorResponse, P2pConnectionState};
use crate::{connection::P2pConnectionService, webrtc};
use crate::{P2pPeerReadyAction, P2pPeerStatus};

use super::{
    P2pConnectionOutgoingAnswerRecvErrorAction, P2pConnectionOutgoingAnswerRecvPendingAction,
    P2pConnectionOutgoingAnswerRecvSuccessAction, P2pConnectionOutgoingError,
    P2pConnectionOutgoingErrorAction, P2pConnectionOutgoingFinalizeErrorAction,
    P2pConnectionOutgoingFinalizePendingAction, P2pConnectionOutgoingFinalizeSuccessAction,
    P2pConnectionOutgoingInitAction, P2pConnectionOutgoingOfferReadyAction,
    P2pConnectionOutgoingOfferSdpCreateErrorAction,
    P2pConnectionOutgoingOfferSdpCreatePendingAction,
    P2pConnectionOutgoingOfferSdpCreateSuccessAction, P2pConnectionOutgoingOfferSendSuccessAction,
    P2pConnectionOutgoingRandomInitAction, P2pConnectionOutgoingReconnectAction,
    P2pConnectionOutgoingState, P2pConnectionOutgoingSuccessAction,
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

impl P2pConnectionOutgoingOfferSdpCreateErrorAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionOutgoingErrorAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionOutgoingErrorAction {
            peer_id: self.peer_id,
            error: P2pConnectionOutgoingError::SdpCreateError(self.error),
        });
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
        let P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
            P2pConnectionOutgoingState::OfferReady { opts, .. },
        )) = &peer.status else { return };
        let signaling_method = &opts.signaling;
        match signaling_method {
            webrtc::SignalingMethod::Http(_) | webrtc::SignalingMethod::Https(_) => {
                let Some(url) = signaling_method.http_url() else { return };
                service.http_signaling_request(url, self.offer);
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
            error: match self.error {
                P2pConnectionErrorResponse::Rejected(reason) => {
                    P2pConnectionOutgoingError::Rejected(reason)
                }
                P2pConnectionErrorResponse::InternalError => {
                    P2pConnectionOutgoingError::RemoteInternalError
                }
            },
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
        store
            .service()
            .set_answer(self.peer_id, self.answer.clone());
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
            error: P2pConnectionOutgoingError::FinalizeError(self.error),
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
