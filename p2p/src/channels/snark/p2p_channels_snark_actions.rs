use openmina_core::snark::Snark;
use serde::{Deserialize, Serialize};

use crate::{P2pState, PeerId};

use super::{P2pChannelsSnarkState, SnarkInfo, SnarkPropagationState};

pub type P2pChannelsSnarkActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pChannelsSnarkAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pChannelsSnarkAction {
    Init(P2pChannelsSnarkInitAction),
    Pending(P2pChannelsSnarkPendingAction),
    Ready(P2pChannelsSnarkReadyAction),

    RequestSend(P2pChannelsSnarkRequestSendAction),
    PromiseReceived(P2pChannelsSnarkPromiseReceivedAction),
    Received(P2pChannelsSnarkReceivedAction),

    RequestReceived(P2pChannelsSnarkRequestReceivedAction),
    ResponseSend(P2pChannelsSnarkResponseSendAction),

    Libp2pReceived(P2pChannelsSnarkLibp2pReceivedAction),
    Libp2pBroadcast(P2pChannelsSnarkLibp2pBroadcastAction),
}

impl P2pChannelsSnarkAction {
    pub fn peer_id(&self) -> Option<&PeerId> {
        Some(match self {
            Self::Init(v) => &v.peer_id,
            Self::Pending(v) => &v.peer_id,
            Self::Ready(v) => &v.peer_id,
            Self::RequestSend(v) => &v.peer_id,
            Self::PromiseReceived(v) => &v.peer_id,
            Self::Received(v) => &v.peer_id,
            Self::RequestReceived(v) => &v.peer_id,
            Self::ResponseSend(v) => &v.peer_id,
            Self::Libp2pReceived(_) | Self::Libp2pBroadcast(_) => return None,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsSnarkInitAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkInitAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_ready_peer(&self.peer_id).map_or(false, |p| {
            matches!(&p.channels.snark, P2pChannelsSnarkState::Enabled)
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsSnarkPendingAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkPendingAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_ready_peer(&self.peer_id).map_or(false, |p| {
            matches!(&p.channels.snark, P2pChannelsSnarkState::Init { .. })
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsSnarkReadyAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkReadyAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state.get_ready_peer(&self.peer_id).map_or(false, |p| {
            matches!(&p.channels.snark, P2pChannelsSnarkState::Pending { .. })
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsSnarkRequestSendAction {
    pub peer_id: PeerId,
    pub limit: u8,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkRequestSendAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_ready_peer(&self.peer_id)
            .map_or(false, |p| match &p.channels.snark {
                P2pChannelsSnarkState::Ready { local, .. } => match local {
                    SnarkPropagationState::WaitingForRequest { .. } => true,
                    SnarkPropagationState::Responded { .. } => true,
                    _ => false,
                },
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsSnarkPromiseReceivedAction {
    pub peer_id: PeerId,
    pub promised_count: u8,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkPromiseReceivedAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_ready_peer(&self.peer_id)
            .map_or(false, |p| match &p.channels.snark {
                P2pChannelsSnarkState::Ready { local, .. } => match local {
                    SnarkPropagationState::Requested {
                        requested_limit, ..
                    } => self.promised_count > 0 && self.promised_count <= *requested_limit,
                    _ => false,
                },
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsSnarkReceivedAction {
    pub peer_id: PeerId,
    pub snark: SnarkInfo,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkReceivedAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .get_ready_peer(&self.peer_id)
            .map_or(false, |p| match &p.channels.snark {
                P2pChannelsSnarkState::Ready { local, .. } => match local {
                    SnarkPropagationState::Responding { .. } => true,
                    _ => false,
                },
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsSnarkRequestReceivedAction {
    pub peer_id: PeerId,
    pub limit: u8,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkRequestReceivedAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        self.limit > 0
            && state
                .get_ready_peer(&self.peer_id)
                .map_or(false, |p| match &p.channels.snark {
                    P2pChannelsSnarkState::Ready { remote, .. } => match remote {
                        SnarkPropagationState::WaitingForRequest { .. } => true,
                        SnarkPropagationState::Responded { .. } => true,
                        _ => false,
                    },
                    _ => false,
                })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsSnarkResponseSendAction {
    pub peer_id: PeerId,
    pub snarks: Vec<SnarkInfo>,
    pub first_index: u64,
    pub last_index: u64,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkResponseSendAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        !self.snarks.is_empty()
            && self.first_index < self.last_index
            && state
                .get_ready_peer(&self.peer_id)
                .map_or(false, |p| match &p.channels.snark {
                    P2pChannelsSnarkState::Ready {
                        remote,
                        next_send_index,
                        ..
                    } => {
                        if self.first_index < *next_send_index {
                            return false;
                        }
                        match remote {
                            SnarkPropagationState::Requested {
                                requested_limit, ..
                            } => self.snarks.len() <= *requested_limit as usize,
                            _ => false,
                        }
                    }
                    _ => false,
                })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsSnarkLibp2pReceivedAction {
    pub peer_id: PeerId,
    pub snark: Snark,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkLibp2pReceivedAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .filter(|p| p.is_libp2p())
            .and_then(|p| p.status.as_ready())
            .map_or(false, |p| p.channels.snark.is_ready())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pChannelsSnarkLibp2pBroadcastAction {
    pub snark: Snark,
}

impl redux::EnablingCondition<P2pState> for P2pChannelsSnarkLibp2pBroadcastAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .iter()
            .any(|(_, p)| p.is_libp2p() && p.status.as_ready().is_some())
    }
}

// --- From<LeafAction> for Action impls.

use crate::channels::P2pChannelsAction;

impl From<P2pChannelsSnarkInitAction> for crate::P2pAction {
    fn from(a: P2pChannelsSnarkInitAction) -> Self {
        Self::Channels(P2pChannelsAction::Snark(a.into()))
    }
}

impl From<P2pChannelsSnarkPendingAction> for crate::P2pAction {
    fn from(a: P2pChannelsSnarkPendingAction) -> Self {
        Self::Channels(P2pChannelsAction::Snark(a.into()))
    }
}

impl From<P2pChannelsSnarkReadyAction> for crate::P2pAction {
    fn from(a: P2pChannelsSnarkReadyAction) -> Self {
        Self::Channels(P2pChannelsAction::Snark(a.into()))
    }
}

impl From<P2pChannelsSnarkRequestSendAction> for crate::P2pAction {
    fn from(a: P2pChannelsSnarkRequestSendAction) -> Self {
        Self::Channels(P2pChannelsAction::Snark(a.into()))
    }
}

impl From<P2pChannelsSnarkPromiseReceivedAction> for crate::P2pAction {
    fn from(a: P2pChannelsSnarkPromiseReceivedAction) -> Self {
        Self::Channels(P2pChannelsAction::Snark(a.into()))
    }
}

impl From<P2pChannelsSnarkReceivedAction> for crate::P2pAction {
    fn from(a: P2pChannelsSnarkReceivedAction) -> Self {
        Self::Channels(P2pChannelsAction::Snark(a.into()))
    }
}

impl From<P2pChannelsSnarkRequestReceivedAction> for crate::P2pAction {
    fn from(a: P2pChannelsSnarkRequestReceivedAction) -> Self {
        Self::Channels(P2pChannelsAction::Snark(a.into()))
    }
}

impl From<P2pChannelsSnarkResponseSendAction> for crate::P2pAction {
    fn from(a: P2pChannelsSnarkResponseSendAction) -> Self {
        Self::Channels(P2pChannelsAction::Snark(a.into()))
    }
}

impl From<P2pChannelsSnarkLibp2pReceivedAction> for crate::P2pAction {
    fn from(a: P2pChannelsSnarkLibp2pReceivedAction) -> Self {
        Self::Channels(P2pChannelsAction::Snark(a.into()))
    }
}

impl From<P2pChannelsSnarkLibp2pBroadcastAction> for crate::P2pAction {
    fn from(a: P2pChannelsSnarkLibp2pBroadcastAction) -> Self {
        Self::Channels(P2pChannelsAction::Snark(a.into()))
    }
}
