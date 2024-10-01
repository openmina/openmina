use super::P2pChannelsTransactionEffectfulAction;
use crate::channels::{
    transaction::{P2pChannelsTransactionAction, TransactionPropagationChannelMsg},
    ChannelId, MsgId, P2pChannelsService,
};
use redux::ActionMeta;

impl P2pChannelsTransactionEffectfulAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pChannelsService,
    {
        match self {
            P2pChannelsTransactionEffectfulAction::Init { peer_id } => {
                store
                    .service()
                    .channel_open(peer_id, ChannelId::TransactionPropagation);
                store.dispatch(P2pChannelsTransactionAction::Pending { peer_id });
            }
            P2pChannelsTransactionEffectfulAction::RequestSend { peer_id, limit } => {
                let msg = TransactionPropagationChannelMsg::GetNext { limit };
                store
                    .service()
                    .channel_send(peer_id, MsgId::first(), msg.into());
            }
            P2pChannelsTransactionEffectfulAction::ResponseSend {
                peer_id,
                transactions,
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
        }
    }
}
