use redux::ActionMeta;

use crate::connection::webrtc::P2pConnectionWebRTCService;
use crate::disconnection::{P2pDisconnectionInitAction, P2pDisconnectionReason};
use crate::peer::P2pPeerReadyAction;
use crate::webrtc;

use super::*;

impl P2pConnectionWebRTCIncomingInitAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionWebRTCService,
        P2pConnectionWebRTCIncomingAnswerSdpCreatePendingAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.opts.peer_id;
        store.service().incoming_init(peer_id, self.opts.offer);
        store.dispatch(P2pConnectionWebRTCIncomingAnswerSdpCreatePendingAction { peer_id });
    }
}

impl P2pConnectionWebRTCIncomingAnswerSdpCreateErrorAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionWebRTCService,
        P2pConnectionWebRTCIncomingErrorAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionWebRTCIncomingErrorAction {
            peer_id: self.peer_id,
            error: P2pConnectionWebRTCIncomingError::SdpCreateError(self.error),
        });
    }
}

impl P2pConnectionWebRTCIncomingAnswerSdpCreateSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionWebRTCService,
        P2pConnectionWebRTCIncomingAnswerReadyAction: redux::EnablingCondition<S>,
    {
        let answer = webrtc::Answer {
            sdp: self.sdp,
            identity_pub_key: store.state().config.identity_pub_key.clone(),
            target_peer_id: self.peer_id,
        };
        store.dispatch(P2pConnectionWebRTCIncomingAnswerReadyAction {
            peer_id: self.peer_id,
            answer,
        });
    }
}

impl P2pConnectionWebRTCIncomingAnswerReadyAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionWebRTCService,
    {
        store.service().set_answer(self.peer_id, self.answer);
    }
}

impl P2pConnectionWebRTCIncomingAnswerSendSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionWebRTCService,
        P2pConnectionWebRTCIncomingFinalizePendingAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionWebRTCIncomingFinalizePendingAction {
            peer_id: self.peer_id,
        });
    }
}

impl P2pConnectionWebRTCIncomingFinalizeErrorAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionWebRTCService,
        P2pConnectionWebRTCIncomingErrorAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionWebRTCIncomingErrorAction {
            peer_id: self.peer_id,
            error: P2pConnectionWebRTCIncomingError::FinalizeError(self.error),
        });
    }
}

impl P2pConnectionWebRTCIncomingFinalizeSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionWebRTCService,
        P2pConnectionWebRTCIncomingSuccessAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionWebRTCIncomingSuccessAction {
            peer_id: self.peer_id,
        });
    }
}

impl P2pConnectionWebRTCIncomingTimeoutAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionWebRTCService,
        P2pConnectionWebRTCIncomingErrorAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionWebRTCIncomingErrorAction {
            peer_id: self.peer_id,
            error: P2pConnectionWebRTCIncomingError::Timeout,
        });
    }
}

impl P2pConnectionWebRTCIncomingSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionWebRTCService,
        P2pPeerReadyAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        store.dispatch(P2pPeerReadyAction {
            peer_id,
            incoming: true,
        });
    }
}

impl P2pConnectionWebRTCIncomingLibp2pReceivedAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionWebRTCService,
        P2pPeerReadyAction: redux::EnablingCondition<S>,
        P2pDisconnectionInitAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.peer_id;
        if let Err(err) = store.state().libp2p_incoming_accept(peer_id) {
            store.dispatch(P2pDisconnectionInitAction {
                peer_id,
                reason: P2pDisconnectionReason::Libp2pIncomingRejected(err),
            });
        } else {
            store.dispatch(P2pPeerReadyAction {
                peer_id,
                incoming: true,
            });
        }
    }
}
