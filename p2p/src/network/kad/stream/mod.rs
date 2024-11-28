mod p2p_network_kad_stream_state;
use redux::Callback;
use serde::{Deserialize, Serialize};

use crate::ConnectionAddr;
use crate::P2pNetworkKademliaAction;
use crate::PeerId;
use crate::StreamId;

pub use self::p2p_network_kad_stream_state::*;

mod p2p_network_kad_stream_actions;
pub use self::p2p_network_kad_stream_actions::*;

use super::P2pNetworkKadEntry;
use super::CID;

#[cfg(feature = "p2p-libp2p")]
mod p2p_network_kad_stream_reducer;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum P2pNetworkKademliaStreamWaitOutgoingCallback {
    AnswerFindNodeRequest {
        callback: Callback<(ConnectionAddr, PeerId, StreamId, CID)>,
        args: CID,
    },
    UpdateFindNodeRequest {
        callback: Callback<(ConnectionAddr, PeerId, StreamId, Vec<P2pNetworkKadEntry>)>,
        args: Vec<P2pNetworkKadEntry>,
    },
}

impl P2pNetworkKademliaStreamWaitOutgoingCallback {
    pub fn answer_find_node_request(cid: CID) -> Self {
        Self::AnswerFindNodeRequest {
            callback: redux::callback!(
                on_p2p_network_stream_wait_outgoing_answer_find_node_request((
                    addr: ConnectionAddr,
                    peer_id: PeerId,
                    stream_id: StreamId,
                    cid: CID
                )) -> crate::P2pAction{
                    P2pNetworkKademliaAction::AnswerFindNodeRequest { addr, peer_id, stream_id, key: cid }
                }
            ),
            args: cid,
        }
    }
    pub fn update_find_node_request(peers: Vec<P2pNetworkKadEntry>) -> Self {
        Self::UpdateFindNodeRequest {
            callback: redux::callback!(
                on_p2p_network_stream_wait_outgoing_answer_find_node_request((
                    addr: ConnectionAddr,
                    peer_id: PeerId,
                    stream_id: StreamId,
                    closest_peers: Vec<P2pNetworkKadEntry>
                )) -> crate::P2pAction{
                    P2pNetworkKademliaAction::UpdateFindNodeRequest { addr, peer_id, stream_id, closest_peers }
                }
            ),
            args: peers,
        }
    }
}
