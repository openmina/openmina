use redux::ActionMeta;

use crate::channels::{ChannelId, MsgId, P2pChannelsService};
#[cfg(feature = "p2p-libp2p")]
use crate::P2pNetworkPubsubAction;

use super::{P2pChannelsTransactionAction, TransactionPropagationChannelMsg};

impl P2pChannelsTransactionAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        match self {
            P2pChannelsTransactionAction::Init { peer_id } => {
                store
                    .service()
                    .channel_open(peer_id, ChannelId::TransactionPropagation);
                store.dispatch(P2pChannelsTransactionAction::Pending { peer_id });
            }
            P2pChannelsTransactionAction::Ready { .. } => {}
            P2pChannelsTransactionAction::RequestSend { peer_id, limit } => {
                let msg = TransactionPropagationChannelMsg::GetNext { limit };
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());
            }
            P2pChannelsTransactionAction::Received { .. } => {}
            P2pChannelsTransactionAction::ResponseSend {
                peer_id,
                transactions,
                ..
            } => {
                if transactions.is_empty() {
                    return;
                }

                let msg = TransactionPropagationChannelMsg::WillSend {
                    count: transactions.len() as u8,
                };
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());

                for tx in transactions {
                    let msg = TransactionPropagationChannelMsg::Transaction(tx);
                    store
                        .service()
                        .channel_send(peer_id, MsgId::first(), msg.into());
                }
            }
            P2pChannelsTransactionAction::Pending { .. } => {}
            P2pChannelsTransactionAction::PromiseReceived { .. } => {}
            P2pChannelsTransactionAction::RequestReceived { .. } => {}
            P2pChannelsTransactionAction::Libp2pReceived { .. } => {}
            #[cfg(not(feature = "p2p-libp2p"))]
            P2pChannelsTransactionAction::Libp2pBroadcast { .. } => {}
            #[cfg(feature = "p2p-libp2p")]
            P2pChannelsTransactionAction::Libp2pBroadcast { transaction, nonce } => {
                use mina_p2p_messages::{gossip::GossipNetMessageV2, v2};
                let message = v2::NetworkPoolTransactionPoolDiffVersionedStableV2(
                    std::iter::once(transaction).collect(),
                );
                let nonce = nonce.into();
                let message = Box::new(GossipNetMessageV2::TransactionPoolDiff { message, nonce });
                store.dispatch(P2pNetworkPubsubAction::Broadcast { message });
            }
        }
    }
}
