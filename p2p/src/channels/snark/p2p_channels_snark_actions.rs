#[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
use openmina_core::snark::Snark;
use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::{channels::P2pChannelsAction, P2pState, PeerId};

use super::{P2pChannelsSnarkState, SnarkInfo, SnarkPropagationState};

pub type P2pChannelsSnarkActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pChannelsSnarkAction>;

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(peer_id)))]
pub enum P2pChannelsSnarkAction {
    Init {
        peer_id: PeerId,
    },
    Pending {
        peer_id: PeerId,
    },
    Ready {
        peer_id: PeerId,
    },
    RequestSend {
        peer_id: PeerId,
        limit: u8,
    },
    PromiseReceived {
        peer_id: PeerId,
        promised_count: u8,
    },
    Received {
        peer_id: PeerId,
        snark: SnarkInfo,
    },
    RequestReceived {
        peer_id: PeerId,
        limit: u8,
    },
    ResponseSend {
        peer_id: PeerId,
        snarks: Vec<SnarkInfo>,
        first_index: u64,
        last_index: u64,
    },
    #[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
    Libp2pReceived {
        peer_id: PeerId,
        snark: Snark,
        nonce: u32,
    },
    #[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
    Libp2pBroadcast {
        snark: Snark,
        nonce: u32,
    },
}

impl P2pChannelsSnarkAction {
    pub fn peer_id(&self) -> Option<&PeerId> {
        match self {
            Self::Init { peer_id }
            | Self::Pending { peer_id }
            | Self::Ready { peer_id }
            | Self::RequestSend { peer_id, .. }
            | Self::PromiseReceived { peer_id, .. }
            | Self::Received { peer_id, .. }
            | Self::RequestReceived { peer_id, .. }
            | Self::ResponseSend { peer_id, .. } => Some(peer_id),
            #[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
            Self::Libp2pReceived { peer_id, .. } => Some(peer_id),
            #[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
            Self::Libp2pBroadcast { .. } => None,
        }
    }
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkAction {
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
        match self {
            P2pChannelsSnarkAction::Init { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(&p.channels.snark, P2pChannelsSnarkState::Enabled)
                })
            }
            P2pChannelsSnarkAction::Pending { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(&p.channels.snark, P2pChannelsSnarkState::Init { .. })
                })
            }
            P2pChannelsSnarkAction::Ready { peer_id } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(&p.channels.snark, P2pChannelsSnarkState::Pending { .. })
                })
            }
            P2pChannelsSnarkAction::RequestSend { peer_id, .. } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(
                        &p.channels.snark,
                        P2pChannelsSnarkState::Ready {
                            local: SnarkPropagationState::WaitingForRequest { .. }
                                | SnarkPropagationState::Responded { .. },
                            ..
                        }
                    )
                })
            }
            P2pChannelsSnarkAction::PromiseReceived {
                peer_id,
                promised_count,
            } => state.get_ready_peer(peer_id).map_or(false, |p| {
                matches!(
                    &p.channels.snark,
                    P2pChannelsSnarkState::Ready {
                        local: SnarkPropagationState::Requested {
                            requested_limit, ..
                        }, ..
                    } if *promised_count > 0 && promised_count <= requested_limit
                )
            }),
            P2pChannelsSnarkAction::Received { peer_id, .. } => {
                state.get_ready_peer(peer_id).map_or(false, |p| {
                    matches!(
                        &p.channels.snark,
                        P2pChannelsSnarkState::Ready {
                            local: SnarkPropagationState::Responding { .. },
                            ..
                        }
                    )
                })
            }
            P2pChannelsSnarkAction::RequestReceived { peer_id, limit } => {
                *limit > 0
                    && state.get_ready_peer(peer_id).map_or(false, |p| {
                        matches!(
                            &p.channels.snark,
                            P2pChannelsSnarkState::Ready {
                                remote: SnarkPropagationState::WaitingForRequest { .. }
                                    | SnarkPropagationState::Responded { .. },
                                ..
                            }
                        )
                    })
            }
            P2pChannelsSnarkAction::ResponseSend {
                peer_id,
                snarks,
                first_index,
                last_index,
            } => {
                !snarks.is_empty()
                    && first_index < last_index
                    && state
                        .get_ready_peer(peer_id)
                        .map_or(false, |p| match &p.channels.snark {
                            P2pChannelsSnarkState::Ready {
                                remote,
                                next_send_index,
                                ..
                            } => {
                                if first_index < next_send_index {
                                    return false;
                                }
                                match remote {
                                    SnarkPropagationState::Requested {
                                        requested_limit, ..
                                    } => snarks.len() <= *requested_limit as usize,
                                    _ => false,
                                }
                            }
                            _ => false,
                        })
            }
            #[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
            P2pChannelsSnarkAction::Libp2pReceived { peer_id, .. } => state
                .peers
                .get(peer_id)
                .filter(|p| p.is_libp2p())
                .and_then(|p| p.status.as_ready())
                .map_or(false, |p| p.channels.snark.is_ready()),
            #[cfg(all(not(target_arch = "wasm32"), feature = "p2p-libp2p"))]
            P2pChannelsSnarkAction::Libp2pBroadcast { .. } => state
                .peers
                .iter()
                .any(|(_, p)| p.is_libp2p() && p.status.as_ready().is_some()),
        }
    }
}

impl From<P2pChannelsSnarkAction> for crate::P2pAction {
    fn from(action: P2pChannelsSnarkAction) -> Self {
        Self::Channels(P2pChannelsAction::Snark(action))
    }
}
