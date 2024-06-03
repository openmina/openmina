use mina_p2p_messages::{gossip::GossipNetMessageV2, v2};
use redux::ActionMeta;

use crate::{
    channels::{ChannelId, MsgId, P2pChannelsService},
    P2pNetworkPubsubAction,
};

use super::{P2pChannelsSnarkAction, SnarkPropagationChannelMsg};

impl P2pChannelsSnarkAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        match self {
            P2pChannelsSnarkAction::Init { peer_id } => {
                store
                    .service()
                    .channel_open(peer_id, ChannelId::SnarkPropagation);
                store.dispatch(P2pChannelsSnarkAction::Pending { peer_id });
            }
            P2pChannelsSnarkAction::Ready { .. } => {}
            P2pChannelsSnarkAction::RequestSend { peer_id, limit } => {
                let msg = SnarkPropagationChannelMsg::GetNext { limit };
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());
            }
            P2pChannelsSnarkAction::Received { .. } => {}
            P2pChannelsSnarkAction::ResponseSend {
                peer_id, snarks, ..
            } => {
                if snarks.is_empty() {
                    return;
                }

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
            P2pChannelsSnarkAction::Libp2pBroadcast { snark, nonce } => {
                let message = Box::new((snark.statement(), (&snark).into()));
                let message = v2::NetworkPoolSnarkPoolDiffVersionedStableV2::AddSolvedWork(message);
                let nonce = nonce.into();
                let message = Box::new(GossipNetMessageV2::SnarkPoolDiff { message, nonce });
                store.dispatch(P2pNetworkPubsubAction::Broadcast { message });
            }
            P2pChannelsSnarkAction::Pending { .. } => {}
            P2pChannelsSnarkAction::PromiseReceived { .. } => {}
            P2pChannelsSnarkAction::RequestReceived { .. } => {}
            P2pChannelsSnarkAction::Libp2pReceived { .. } => {}
        }
    }
}
