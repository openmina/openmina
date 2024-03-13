use std::{collections::BTreeMap, net::SocketAddr};

use redux::Timestamp;
use serde::{Deserialize, Serialize};

use super::{
    bootstrap::P2pNetworkKadBootstrapState, request::P2pNetworkKadRequestState,
    stream::P2pNetworkKadStreamState, P2pNetworkKadRoutingTable,
};
use crate::{bootstrap::P2pNetworkKadBootstrapStats, PeerId, StreamId};

/// Kademlia status.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum P2pNetworkKadStatus {
    /// Initial state.
    #[default]
    Init,
    /// Bootstrap is in progress.
    Bootstrapping(super::bootstrap::P2pNetworkKadBootstrapState),
    /// Kademlia is bootstrapped.
    Bootstrapped {
        /// Timestamp of the bootstrap.
        time: Timestamp,
        /// Stats for the latest bootstrap process.
        stats: P2pNetworkKadBootstrapStats,
    },
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct P2pNetworkKadState {
    pub routing_table: P2pNetworkKadRoutingTable,
    pub latest_request_peers: P2pNetworkKadLatestRequestPeers,
    pub requests: BTreeMap<SocketAddr, P2pNetworkKadRequestState>,
    pub streams: crate::network::scheduler::StreamState<P2pNetworkKadStreamState>,
    pub status: P2pNetworkKadStatus,
}

impl P2pNetworkKadState {
    pub fn bootstrap_state(&self) -> Option<&super::bootstrap::P2pNetworkKadBootstrapState> {
        if let P2pNetworkKadStatus::Bootstrapping(state) = &self.status {
            Some(state)
        } else {
            None
        }
    }

    pub fn bootstrap_state_mut(&mut self) -> Option<&mut P2pNetworkKadBootstrapState> {
        if let P2pNetworkKadStatus::Bootstrapping(state) = &mut self.status {
            Some(state)
        } else {
            None
        }
    }

    pub fn bootstrap_stats(&self) -> Option<&P2pNetworkKadBootstrapStats> {
        match &self.status {
            P2pNetworkKadStatus::Init => None,
            P2pNetworkKadStatus::Bootstrapping(state) => Some(&state.stats),
            P2pNetworkKadStatus::Bootstrapped { stats, .. } => Some(stats),
        }
    }

    pub fn request(&self, addr: &SocketAddr) -> Option<&P2pNetworkKadRequestState> {
        self.requests.get(addr)
    }

    pub fn create_request(
        &mut self,
        addr: SocketAddr,
        peer_id: PeerId,
        key: PeerId,
    ) -> Result<&mut P2pNetworkKadRequestState, &P2pNetworkKadRequestState> {
        match self.requests.entry(addr) {
            std::collections::btree_map::Entry::Vacant(v) => {
                Ok(v.insert(P2pNetworkKadRequestState {
                    peer_id,
                    key,
                    addr,
                    status: crate::request::P2pNetworkKadRequestStatus::Default,
                }))
            }
            std::collections::btree_map::Entry::Occupied(o) => Err(o.into_mut()),
        }
    }

    pub fn find_kad_stream_state(
        &self,
        peer_id: &PeerId,
        stream_id: &StreamId,
    ) -> Option<&P2pNetworkKadStreamState> {
        self.streams.get(peer_id)?.get(stream_id)
    }

    pub fn create_kad_stream_state(
        &mut self,
        peer_id: &PeerId,
        stream_id: &StreamId,
    ) -> Result<&mut P2pNetworkKadStreamState, &P2pNetworkKadStreamState> {
        match self
            .streams
            .entry(peer_id.clone())
            .or_default()
            .entry(*stream_id)
        {
            std::collections::btree_map::Entry::Vacant(e) => Ok(e.insert(Default::default())),
            std::collections::btree_map::Entry::Occupied(e) => Err(e.into_mut()),
        }
    }

    pub fn find_kad_stream_state_mut(
        &mut self,
        peer_id: &PeerId,
        stream_id: &StreamId,
    ) -> Option<&mut P2pNetworkKadStreamState> {
        self.streams.get_mut(peer_id)?.get_mut(stream_id)
    }

    pub fn remove_kad_stream_state(&mut self, peer_id: &PeerId, stream_id: &StreamId) -> bool {
        self.streams
            .get_mut(peer_id)
            .map_or(false, |m| m.remove(stream_id).is_some())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, derive_more::Deref, derive_more::From)]
pub struct P2pNetworkKadLatestRequestPeers(Vec<(PeerId, P2pNetworkKadLatestRequestPeerKind)>);

impl P2pNetworkKadLatestRequestPeers {
    pub fn new(&self) -> impl Iterator<Item = &'_ PeerId> {
        self.of_kind(P2pNetworkKadLatestRequestPeerKind::New)
    }

    pub fn existing(&self) -> impl Iterator<Item = &'_ PeerId> {
        self.of_kind(P2pNetworkKadLatestRequestPeerKind::Existing)
    }

    pub fn discarded(&self) -> impl Iterator<Item = &'_ PeerId> {
        self.of_kind(P2pNetworkKadLatestRequestPeerKind::Discarded)
    }

    fn of_kind(
        &self,
        kind: P2pNetworkKadLatestRequestPeerKind,
    ) -> impl Iterator<Item = &'_ PeerId> {
        self.iter()
            .filter_map(move |(peer_id, k)| (kind == *k).then_some(peer_id))
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum P2pNetworkKadLatestRequestPeerKind {
    New,
    Existing,
    Discarded,
}
