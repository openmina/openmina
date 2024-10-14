use serde::{Deserialize, Serialize};

use crate::PeerId;

use super::{
    discovery::{P2pChannelsSignalingDiscoveryState, SignalingDiscoveryState},
    exchange::{P2pChannelsSignalingExchangeState, SignalingExchangeState},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsSignalingState {
    pub discovery: P2pChannelsSignalingDiscoveryState,
    pub exchange: P2pChannelsSignalingExchangeState,
}

impl P2pChannelsSignalingState {
    pub fn am_looking_for_peer(&self) -> bool {
        match &self.discovery {
            P2pChannelsSignalingDiscoveryState::Ready { remote, .. } => {
                matches!(remote, SignalingDiscoveryState::DiscoveryRequested { .. })
            }
            _ => false,
        }
    }

    pub fn received_discovered_peer_id(&self) -> Option<PeerId> {
        match &self.discovery {
            P2pChannelsSignalingDiscoveryState::Ready { remote, .. } => match remote {
                SignalingDiscoveryState::Discovered {
                    target_public_key, ..
                }
                | SignalingDiscoveryState::DiscoveredAccepted {
                    target_public_key, ..
                } => Some(target_public_key.peer_id()),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn is_looking_for_peer(&self) -> bool {
        match &self.discovery {
            P2pChannelsSignalingDiscoveryState::Ready { local, .. } => {
                matches!(local, SignalingDiscoveryState::DiscoveryRequested { .. })
            }
            _ => false,
        }
    }

    pub fn sent_discovered_peer_id(&self) -> Option<PeerId> {
        match &self.discovery {
            P2pChannelsSignalingDiscoveryState::Ready { local, .. } => match local {
                SignalingDiscoveryState::Discovered {
                    target_public_key, ..
                }
                | SignalingDiscoveryState::DiscoveredAccepted {
                    target_public_key, ..
                } => Some(target_public_key.peer_id()),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn is_looking_for_incoming_peer(&self) -> bool {
        match &self.exchange {
            P2pChannelsSignalingExchangeState::Ready { remote, .. } => {
                matches!(remote, SignalingExchangeState::Requested { .. })
            }
            _ => false,
        }
    }
}
