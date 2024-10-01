use super::P2pChannelsSnarkEffectfulAction;
use crate::channels::{
    snark::{P2pChannelsSnarkAction, SnarkPropagationChannelMsg},
    ChannelId, MsgId, P2pChannelsService,
};
use redux::ActionMeta;

impl P2pChannelsSnarkEffectfulAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        match self {
            P2pChannelsSnarkEffectfulAction::Init { peer_id } => {
                store
                    .service()
                    .channel_open(peer_id, ChannelId::SnarkPropagation);
                store.dispatch(P2pChannelsSnarkAction::Pending { peer_id });
            }
            P2pChannelsSnarkEffectfulAction::RequestSend { peer_id, limit } => {
                let msg = SnarkPropagationChannelMsg::GetNext { limit };
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());
            }
            P2pChannelsSnarkEffectfulAction::ResponseSend {
                peer_id, snarks, ..
            } => {
                let msg = SnarkPropagationChannelMsg::WillSend {
                    count: snarks.len() as u8,
                };
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());

                for snark in snarks {
                    let msg = SnarkPropagationChannelMsg::Snark(snark);
                    store
                        .service()
                        .channel_send(peer_id, MsgId::first(), msg.into());
                }
            }
        }
    }
}
