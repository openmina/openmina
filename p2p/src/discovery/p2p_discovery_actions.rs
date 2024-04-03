use openmina_core::{action_debug, action_info, action_warn, log::ActionEvent};
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
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
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

impl ActionEvent for P2pDiscoveryAction {
    fn action_event<T>(&self, context: &T)
    where
        T: openmina_core::log::EventContext,
    {
        match self {
            P2pDiscoveryAction::Init { peer_id } => {
                action_debug!(context, peer_id = display(peer_id))
            }
            P2pDiscoveryAction::Success { peer_id, peers } => {
                action_debug!(context, peer_id = display(peer_id), peers = debug(peers))
            }
            P2pDiscoveryAction::KademliaBootstrap => action_debug!(context),
            P2pDiscoveryAction::KademliaInit => action_debug!(context),
            P2pDiscoveryAction::KademliaAddRoute { peer_id, addresses } => {
                action_debug!(
                    context,
                    peer_id = display(peer_id),
                    addresses = debug(addresses)
                )
            }
            P2pDiscoveryAction::KademliaSuccess { peers } => {
                action_info!(context, peers = debug(peers))
            }
            P2pDiscoveryAction::KademliaFailure { description } => {
                action_warn!(context, description)
            }
        }
    }
}
