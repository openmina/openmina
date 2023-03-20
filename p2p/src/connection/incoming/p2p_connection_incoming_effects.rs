use redux::ActionMeta;

use crate::connection::P2pConnectionState;
use crate::{connection::P2pConnectionService, webrtc};
use crate::{P2pPeerReadyAction, P2pPeerStatus};

use super::{
    IncomingSignalingMethod, P2pConnectionIncomingAnswerReadyAction,
    P2pConnectionIncomingAnswerSdpCreatePendingAction,
    P2pConnectionIncomingAnswerSdpCreateSuccessAction,
    P2pConnectionIncomingAnswerSendSuccessAction, P2pConnectionIncomingErrorAction,
    P2pConnectionIncomingFinalizeErrorAction, P2pConnectionIncomingFinalizePendingAction,
    P2pConnectionIncomingFinalizeSuccessAction, P2pConnectionIncomingInitAction,
    P2pConnectionIncomingState, P2pConnectionIncomingSuccessAction,
};

impl P2pConnectionIncomingInitAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionIncomingAnswerSdpCreatePendingAction: redux::EnablingCondition<S>,
    {
        let peer_id = self.offer.target_peer_id;
        store.service().incoming_init(peer_id, self.offer);
        store.dispatch(P2pConnectionIncomingAnswerSdpCreatePendingAction { peer_id });
    }
}

impl P2pConnectionIncomingAnswerSdpCreateSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionIncomingAnswerReadyAction: redux::EnablingCondition<S>,
    {
        let answer = webrtc::Answer {
            sdp: self.sdp,
            identity_pub_key: store.state().config.identity_pub_key.clone(),
            target_peer_id: self.peer_id,
        };
        store.dispatch(P2pConnectionIncomingAnswerReadyAction {
            peer_id: self.peer_id,
            answer,
        });
    }
}

impl P2pConnectionIncomingAnswerReadyAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionIncomingAnswerSendSuccessAction: redux::EnablingCondition<S>,
    {
        let (state, service) = store.state_and_service();
        let Some(peer) = state.peers.get(&self.peer_id) else { return };
        let P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
            P2pConnectionIncomingState::AnswerReady { signaling, .. },
        )) = &peer.status else { return };
        // TODO(binier): use dial_opts from outgoing state instead.
        service.set_answer(self.peer_id, self.answer.clone());
        match signaling {
            IncomingSignalingMethod::Http => {
                service.http_signaling_respond(self.answer);
            }
        }
        store.dispatch(P2pConnectionIncomingAnswerSendSuccessAction {
            peer_id: self.peer_id,
        });
    }
}

impl P2pConnectionIncomingAnswerSendSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionIncomingFinalizePendingAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionIncomingFinalizePendingAction {
            peer_id: self.peer_id,
        });
    }
}

impl P2pConnectionIncomingFinalizeErrorAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionIncomingErrorAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionIncomingErrorAction {
            peer_id: self.peer_id,
            error: self.error,
        });
    }
}

impl P2pConnectionIncomingFinalizeSuccessAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
        P2pConnectionIncomingSuccessAction: redux::EnablingCondition<S>,
    {
        store.dispatch(P2pConnectionIncomingSuccessAction {
            peer_id: self.peer_id,
        });
    }
}

impl P2pConnectionIncomingSuccessAction {
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
