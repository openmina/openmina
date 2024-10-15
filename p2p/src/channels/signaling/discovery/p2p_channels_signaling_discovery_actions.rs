use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::{
    channels::P2pChannelsAction,
    connection::P2pConnectionResponse,
    identity::PublicKey,
    webrtc::{EncryptedAnswer, EncryptedOffer, Offer},
    P2pState, PeerId,
};

use super::{P2pChannelsSignalingDiscoveryState, SignalingDiscoveryState};

#[derive(Debug, Clone, Serialize, Deserialize, ActionEvent)]
#[action_event(fields(display(peer_id)))]
pub enum P2pChannelsSignalingDiscoveryAction {
    /// Initialize channel.
    Init {
        peer_id: PeerId,
    },
    Pending {
        peer_id: PeerId,
    },
    /// Channel is ready.
    Ready {
        peer_id: PeerId,
    },
    /// Send request to get next peer discovery request from peer.
    RequestSend {
        peer_id: PeerId,
    },
    DiscoveryRequestReceived {
        peer_id: PeerId,
    },
    DiscoveredSend {
        peer_id: PeerId,
        target_public_key: PublicKey,
    },
    DiscoveredRejectReceived {
        peer_id: PeerId,
    },
    DiscoveredAcceptReceived {
        peer_id: PeerId,
        offer: EncryptedOffer,
    },
    AnswerSend {
        peer_id: PeerId,
        answer: Option<EncryptedAnswer>,
    },
    /// Received request to get next peer discovery request from us.
    RequestReceived {
        peer_id: PeerId,
    },
    DiscoveryRequestSend {
        peer_id: PeerId,
    },
    DiscoveredReceived {
        peer_id: PeerId,
        target_public_key: PublicKey,
    },
    DiscoveredReject {
        peer_id: PeerId,
    },
    DiscoveredAccept {
        peer_id: PeerId,
        offer: Box<Offer>,
    },
    AnswerReceived {
        peer_id: PeerId,
        answer: Option<EncryptedAnswer>,
    },
    AnswerDecrypted {
        peer_id: PeerId,
        answer: P2pConnectionResponse,
    },
}

