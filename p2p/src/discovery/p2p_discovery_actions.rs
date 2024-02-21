use serde::{Deserialize, Serialize};

use crate::{connection::outgoing::P2pConnectionOutgoingInitOpts, P2pState, PeerId};

// use super::{incoming::P2pConnectionIncomingAction, outgoing::P2pConnectionOutgoingAction};

pub type P2pDiscoveryActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pDiscoveryAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pDiscoveryAction {
    Init {
        peer_id: PeerId,
    },
    Success {
        peer_id: PeerId,
        peers: Vec<P2pConnectionOutgoingInitOpts>,
    },
    KademliaBootstrap,
    KademliaInit,
    KademliaAddRoute {
        peer_id: PeerId,
        addresses: Vec<P2pConnectionOutgoingInitOpts>,
    },
    KademliaSuccess {
        peers: Vec<PeerId>,
    },
    KademliaFailure {
        description: String,
    },
}

impl redux::EnablingCondition<P2pState> for P2pDiscoveryAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        match self {
            Self::Init { peer_id } => state.get_ready_peer(peer_id).is_some(),
            Self::Success { .. } => true,
            Self::KademliaBootstrap => !state.kademlia.is_ready && !state.kademlia.is_bootstrapping,
            Self::KademliaInit => {
                state.kademlia.is_ready
                    && state.kademlia.outgoing_requests < 1
                    && !state.already_knows_max_peers()
            }
            Self::KademliaAddRoute { .. } => true,
            Self::KademliaSuccess { .. } => true,
            Self::KademliaFailure { .. } => true,
        }
    }
}
