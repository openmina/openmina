use std::net::IpAddr;

use p2p::{
    connection::outgoing::{P2pConnectionOutgoingInitLibp2pOpts, P2pConnectionOutgoingInitOpts},
    PeerId,
};

pub trait TestNode {
    fn peer_id(&self) -> PeerId;

    fn libp2p_port(&self) -> u16;

    fn dial_opts(&self, host: IpAddr) -> P2pConnectionOutgoingInitOpts {
        P2pConnectionOutgoingInitOpts::LibP2P(P2pConnectionOutgoingInitLibp2pOpts {
            peer_id: self.peer_id(),
            host: host.into(),
            port: self.libp2p_port(),
        })
    }
}
