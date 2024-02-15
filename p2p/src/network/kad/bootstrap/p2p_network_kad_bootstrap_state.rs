use std::{
    collections::{BTreeSet, VecDeque},
    net::SocketAddr,
};

use multiaddr::Multiaddr;
use redux::Timestamp;
use serde::{Deserialize, Serialize};

use crate::PeerId;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct P2pNetworkKadBootstrapState {
    /// Key that is used to request closest peers. Usually self peer_id.
    pub key: PeerId,
    /// Peers that already been contacted (successfully or not) for FIND_NODE.
    pub processed_peers: BTreeSet<PeerId>,
    /// Ongoing FIND_NODE requests.
    pub requests: Vec<P2pNetworkKadBoostrapRequestState>,
    /// Queue of peers for FIND_NODE query.
    pub queue: VecDeque<(PeerId, Vec<Multiaddr>)>,
    /// Number of peers received.
    pub discovered_peers_num: usize,
}

impl P2pNetworkKadBootstrapState {
    pub fn new<'a, I>(key: PeerId, peers: I) -> Self
    where
        I: 'a + IntoIterator<Item = (&'a PeerId, &'a Vec<Multiaddr>)>,
    {
        let queue = peers
            .into_iter()
            .filter_map(|(peer_id, maddrs)| {
                if maddrs.is_empty() {
                    None
                } else {
                    Some((peer_id.clone(), maddrs.clone()))
                }
            })
            .collect();
        P2pNetworkKadBootstrapState {
            key,
            processed_peers: BTreeSet::new(),
            requests: Vec::with_capacity(3),
            queue,
            discovered_peers_num: 0,
        }
    }

    pub fn request(
        &self,
        addr: &SocketAddr,
    ) -> Option<(usize, &P2pNetworkKadBoostrapRequestState)> {
        self.requests
            .iter()
            .enumerate()
            .find(|(_, req)| addr == &req.addr)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct P2pNetworkKadBoostrapRequestState {
    /// Peer id
    pub peer_id: PeerId,
    /// Address that is used for the current connection.
    // TODO: generalize to DNS addrs
    pub addr: SocketAddr,
    /// When connection to the peer was initiated.
    pub time: Timestamp,
    /// Addresses yet to be used, if current connection will fail.
    // TODO: use Multiaddr
    pub addrs_to_use: Vec<SocketAddr>,
}
