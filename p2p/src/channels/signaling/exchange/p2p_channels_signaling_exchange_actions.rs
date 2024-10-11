use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::{
    channels::P2pChannelsAction,
    connection::P2pConnectionResponse,
    identity::PublicKey,
    webrtc::{EncryptedAnswer, EncryptedOffer, Offer},
    P2pState, PeerId,
};

use super::{P2pChannelsSignalingExchangeState, SignalingExchangeState};

#[derive(Debug, Clone, Serialize, Deserialize, ActionEvent)]
#[action_event(fields(display(peer_id)))]
pub enum P2pChannelsSignalingExchangeAction {
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
    /// Send request to get next offer/incoming connection from peer.
    RequestSend {
        peer_id: PeerId,
    },
    OfferReceived {
        peer_id: PeerId,
        offerer_pub_key: PublicKey,
        offer: EncryptedOffer,
    },
    OfferDecryptError {
        peer_id: PeerId,
    },
    OfferDecryptSuccess {
        peer_id: PeerId,
        offer: Offer,
    },
    AnswerSend {
        peer_id: PeerId,
        answer: P2pConnectionResponse,
    },
    /// Received request to get next offer/incoming connection from peer.
    RequestReceived {
        peer_id: PeerId,
    },
    OfferSend {
        peer_id: PeerId,
        offerer_pub_key: PublicKey,
        offer: EncryptedOffer,
    },
    AnswerReceived {
        peer_id: PeerId,
        answer: Option<EncryptedAnswer>,
    },
}

impl P2pChannelsSignalingExchangeAction {
    pub fn peer_id(&self) -> &PeerId {
        match self {
            Self::Init { peer_id }
            | Self::Pending { peer_id }
            | Self::Ready { peer_id }
            | Self::RequestSend { peer_id }
            | Self::OfferReceived { peer_id, .. }
            | Self::OfferDecryptError { peer_id, .. }
            | Self::OfferDecryptSuccess { peer_id, .. }
            | Self::AnswerSend { peer_id, .. }
            | Self::RequestReceived { peer_id }
            | Self::OfferSend { peer_id, .. }
            | Self::AnswerReceived { peer_id, .. } => peer_id,
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSignalingExchangeAction {
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
        match self {
            P2pChannelsSignalingExchangeAction::Init { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(
                        &p.channels.signaling.exchange,
                        P2pChannelsSignalingExchangeState::Enabled
                    )
                })
            }
            P2pChannelsSignalingExchangeAction::Pending { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(
                        &p.channels.signaling.exchange,
                        P2pChannelsSignalingExchangeState::Init { .. }
                    )
                })
            }
            P2pChannelsSignalingExchangeAction::Ready { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(
                        &p.channels.signaling.exchange,
                        P2pChannelsSignalingExchangeState::Pending { .. }
                    )
                })
            }
            P2pChannelsSignalingExchangeAction::RequestSend { peer_id } => {
                !state.already_has_max_peers()
                    && state.get_ready_peer(peer_id).map_or(false, |p| {
                        match &p.channels.signaling.exchange {
                            P2pChannelsSignalingExchangeState::Ready { local, .. } => matches!(
                                local,
                                SignalingExchangeState::WaitingForRequest { .. }
                                    | SignalingExchangeState::Answered { .. },
                            ),
                            _ => false,
                        }
                    })
            }
            P2pChannelsSignalingExchangeAction::OfferReceived { peer_id, .. } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.signaling.exchange {
                    P2pChannelsSignalingExchangeState::Ready { local, .. } => {
                        matches!(local, SignalingExchangeState::Requested { .. })
                    }
                    _ => false,
                }),
            P2pChannelsSignalingExchangeAction::OfferDecryptError { peer_id } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.signaling.exchange {
                    P2pChannelsSignalingExchangeState::Ready { local, .. } => {
                        matches!(local, SignalingExchangeState::Requested { .. })
                    }
                    _ => false,
                }),
            P2pChannelsSignalingExchangeAction::OfferDecryptSuccess { peer_id, .. } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.signaling.exchange {
                    P2pChannelsSignalingExchangeState::Ready { local, .. } => {
                        matches!(local, SignalingExchangeState::Requested { .. })
                    }
                    _ => false,
                }),
            P2pChannelsSignalingExchangeAction::AnswerSend { peer_id, .. } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.signaling.exchange {
                    P2pChannelsSignalingExchangeState::Ready { local, .. } => {
                        matches!(local, SignalingExchangeState::Offered { .. })
                    }
                    _ => false,
                }),
            P2pChannelsSignalingExchangeAction::RequestReceived { peer_id } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.signaling.exchange {
                    P2pChannelsSignalingExchangeState::Ready { remote, .. } => matches!(
                        remote,
                        SignalingExchangeState::WaitingForRequest { .. }
                            | SignalingExchangeState::Answered { .. }
                    ),
                    _ => false,
                }),
            P2pChannelsSignalingExchangeAction::OfferSend { peer_id, .. } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.signaling.exchange {
                    P2pChannelsSignalingExchangeState::Ready { remote, .. } => {
                        matches!(remote, SignalingExchangeState::Requested { .. })
                    }
                    _ => false,
                }),
            P2pChannelsSignalingExchangeAction::AnswerReceived { peer_id, .. } => state
                .get_ready_peer(peer_id)
                .map_or(false, |p| match &p.channels.signaling.exchange {
                    P2pChannelsSignalingExchangeState::Ready { remote, .. } => {
                        matches!(remote, SignalingExchangeState::Offered { .. })
                    }
                    _ => false,
                }),
        }
    }
}

impl From<P2pChannelsSignalingExchangeAction> for crate::P2pAction {
    fn from(action: P2pChannelsSignalingExchangeAction) -> Self {
        Self::Channels(P2pChannelsAction::SignalingExchange(action))
    }
}
