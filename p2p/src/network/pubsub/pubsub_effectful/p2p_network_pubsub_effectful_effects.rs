use openmina_core::bug_condition;

use libp2p_identity::{DecodingError, PublicKey};

use super::super::pb;

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

                for message in messages {
                    let result = if let Some(signature) = message.signature.clone() {
                        if let Ok(Some(pk)) = originator(&message) {
                            // the message is mutable only in this function
                            let mut message = message;
                            let Ok(data) = encode_without_signature_and_key(&mut message) else {
                                // should never happen;
                                // we just decode this message, so it should encode without error
                                bug_condition!("serialization error");
                                return;
                            };
                            // keep the binding immutable
                            let message = message;

                            if store.service().verify_publication(&pk, &data, &signature) {
                                store.dispatch(P2pNetworkPubsubAction::IncomingMessage {
                                    peer_id,
                                    message,
                                    seen_limit,
                                });
                                Ok(())
                            } else {
                                Err("invalid signature")
                            }
                        } else {
                            Err("message doesn't contain verifying key")
                        }
                    } else {
                        Err("message doesn't contain signature")
                    };

                    if let Err(error) = result {
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
                }
            }
        }
    }
}

pub fn originator(message: &pb::Message) -> Result<Option<PublicKey>, DecodingError> {
    message
        .key
        .as_deref()
        .or_else(|| message.from.as_deref().and_then(|f| f.get(2..)))
        .map(PublicKey::try_decode_protobuf)
        .transpose()
}

/// The reference to the message is mutable, but it is very important to keep the message the same
/// after this function is done.
pub fn encode_without_signature_and_key(
    message: &mut pb::Message,
) -> Result<Vec<u8>, prost::EncodeError> {
    // take the fields
    let signature = message.signature.take();
    let key = message.key.take();

    let mut data = vec![];
    let result = prost::Message::encode(&*message, &mut data);

    // put the fields back
    message.signature = signature;
    message.key = key;

    result.map(|()| data)
}
