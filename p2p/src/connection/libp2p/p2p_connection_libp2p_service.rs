use crate::{libp2p::P2pLibP2pAddr, PeerId};

pub trait P2pConnectionLibP2pService: redux::Service {
    /// Initiates an outgoing connection.
    fn outgoing_init(&mut self, peer_id: PeerId, addrs: Vec<P2pLibP2pAddr>);

    fn start_discovery(&mut self, peers: Vec<(PeerId, Vec<P2pLibP2pAddr>)>);

    fn find_random_peer(&mut self);
}
