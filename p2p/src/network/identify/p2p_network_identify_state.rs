use crate::{network::identify::stream::P2pNetworkIdentifyStreamState, PeerId, StreamId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct P2pNetworkIdentifyState {
    pub streams: crate::network::scheduler::StreamState<P2pNetworkIdentifyStreamState>,
}

impl P2pNetworkIdentifyState {
    pub fn find_identify_stream_state(
        &self,
        peer_id: &PeerId,
        stream_id: &StreamId,
    ) -> Option<&P2pNetworkIdentifyStreamState> {
        self.streams.get(peer_id)?.get(stream_id)
    }

    pub fn create_identify_stream_state(
        &mut self,
        peer_id: &PeerId,
        stream_id: &StreamId,
    ) -> Result<&mut P2pNetworkIdentifyStreamState, &P2pNetworkIdentifyStreamState> {
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

    pub fn find_identify_stream_state_mut(
        &mut self,
        peer_id: &PeerId,
        stream_id: &StreamId,
    ) -> Option<&mut P2pNetworkIdentifyStreamState> {
        self.streams.get_mut(peer_id)?.get_mut(stream_id)
    }

    pub fn remove_identify_stream_state(&mut self, peer_id: &PeerId, stream_id: &StreamId) -> bool {
        self.streams
            .get_mut(peer_id)
            .map_or(false, |m| m.remove(stream_id).is_some())
    }
}
