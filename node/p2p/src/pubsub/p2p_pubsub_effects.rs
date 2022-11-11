use redux::ActionMeta;

use super::{
    GossipNetMessageV1, P2pPubsubBytesPublishAction, P2pPubsubBytesReceivedAction,
    P2pPubsubMessagePublishAction, P2pPubsubMessageReceivedAction, P2pPubsubService,
};

impl P2pPubsubMessagePublishAction {
    pub fn effects<Store, S>(&self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        P2pPubsubBytesPublishAction: redux::EnablingCondition<S>,
    {
        let mut encoded = vec![0; 8];
        match binprot::BinProtWrite::binprot_write(&self.message, &mut encoded) {
            Ok(_) => {}
            Err(err) => {
                // TODO(binier)
                return;
                // log::error!("Failed to encode GossipSub Message: {:?}", err);
                // panic!("{}", err);
            }
        }
        let msg_len = (encoded.len() as u64 - 8).to_le_bytes();
        encoded[..8].clone_from_slice(&msg_len);

        store.dispatch(P2pPubsubBytesPublishAction {
            topic: self.topic.clone(),
            bytes: encoded,
            rpc_id: self.rpc_id,
        });
    }
}

impl P2pPubsubBytesPublishAction {
    pub fn effects<Store, S>(self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        Store::Service: P2pPubsubService,
        P2pPubsubBytesPublishAction: redux::EnablingCondition<S>,
    {
        store.service().publish(self.topic, self.bytes);
    }
}

impl P2pPubsubBytesReceivedAction {
    pub fn effects<Store, S>(&self, _: &ActionMeta, store: &mut Store)
    where
        Store: crate::P2pStore<S>,
        P2pPubsubMessageReceivedAction: redux::EnablingCondition<S>,
    {
        let len = u64::from_le_bytes(self.bytes[0..8].try_into().unwrap());
        let data = &self.bytes[8..];
        assert_eq!(len, data.len() as u64);
        let message: GossipNetMessageV1 = binprot::BinProtRead::binprot_read(&mut &*data).unwrap();

        store.dispatch(P2pPubsubMessageReceivedAction {
            author: self.author,
            sender: self.sender,
            topic: self.topic.clone(),
            message,
        });
    }
}
