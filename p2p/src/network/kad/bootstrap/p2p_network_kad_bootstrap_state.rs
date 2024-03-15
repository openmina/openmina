use std::{
    collections::{BTreeMap, BTreeSet},
    net::SocketAddr,
};

use redux::Timestamp;
use serde::{Deserialize, Serialize};

use crate::{
    connection::outgoing::P2pConnectionOutgoingInitOpts, P2pNetworkKadKey,
    P2pNetworkKadLatestRequestPeers, PeerId,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct P2pNetworkKadBootstrapState {
    /// Key that is used to request closest peers. Usually self peer_id.
    pub key: PeerId,
    /// Kademlia key, `sha265(self.key)`.
    pub kademlia_key: P2pNetworkKadKey,
    /// Peers that already been contacted (successfully or not) for FIND_NODE.
    pub processed_peers: BTreeSet<PeerId>,
    /// Ongoing FIND_NODE requests.
    ///
    /// TODO: replace with something more lightweight.
    pub requests: BTreeMap<PeerId, P2pNetworkKadBoostrapRequestState>,
    ///
    pub successfull_requests: usize,
    /// Bootstrap requests statistics.
    pub stats: P2pNetworkKadBootstrapStats,
}

impl P2pNetworkKadBootstrapState {
    pub fn new(key: PeerId) -> Self {
        P2pNetworkKadBootstrapState {
            key,
            kademlia_key: key.into(),
            processed_peers: BTreeSet::new(),
            requests: BTreeMap::new(),
            successfull_requests: 0,
            stats: Default::default(),
        }
    }

    pub fn request(&self, peer_id: &PeerId) -> Option<&P2pNetworkKadBoostrapRequestState> {
        self.requests.get(peer_id)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct P2pNetworkKadBoostrapRequestState {
    /// Address that is used for the current connection.
    // TODO: generalize to DNS addrs
    pub addr: SocketAddr,
    /// When connection to the peer was initiated.
    pub time: Timestamp,
    /// Addresses yet to be used, if current connection will fail.
    // TODO: use Multiaddr
    pub addrs_to_use: Vec<SocketAddr>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct P2pNetworkKadBootstrapStats {
    pub requests: Vec<P2pNetworkKadBootstrapRequestStat>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum P2pNetworkKadBootstrapRequestStat {
    Ongoing(P2pNetworkKadBootstrapOngoingRequest),
    Successfull(P2pNetworkKadBootstrapSuccessfullRequest),
    Failed(P2pNetworkKadBootstrapFailedRequest),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct P2pNetworkKadBootstrapOngoingRequest {
    pub peer_id: PeerId,
    pub address: P2pConnectionOutgoingInitOpts,
    pub start: Timestamp,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct P2pNetworkKadBootstrapSuccessfullRequest {
    pub peer_id: PeerId,
    pub address: P2pConnectionOutgoingInitOpts,
    pub start: Timestamp,
    pub finish: Timestamp,
    pub closest_peers: P2pNetworkKadLatestRequestPeers,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct P2pNetworkKadBootstrapFailedRequest {
    pub peer_id: PeerId,
    pub address: P2pConnectionOutgoingInitOpts,
    pub start: Timestamp,
    pub finish: Timestamp,
    pub error: String,
}
