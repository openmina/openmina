use openmina_core::bug_condition;

use libp2p_identity::PublicKey;

use crate::{
    P2pCryptoService, P2pNetworkConnectionError, P2pNetworkPubsubAction, P2pNetworkSchedulerAction,
};

use super::P2pNetworkPubsubEffectfulAction;

impl P2pNetworkPubsubEffectfulAction {
    pub fn effects<Store, S>(self, _meta: &redux::ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pCryptoService,
    {
        let state = &store.state().network.scheduler.broadcast_state;

        match self {
            P2pNetworkPubsubEffectfulAction::Sign { author, topic } => {
                if let Some(to_sign) = state.to_sign.front() {
                    let mut publication = vec![];
                    if prost::Message::encode(to_sign, &mut publication).is_err() {
                        store.dispatch(P2pNetworkPubsubAction::SignError { author, topic });
                    } else {
                        let signature = store.service().sign_publication(&publication).into();
                        store.dispatch(P2pNetworkPubsubAction::BroadcastSigned { signature });
                    }
                }
            }
            P2pNetworkPubsubEffectfulAction::IncomingData {
                peer_id,
                seen_limit,
            } => {
                let Some(state) = state.clients.get(&peer_id) else {
                    // TODO: investigate, cannot reproduce this
                    // bug_condition!("{:?} not found in state.clients", peer_id);
                    return;
                };
                let messages = state.incoming_messages.clone();

                for mut message in messages {
                    let mut error = None;

                    let originator =
                        match message.key.as_deref().map(PublicKey::try_decode_protobuf) {
                            Some(Ok(v)) => Some(v),
                            _ => PublicKey::try_decode_protobuf(&message.from()[2..]).ok(),
                        };

                    if let Some(signature) = message.signature.take() {
                        if let Some(pk) = originator {
                            message.key = None;
                            let mut data = vec![];

                            if prost::Message::encode(&message, &mut data).is_err() {
                                // should never happen;
                                // we just decode this message, so it should encode without error
                                bug_condition!("serialization error");
                                return;
                            };

                            if !store.service().verify_publication(&pk, &data, &signature) {
                                error = Some("invalid signature");
                            }
                        } else {
                            error = Some("message doesn't contain verifying key");
                        }
                    } else {
                        error = Some("message doesn't contain signature");
                    }

                    if let Some(error) = error {
                        let Some((addr, _)) = store.state().network.scheduler.find_peer(&peer_id)
                        else {
                            bug_condition!("{:?} not found in scheduler state", peer_id);
                            return;
                        };

                        store.dispatch(P2pNetworkSchedulerAction::Error {
                            addr: *addr,
                            error: P2pNetworkConnectionError::PubSubError(error.to_string()),
                        });

                        return;
                    }

                    store.dispatch(P2pNetworkPubsubAction::IncomingMessage {
                        peer_id,
                        message,
                        seen_limit,
                    });
                }
            }
        }
    }
}
