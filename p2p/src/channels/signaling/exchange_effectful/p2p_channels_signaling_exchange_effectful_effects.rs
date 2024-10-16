use openmina_core::bug_condition;
use redux::ActionMeta;

use crate::{
    channels::{
        signaling::exchange::{P2pChannelsSignalingExchangeAction, SignalingExchangeChannelMsg},
        ChannelId, MsgId,
    },
    webrtc::{EncryptedAnswer, Offer},
    P2pChannelsService,
};

use super::P2pChannelsSignalingExchangeEffectfulAction;

impl P2pChannelsSignalingExchangeEffectfulAction {
    pub fn effects<Store, S>(self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        match self {
            P2pChannelsSignalingExchangeEffectfulAction::Init { peer_id } => {
                store
                    .service()
                    .channel_open(peer_id, ChannelId::SignalingExchange);
                store.dispatch(P2pChannelsSignalingExchangeAction::Pending { peer_id });
            }
            P2pChannelsSignalingExchangeEffectfulAction::MessageSend { peer_id, message } => {
                message_send(store.service(), peer_id, message);
            }
            P2pChannelsSignalingExchangeEffectfulAction::OfferDecrypt {
                peer_id,
                pub_key,
                offer,
            } => {
                match store.service().decrypt::<Offer>(&pub_key, &offer) {
                    Err(()) => {
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
            P2pChannelsSignalingExchangeEffectfulAction::AnswerEncryptAndSend {
                peer_id,
                pub_key,
                answer,
            } => {
                let answer = match answer {
                    None => {
                        answer_message_send(store.service(), peer_id, None);
                        return;
                    }
                    Some(v) => v,
                };
                match store.service().encrypt(&pub_key, &answer) {
                    Err(()) => bug_condition!("Failed to encrypt webrtc answer. Shouldn't happen since we managed to decrypt sent offer."),
                    Ok(answer) => {
                        answer_message_send(store.service(), peer_id, Some(answer));
                    }
                }
            }
        }
    }
}

fn answer_message_send<S>(service: &mut S, peer_id: crate::PeerId, answer: Option<EncryptedAnswer>)
where
    S: P2pChannelsService,
{
    message_send(
        service,
        peer_id,
        SignalingExchangeChannelMsg::Answer(answer),
    )
}

fn message_send<S>(service: &mut S, peer_id: crate::PeerId, message: SignalingExchangeChannelMsg)
where
    S: P2pChannelsService,
{
    service.channel_send(peer_id, MsgId::first(), message.into())
}
