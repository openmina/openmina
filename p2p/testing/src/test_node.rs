use std::net::IpAddr;

use libp2p::multiaddr::multiaddr;
use libp2p::Multiaddr;
use p2p::{
    connection::outgoing::{P2pConnectionOutgoingInitLibp2pOpts, P2pConnectionOutgoingInitOpts},
    PeerId,
};

pub trait TestNode {
    fn peer_id(&self) -> PeerId;

    fn libp2p_port(&self) -> u16;

    fn rust_dial_opts(&self, host: IpAddr) -> P2pConnectionOutgoingInitOpts {
        P2pConnectionOutgoingInitOpts::LibP2P(P2pConnectionOutgoingInitLibp2pOpts {
            peer_id: self.peer_id(),
            host: host.into(),
            port: self.libp2p_port(),
        })
    }

    fn libp2p_dial_opts(&self, host: IpAddr) -> Multiaddr {
        let peer_id: libp2p::PeerId = self.peer_id().try_into().unwrap();

        match host {
            IpAddr::V4(ip) => {
                multiaddr!(Ip4(ip), Tcp(self.libp2p_port()), P2p(peer_id))
            }
            IpAddr::V6(ip) => {
                multiaddr!(Ip6(ip), Tcp(self.libp2p_port()), P2p(peer_id))
            }
        }
    }
}
