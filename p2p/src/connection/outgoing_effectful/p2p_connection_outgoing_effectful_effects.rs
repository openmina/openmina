use openmina_core::bug_condition;
use redux::ActionMeta;

use crate::{
    connection::{
        outgoing::{P2pConnectionOutgoingAction, P2pConnectionOutgoingError},
        P2pConnectionService,
    },
    webrtc,
};

use super::P2pConnectionOutgoingEffectfulAction;

impl P2pConnectionOutgoingEffectfulAction {
    pub fn effects<Store, S>(self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
    {
        match self {
            P2pConnectionOutgoingEffectfulAction::RandomInit { peers } => {
                let picked_peer = store.service().random_pick(&peers);
                if let Some(picked_peer) = picked_peer {
                    store.dispatch(P2pConnectionOutgoingAction::Reconnect {
                        opts: picked_peer,
                        rpc_id: None,
                    });
                } else {
                    bug_condition!("Picked peer is None");
                }
            }
            P2pConnectionOutgoingEffectfulAction::Init { opts, .. } => {
                let peer_id = *opts.peer_id();
                store.service().outgoing_init(opts);
                store.dispatch(P2pConnectionOutgoingAction::OfferSdpCreatePending { peer_id });
            }
            P2pConnectionOutgoingEffectfulAction::OfferSend {
                peer_id,
                offer,
                signaling_method,
            } => {
                match signaling_method {
                    webrtc::SignalingMethod::Http(_)
                    | webrtc::SignalingMethod::Https(_)
                    | webrtc::SignalingMethod::HttpsProxy(_, _) => {
                        let Some(url) = signaling_method.http_url() else {
                            return;
                        };
                        store.service().http_signaling_request(url, *offer);
                    }
                    webrtc::SignalingMethod::P2p { .. } => {
                        bug_condition!("`P2pConnectionOutgoingEffectfulAction::OfferSend` shouldn't be called for `webrtc::SignalingMethod::P2p`");
                        return;
                    }
                }
                store.dispatch(P2pConnectionOutgoingAction::OfferSendSuccess { peer_id });
            }
            P2pConnectionOutgoingEffectfulAction::AnswerSet { peer_id, answer } => {
                store.service().set_answer(peer_id, *answer);
                store.dispatch(P2pConnectionOutgoingAction::FinalizePending { peer_id });
            }
            P2pConnectionOutgoingEffectfulAction::ConnectionAuthorizationEncryptAndSend {
                peer_id,
                other_pub_key,
                auth,
            } => {
                store
                    .service()
                    .auth_encrypt_and_send(peer_id, &other_pub_key, auth);
            }
            P2pConnectionOutgoingEffectfulAction::ConnectionAuthorizationDecryptAndCheck {
                peer_id,
                other_pub_key,
                expected_auth,
                auth,
            } => {
                if store
                    .service()
                    .auth_decrypt(&other_pub_key, auth)
                    .is_some_and(|remote_auth| remote_auth == expected_auth)
                {
                    store.dispatch(P2pConnectionOutgoingAction::Success { peer_id });
                } else {
                    store.dispatch(P2pConnectionOutgoingAction::Error {
                        peer_id,
                        error: P2pConnectionOutgoingError::ConnectionAuthError,
                    });
                }
            }
        }
    }
}
