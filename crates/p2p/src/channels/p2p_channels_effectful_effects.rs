use mina_core::bug_condition;
use redux::ActionMeta;

use crate::webrtc::{Offer, P2pConnectionResponse};

use super::{
    signaling::{
        discovery::{P2pChannelsSignalingDiscoveryAction, SignalingDiscoveryChannelMsg},
        exchange::{P2pChannelsSignalingExchangeAction, SignalingExchangeChannelMsg},
    },
    ChannelMsg, MsgId, P2pChannelsEffectfulAction, P2pChannelsService,
};

impl P2pChannelsEffectfulAction {
    pub fn effects<Store, S>(self, meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        match self {
            P2pChannelsEffectfulAction::InitChannel {
                peer_id,
                id,
                on_success,
            } => {
                store.service().channel_open(peer_id, id);
                store.dispatch_callback(on_success, peer_id);
            }
            P2pChannelsEffectfulAction::MessageSend {
                peer_id,
                msg_id,
                msg,
            } => {
                store.service().channel_send(peer_id, msg_id, msg);
            }
            P2pChannelsEffectfulAction::SignalingDiscoveryAnswerDecrypt {
                peer_id,
                pub_key,
                answer,
            } => {
                match store
                    .service()
                    .decrypt::<P2pConnectionResponse>(&pub_key, &answer)
                {
                    Err(_) => {
                        store.dispatch(P2pChannelsSignalingDiscoveryAction::AnswerDecrypted {
                            peer_id,
                            answer: P2pConnectionResponse::SignalDecryptionFailed,
                        });
                    }
                    Ok(answer) => {
                        store.dispatch(P2pChannelsSignalingDiscoveryAction::AnswerDecrypted {
                            peer_id,
                            answer,
                        });
                    }
                }
            }
            P2pChannelsEffectfulAction::SignalingDiscoveryOfferEncryptAndSend {
                peer_id,
                pub_key,
                offer,
            } => match store.service().encrypt(&pub_key, offer.as_ref()) {
                Err(_) => {
                    // TODO: handle
                    mina_core::error!(
                        meta.time();
                        summary = "Failed to encrypt webrtc offer",
                        peer_id = peer_id.to_string()
                    );
                }
                Ok(offer) => {
                    let message = SignalingDiscoveryChannelMsg::DiscoveredAccept(offer);
                    store
                        .service()
                        .channel_send(peer_id, super::MsgId::first(), message.into());
                }
            },
            P2pChannelsEffectfulAction::SignalingExchangeOfferDecrypt {
                peer_id,
                pub_key,
                offer,
            } => {
                match store.service().decrypt::<Offer>(&pub_key, &offer) {
                    Err(_) => {
                        store.dispatch(P2pChannelsSignalingExchangeAction::OfferDecryptError {
                            peer_id,
                        });
                    }
                    Ok(offer) if offer.identity_pub_key != pub_key => {
                        // TODO(binier): propagate specific error.
                        // This is invalid behavior either from relayer or offerer.
                        store.dispatch(P2pChannelsSignalingExchangeAction::OfferDecryptError {
                            peer_id,
                        });
                    }
                    Ok(offer) => {
                        store.dispatch(P2pChannelsSignalingExchangeAction::OfferDecryptSuccess {
                            peer_id,
                            offer,
                        });
                    }
                }
            }
            P2pChannelsEffectfulAction::SignalingExchangeAnswerEncryptAndSend {
                peer_id,
                pub_key,
                answer,
            } => {
                let Some(answer) = answer else {
                    let message = SignalingExchangeChannelMsg::Answer(None);
                    store.service().channel_send(
                        peer_id,
                        MsgId::first(),
                        ChannelMsg::SignalingExchange(message),
                    );
                    return;
                };

                match store.service().encrypt(&pub_key, &answer) {
                    Err(_) => bug_condition!("Failed to encrypt webrtc answer. Shouldn't happen since we managed to decrypt sent offer."),
                    Ok(answer) => {
                        let message = SignalingExchangeChannelMsg::Answer(Some(answer));
                        store.service().channel_send(
                            peer_id,
                            MsgId::first(),
                            ChannelMsg::SignalingExchange(message),
                        );
                    }
                }
            }
        }
    }
}
