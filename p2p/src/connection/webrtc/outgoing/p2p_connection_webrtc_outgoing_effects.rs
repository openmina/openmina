use openmina_core::error;
use redux::ActionMeta;

use crate::connection::P2pConnectionState;
use crate::connection::webrtc::P2pConnectionWebRTCErrorResponse;
use crate::connection::webrtc::P2pConnectionWebRTCService;
use crate::peer::P2pPeerReadyAction;
use crate::webrtc::Host;
use crate::P2pPeerStatus;
use crate::webrtc;

use super::*;

// impl P2pConnectionWebRTCOutgoingRandomInitAction {
//     pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
//     where
//         Store: crate::P2pStore<S>,
//         Store::Service: P2pConnectionWebRTCService,
//         P2pConnectionWebRTCOutgoingInitAction: redux::EnablingCondition<S>,
//     {
//         // let peers = store.state().initial_unused_peers();
//         // let picked_peer = store.service().random_pick(&peers);
//         // store.dispatch(P2pConnectionWebRTCOutgoingInitAction {

//         //     rpc_id: None,
//         // });
//         todo!();
//     }
// }

impl P2pConnectionWebRTCOutgoingInitAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionWebRTCService,
        P2pConnectionWebRTCOutgoingOfferSdpCreatePendingAction: redux::EnablingCondition<S>,
        P2pConnectionWebRTCOutgoingFinalizePendingAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        let Some(peer) = store.state().get_webrtc_peer(&peer_id) else {
            error!(meta.time(); "incorrect state for peer {}: {:?}", peer_id, store.state().peers.get(&peer_id));
            return;
        };
        let Some(dial_opts) = peer.dial_opts.clone() else {
            error!(meta.time(); "no dial options for for peer {}", peer_id);
            return;
        };
        store.service().outgoing_init(peer_id, dial_opts);
        if !store.dispatch(P2pConnectionWebRTCOutgoingFinalizePendingAction { peer_id }) {
            store.dispatch(P2pConnectionWebRTCOutgoingOfferSdpCreatePendingAction { peer_id });
        }
    }
}

impl P2pConnectionWebRTCOutgoingOfferSdpCreateErrorAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionWebRTCService,
        P2pConnectionWebRTCOutgoingErrorAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionWebRTCOutgoingErrorAction {
            peer_id: self.peer_id,
            error: P2pConnectionWebRTCOutgoingError::SdpCreateError(self.error),
        });
    }
}

impl P2pConnectionWebRTCOutgoingOfferSdpCreateSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionWebRTCService,
        P2pConnectionWebRTCOutgoingOfferReadyAction: redux::EnablingCondition<S>,
    {
        let offer = webrtc::Offer {
            sdp: self.sdp,
            identity_pub_key: store.state().config.identity_pub_key.clone(),
            target_peer_id: self.peer_id,
            // TODO(vlad9486): put real address
            host: Host::Ipv4([127, 0, 0, 1].into()),
            listen_port: store.state().config.listen_port,
        };
        store.dispatch(P2pConnectionWebRTCOutgoingOfferReadyAction {
            peer_id: self.peer_id,
            offer,
        });
    }
}

impl P2pConnectionWebRTCOutgoingOfferReadyAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionWebRTCService,
        P2pConnectionWebRTCOutgoingOfferSendSuccessAction: redux::EnablingCondition<S>,
    {
        let (state, service) = store.state_and_service();
        let Some(peer) = state.get_webrtc_peer(&self.peer_id) else {
            return;
        };
        let P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
            P2pConnectionWebRTCOutgoingState::OfferReady { .. },
        )) = &peer.status
        else {
            return;
        };

        let Some(url) = peer.dial_opts.as_ref().and_then(|sm| sm.http_url()) else {
            return;
        };
        service.http_signaling_request(url, self.offer);

        store.dispatch(P2pConnectionWebRTCOutgoingOfferSendSuccessAction {
            peer_id: self.peer_id,
        });
    }
}

impl P2pConnectionWebRTCOutgoingOfferSendSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionWebRTCService,
        P2pConnectionWebRTCOutgoingAnswerRecvPendingAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionWebRTCOutgoingAnswerRecvPendingAction {
            peer_id: self.peer_id,
        });
    }
}

impl P2pConnectionWebRTCOutgoingAnswerRecvErrorAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionWebRTCService,
        P2pConnectionWebRTCOutgoingErrorAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionWebRTCOutgoingErrorAction {
            peer_id: self.peer_id,
            error: match self.error {
                P2pConnectionWebRTCErrorResponse::Rejected(reason) => {
                    P2pConnectionWebRTCOutgoingError::Rejected(reason)
                }
                P2pConnectionWebRTCErrorResponse::InternalError => {
                    P2pConnectionWebRTCOutgoingError::RemoteInternalError
                }
            },
        });
    }
}

impl P2pConnectionWebRTCOutgoingAnswerRecvSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionWebRTCService,
        P2pConnectionWebRTCOutgoingFinalizePendingAction: redux::EnablingCondition<S>,
    {
        store
            .service()
            .set_answer(self.peer_id, self.answer.clone());
        store.dispatch(P2pConnectionWebRTCOutgoingFinalizePendingAction {
            peer_id: self.peer_id,
        });
    }
}

impl P2pConnectionWebRTCOutgoingFinalizeErrorAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionWebRTCService,
        P2pConnectionWebRTCOutgoingErrorAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionWebRTCOutgoingErrorAction {
            peer_id: self.peer_id,
            error: P2pConnectionWebRTCOutgoingError::FinalizeError(self.error),
        });
    }
}

impl P2pConnectionWebRTCOutgoingFinalizeSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionWebRTCService,
        P2pConnectionWebRTCOutgoingSuccessAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionWebRTCOutgoingSuccessAction {
            peer_id: self.peer_id,
        });
    }
}

impl P2pConnectionWebRTCOutgoingTimeoutAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionWebRTCService,
        P2pConnectionWebRTCOutgoingErrorAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionWebRTCOutgoingErrorAction {
            peer_id: self.peer_id,
            error: P2pConnectionWebRTCOutgoingError::Timeout,
        });
    }
}

impl P2pConnectionWebRTCOutgoingSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionWebRTCService,
        P2pPeerReadyAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        store.dispatch(P2pPeerReadyAction {
            peer_id,
            incoming: false,
        });
    }
}
