use redux::ActionMeta;

use super::P2pConnectionIncomingEffectfulAction;
use crate::connection::{
    incoming::{P2pConnectionIncomingAction, P2pConnectionIncomingError},
    P2pConnectionService,
};

impl P2pConnectionIncomingEffectfulAction {
    pub fn effects<Store, S>(self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pConnectionService,
    {
        match self {
            P2pConnectionIncomingEffectfulAction::Init { opts } => {
                let peer_id = opts.peer_id;
                store.service().incoming_init(peer_id, *opts.offer);
                store.dispatch(P2pConnectionIncomingAction::AnswerSdpCreatePending { peer_id });
            }
            P2pConnectionIncomingEffectfulAction::ConnectionAuthorizationEncryptAndSend {
                peer_id,
                other_pub_key,
                auth,
            } => {
                store
                    .service()
                    .auth_encrypt_and_send(peer_id, &other_pub_key, auth);
            }
            P2pConnectionIncomingEffectfulAction::ConnectionAuthorizationDecryptAndCheck {
                peer_id,
                other_pub_key,
                expected_auth,
                auth,
            } => {
                if store
                    .service()
                    .auth_decrypt(&other_pub_key, auth)
                    .map_or(false, |remote_auth| remote_auth == expected_auth)
                {
                    store.dispatch(P2pConnectionIncomingAction::Success { peer_id });
                } else {
                    store.dispatch(P2pConnectionIncomingAction::Error {
                        peer_id,
                        error: P2pConnectionIncomingError::ConnectionAuthError,
                    });
                }
            }
        }
    }
}
