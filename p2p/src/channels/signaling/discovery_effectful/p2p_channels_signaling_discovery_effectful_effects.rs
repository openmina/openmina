use redux::ActionMeta;

use crate::{
    channels::{
        signaling::discovery::{P2pChannelsSignalingDiscoveryAction, SignalingDiscoveryChannelMsg},
        ChannelId, MsgId,
    },
    connection::P2pConnectionResponse,
    P2pChannelsService,
};

use super::P2pChannelsSignalingDiscoveryEffectfulAction;

impl P2pChannelsSignalingDiscoveryEffectfulAction {
    pub fn effects<Store, S>(self, _meta: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        match self {
            P2pChannelsSignalingDiscoveryEffectfulAction::Init { peer_id } => {
                store
                    .service()
                    .channel_open(peer_id, ChannelId::SignalingDiscovery);
                store.dispatch(P2pChannelsSignalingDiscoveryAction::Pending { peer_id });
            }
            P2pChannelsSignalingDiscoveryEffectfulAction::MessageSend { peer_id, message } => {
                message_send(store.service(), peer_id, message);
            }
            P2pChannelsSignalingDiscoveryEffectfulAction::OfferEncryptAndSend {
                peer_id,
                pub_key,
                offer,
            } => match store.service().encrypt(&pub_key, &*offer) {
                Err(()) => todo!("Failed to encrypt webrtc offer. Handle it."),
                Ok(offer) => {
                    let message = SignalingDiscoveryChannelMsg::DiscoveredAccept(offer);
                    message_send(store.service(), peer_id, message);
                }
            },
            P2pChannelsSignalingDiscoveryEffectfulAction::AnswerDecrypt {
                peer_id,
                pub_key,
                answer,
            } => {
                match store
                    .service()
                    .decrypt::<P2pConnectionResponse>(&pub_key, &answer)
                {
                    Err(()) => {
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
        }
    }
}

fn message_send<S>(service: &mut S, peer_id: crate::PeerId, message: SignalingDiscoveryChannelMsg)
where
    S: P2pChannelsService,
{
    service.channel_send(peer_id, MsgId::first(), message.into())
}
