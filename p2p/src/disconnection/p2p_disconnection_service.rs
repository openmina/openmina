use crate::PeerId;

pub trait P2pDisconnectionService: redux::Service {
    fn disconnect(&mut self, peer_id: PeerId);
}