impl P2pChannelsSignalingDiscoveryAction {
    pub fn peer_id(&self) -> &PeerId {
        match self {
            Self::Init { peer_id }
            | Self::Pending { peer_id }
            | Self::Ready { peer_id }
            | Self::RequestSend { peer_id }
            | Self::DiscoveryRequestReceived { peer_id }
            | Self::DiscoveredSend { peer_id, .. }
            | Self::DiscoveredRejectReceived { peer_id }
            | Self::DiscoveredAcceptReceived { peer_id, .. }
            | Self::AnswerSend { peer_id, .. }
            | Self::RequestReceived { peer_id }
            | Self::DiscoveryRequestSend { peer_id, .. }
            | Self::DiscoveredReceived { peer_id, .. }
            | Self::DiscoveredReject { peer_id, .. }
            | Self::DiscoveredAccept { peer_id, .. }
            | Self::AnswerReceived { peer_id, .. }
            | Self::AnswerDecrypted { peer_id, .. } => peer_id,
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSignalingDiscoveryAction {
    fn is_enabled(&self, state: &P2pState, now: redux::Timestamp) -> bool {
        match self {
            P2pChannelsSignalingDiscoveryAction::Init { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(
                        &p.channels.signaling.discovery,
                        P2pChannelsSignalingDiscoveryState::Enabled
                    )
                })
            }
            P2pChannelsSignalingDiscoveryAction::Pending { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(
                        &p.channels.signaling.discovery,
                        P2pChannelsSignalingDiscoveryState::Init { .. }
                    )
                })
            }
            P2pChannelsSignalingDiscoveryAction::Ready { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(
                        &p.channels.signaling.discovery,
                        P2pChannelsSignalingDiscoveryState::Pending { .. }
                    )
                })
            }
            P2pChannelsSignalingDiscoveryAction::RequestSend { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    match &p.channels.signaling.discovery {
                        P2pChannelsSignalingDiscoveryState::Ready { local, .. } => {
                            match local {
                                SignalingDiscoveryState::WaitingForRequest { .. } => true,
                                SignalingDiscoveryState::DiscoveredRejected { time, .. }
                                | SignalingDiscoveryState::Answered { time, .. } => {
                                    // Allow one discovery request per minute.
                                    // TODO(binier): make configurable
                                    now.checked_sub(*time)
                                        .map_or(false, |dur| dur.as_secs() >= 60)
                                }
                                _ => false,
                            }
                        }
                        _ => false,
                    }
                })
            }
            P2pChannelsSignalingDiscoveryAction::DiscoveryRequestReceived { peer_id, .. } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.signaling.discovery {
                    P2pChannelsSignalingDiscoveryState::Ready { local, .. } => {
                        matches!(local, SignalingDiscoveryState::Requested { .. })
                    }
                    _ => false,
                }),
            P2pChannelsSignalingDiscoveryAction::DiscoveredSend {
                peer_id,
                target_public_key,
                ..
            } => {
                let target_peer_id = target_public_key.peer_id();
                let has_peer_requested_discovery =
                    state.get_ready_peer(peer_id).map_or(false, |p| {
                        match &p.channels.signaling.discovery {
                            P2pChannelsSignalingDiscoveryState::Ready { local, .. } => {
                                matches!(local, SignalingDiscoveryState::DiscoveryRequested { .. })
                            }
                            _ => false,
                        }
                    });
                let target_peer_already_discovering_them =
                    state.get_ready_peer(&target_peer_id).map_or(false, |p| {
                        p.channels.signaling.sent_discovered_peer_id() == Some(*peer_id)
                    });
                has_peer_requested_discovery
                    && !target_peer_already_discovering_them
                    && state.ready_peers_iter().all(|(_, p)| {
                        p.channels.signaling.sent_discovered_peer_id() != Some(target_peer_id)
                    })
            }
            P2pChannelsSignalingDiscoveryAction::DiscoveredRejectReceived { peer_id, .. } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.signaling.discovery {
                    P2pChannelsSignalingDiscoveryState::Ready { local, .. } => {
                        matches!(local, SignalingDiscoveryState::Discovered { .. })
                    }
                    _ => false,
                }),
            P2pChannelsSignalingDiscoveryAction::DiscoveredAcceptReceived { peer_id, .. } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.signaling.discovery {
                    P2pChannelsSignalingDiscoveryState::Ready { local, .. } => {
                        matches!(local, SignalingDiscoveryState::Discovered { .. })
                    }
                    _ => false,
                }),
            P2pChannelsSignalingDiscoveryAction::AnswerSend { peer_id, .. } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.signaling.discovery {
                    P2pChannelsSignalingDiscoveryState::Ready { local, .. } => {
                        matches!(local, SignalingDiscoveryState::DiscoveredAccepted { .. })
                    }
                    _ => false,
                }),
            P2pChannelsSignalingDiscoveryAction::RequestReceived { peer_id } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.signaling.discovery {
                    P2pChannelsSignalingDiscoveryState::Ready { remote, .. } => matches!(
                        remote,
                        SignalingDiscoveryState::WaitingForRequest { .. }
                            | SignalingDiscoveryState::DiscoveredRejected { .. }
                            | SignalingDiscoveryState::Answered { .. }
                    ),
                    _ => false,
                }),
            // TODO(binier): constrain interval between these requests
            // to handle malicious peers.
            P2pChannelsSignalingDiscoveryAction::DiscoveryRequestSend { peer_id, .. } => {
                !state.already_has_min_peers()
                    && state.get_ready_peer(peer_id).map_or(false, |p| {
                        match &p.channels.signaling.discovery {
                            P2pChannelsSignalingDiscoveryState::Ready { remote, .. } => {
                                matches!(remote, SignalingDiscoveryState::Requested { .. })
                            }
                            _ => false,
                        }
                    })
            }
            P2pChannelsSignalingDiscoveryAction::DiscoveredReceived { peer_id, .. } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.signaling.discovery {
                    P2pChannelsSignalingDiscoveryState::Ready { remote, .. } => {
                        matches!(remote, SignalingDiscoveryState::DiscoveryRequested { .. })
                    }
                    _ => false,
                }),
            P2pChannelsSignalingDiscoveryAction::DiscoveredReject { peer_id, .. } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.signaling.discovery {
                    P2pChannelsSignalingDiscoveryState::Ready { remote, .. } => {
                        matches!(remote, SignalingDiscoveryState::Discovered { .. })
                    }
                    _ => false,
                }),
            P2pChannelsSignalingDiscoveryAction::DiscoveredAccept { peer_id, .. } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.signaling.discovery {
                    P2pChannelsSignalingDiscoveryState::Ready { remote, .. } => {
                        matches!(remote, SignalingDiscoveryState::Discovered { .. })
                    }
                    _ => false,
                }),
            P2pChannelsSignalingDiscoveryAction::AnswerReceived { peer_id, .. } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.signaling.discovery {
                    P2pChannelsSignalingDiscoveryState::Ready { remote, .. } => {
                        matches!(remote, SignalingDiscoveryState::DiscoveredAccepted { .. })
                    }
                    _ => false,
                }),
            P2pChannelsSignalingDiscoveryAction::AnswerDecrypted { peer_id, .. } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.signaling.discovery {
                    P2pChannelsSignalingDiscoveryState::Ready { remote, .. } => {
                        matches!(remote, SignalingDiscoveryState::DiscoveredAccepted { .. })
                    }
                    _ => false,
                }),
        }
    }
}

impl From<P2pChannelsSignalingDiscoveryAction> for crate::P2pAction {
    fn from(action: P2pChannelsSignalingDiscoveryAction) -> Self {
        Self::Channels(P2pChannelsAction::SignalingDiscovery(action))
    }
}
