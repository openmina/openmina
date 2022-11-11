use super::PubsubTopic;

pub trait P2pPubsubService: redux::Service {
    fn publish(&mut self, topic: PubsubTopic, bytes: Vec<u8>);
}
