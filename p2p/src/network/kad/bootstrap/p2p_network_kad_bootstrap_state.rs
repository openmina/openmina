use std::{
    collections::{BTreeMap, BTreeSet},
    net::SocketAddr,
};

use malloc_size_of_derive::MallocSizeOf;
use redux::Timestamp;
use serde::{Deserialize, Serialize};

use crate::{
    connection::outgoing::P2pConnectionOutgoingInitOpts, P2pNetworkKadKey, P2pNetworkKadKeyError,
    P2pNetworkKadLatestRequestPeers, PeerId,
};

#[derive(Clone, Debug, Serialize, Deserialize, MallocSizeOf)]
pub struct P2pNetworkKadBootstrapState {
    /// Key that is used to request closest peers. Usually self peer_id.
    pub key: PeerId,
    /// Kademlia key, `sha265(self.key)`.
    pub kademlia_key: P2pNetworkKadKey,
    /// Peers that already been contacted (successfully or not) for FIND_NODE.
    #[with_malloc_size_of_func = "measurement::peer_id_map"]
    pub processed_peers: BTreeSet<PeerId>,
    /// Ongoing FIND_NODE requests.
    ///
    /// TODO: replace with something more lightweight.
    #[with_malloc_size_of_func = "measurement::requests_map"]
    pub requests: BTreeMap<PeerId, P2pNetworkKadBoostrapRequestState>,
    /// Number of successful requests
    pub successful_requests: usize,
    /// Bootstrap requests statistics.
    pub stats: P2pNetworkKadBootstrapStats,
    /// Constructing request
    pub peer_id_req_vec: Vec<(PeerId, P2pNetworkKadBoostrapRequestState)>,
    /// Number of requests to construct
    pub requests_number: usize,
}

impl P2pNetworkKadBootstrapState {
    pub fn new(key: PeerId) -> Result<Self, P2pNetworkKadKeyError> {
        Ok(P2pNetworkKadBootstrapState {
            key,
            kademlia_key: key.try_into()?,
            processed_peers: BTreeSet::new(),
            requests: BTreeMap::new(),
            successful_requests: 0,
            stats: Default::default(),
            peer_id_req_vec: vec![],
            requests_number: 0,
        })
    }

    pub fn request(&self, peer_id: &PeerId) -> Option<&P2pNetworkKadBoostrapRequestState> {
        self.requests.get(peer_id)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, MallocSizeOf)]
pub struct P2pNetworkKadBoostrapRequestState {
    /// Address that is used for the current connection.
    // TODO: generalize to DNS addrs
    #[ignore_malloc_size_of = "doesn't allocate"]
    pub addr: SocketAddr,
    /// When connection to the peer was initiated.
    #[ignore_malloc_size_of = "doesn't allocate"]
    pub time: Timestamp,
    /// Addresses yet to be used, if current connection will fail.
    // TODO: use Multiaddr
    #[with_malloc_size_of_func = "measurement::socket_addr_vec"]
    pub addrs_to_use: Vec<SocketAddr>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, MallocSizeOf)]
pub struct P2pNetworkKadBootstrapStats {
    pub requests: Vec<P2pNetworkKadBootstrapRequestStat>,
}

#[derive(Clone, Debug, Serialize, Deserialize, MallocSizeOf)]
#[serde(tag = "type")]
pub enum P2pNetworkKadBootstrapRequestStat {
    Ongoing(P2pNetworkKadBootstrapOngoingRequest),
    Successful(P2pNetworkKadBootstrapSuccessfulRequest),
    Failed(P2pNetworkKadBootstrapFailedRequest),
}

#[derive(Clone, Debug, Serialize, Deserialize, MallocSizeOf)]
pub struct P2pNetworkKadBootstrapOngoingRequest {
    pub peer_id: PeerId,
    pub address: P2pConnectionOutgoingInitOpts,
    #[ignore_malloc_size_of = "doesn't allocate"]
    pub start: Timestamp,
}

#[derive(Clone, Debug, Serialize, Deserialize, MallocSizeOf)]
pub struct P2pNetworkKadBootstrapSuccessfulRequest {
    pub peer_id: PeerId,
    pub address: P2pConnectionOutgoingInitOpts,
    #[ignore_malloc_size_of = "doesn't allocate"]
    pub start: Timestamp,
    #[ignore_malloc_size_of = "doesn't allocate"]
    pub finish: Timestamp,
    pub closest_peers: P2pNetworkKadLatestRequestPeers,
}

#[derive(Clone, Debug, Serialize, Deserialize, MallocSizeOf)]
pub struct P2pNetworkKadBootstrapFailedRequest {
    pub peer_id: PeerId,
    pub address: P2pConnectionOutgoingInitOpts,
    #[ignore_malloc_size_of = "doesn't allocate"]
    pub start: Timestamp,
    #[ignore_malloc_size_of = "doesn't allocate"]
    pub finish: Timestamp,
    pub error: String,
}

mod measurement {
    use std::{
        collections::{BTreeMap, BTreeSet},
        mem,
        net::SocketAddr,
    };

    use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};

    use super::P2pNetworkKadBoostrapRequestState;
    use crate::PeerId;

    pub fn socket_addr_vec(val: &Vec<SocketAddr>, _ops: &mut MallocSizeOfOps) -> usize {
        val.capacity() * mem::size_of::<SocketAddr>()
    }

    pub fn peer_id_map(val: &BTreeSet<PeerId>, _ops: &mut MallocSizeOfOps) -> usize {
        val.len() * mem::size_of::<PeerId>()
    }

    pub fn requests_map(
        val: &BTreeMap<PeerId, P2pNetworkKadBoostrapRequestState>,
        ops: &mut MallocSizeOfOps,
    ) -> usize {
        val.iter()
            .map(|(k, v)| mem::size_of_val(k) + mem::size_of_val(v) + v.size_of(ops))
            .sum()
    }
}
