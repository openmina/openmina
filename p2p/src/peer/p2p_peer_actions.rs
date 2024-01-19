use openmina_core::{block::ArcBlockWithHash, requests::RpcId};
use serde::{Deserialize, Serialize};

use crate::{
    connection::{
        libp2p::outgoing::{
            P2pConnectionLibP2pOutgoingErrorState, P2pConnectionLibP2pOutgoingState,
        },
        webrtc::{
            incoming::P2pConnectionWebRTCIncomingState, outgoing::P2pConnectionWebRTCOutgoingState,
        },
        P2pConnectionState,
    },
    libp2p::P2pLibP2pAddr,
    webrtc::SignalingMethod,
    P2pAction, P2pLibP2pPeerState, P2pPeerState, P2pPeerStatus, P2pState, P2pWebRTCPeerState,
    P2pWebRTCPeerStatus, PeerId,
};

pub type P2pPeerActionWithMeta = redux::ActionWithMeta<P2pPeerAction>;
pub type P2pPeerActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pPeerAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pPeerAction {
    AddLibP2p(P2pPeerAddLibP2pAction),
    AddWebRTC(P2pPeerAddWebRTCAction),
    Reconnect(P2pPeerReconnectAction),
    Ready(P2pPeerReadyAction),
    BestTipUpdate(P2pPeerBestTipUpdateAction),
}

impl P2pPeerAction {
    pub fn peer_id(&self) -> &PeerId {
        match self {
            Self::AddLibP2p(v) => &v.peer_id,
            Self::AddWebRTC(v) => &v.peer_id,
            Self::Reconnect(v) => &v.peer_id,
            Self::Ready(v) => &v.peer_id,
            Self::BestTipUpdate(v) => &v.peer_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPeerAddLibP2pAction {
    pub peer_id: PeerId,
    pub addrs: Vec<P2pLibP2pAddr>,
    pub rpc_id: Option<RpcId>,
}

impl redux::EnablingCondition<P2pState> for P2pPeerAddLibP2pAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        !state.peers.contains_key(&self.peer_id) && !self.addrs.is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPeerAddWebRTCAction {
    pub peer_id: PeerId,
    pub addr: SignalingMethod,
    pub rpc_id: Option<RpcId>,
}

impl redux::EnablingCondition<P2pState> for P2pPeerAddWebRTCAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        !state.peers.contains_key(&self.peer_id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPeerReconnectAction {
    pub peer_id: PeerId,
}

impl redux::EnablingCondition<P2pState> for P2pPeerReconnectAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        if state.already_has_min_peers() {
            return false;
        }

        state
            .peers
            .iter()
            .filter_map(|(id, p)| match p {
                P2pPeerState::WebRTC(P2pWebRTCPeerState {
                    dial_opts, status, ..
                }) => {
                    if dial_opts.is_none() {
                        return None;
                    }
                    match status {
                        P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                            P2pConnectionWebRTCOutgoingState::Error { time, .. },
                        )) => Some((time, id)),
                        P2pPeerStatus::Connecting(P2pConnectionState::Incoming(
                            P2pConnectionWebRTCIncomingState::Error { time, .. },
                        )) => Some((time, id)),
                        P2pWebRTCPeerStatus::Disconnected { time } => Some((time, id)),
                        _ => None,
                    }
                }
                P2pPeerState::Libp2p(P2pLibP2pPeerState {
                    status, dial_opts, ..
                }) => {
                    if dial_opts.is_empty() {
                        return None;
                    }
                    match status {
                        P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                            P2pConnectionLibP2pOutgoingState::Error(
                                P2pConnectionLibP2pOutgoingErrorState { time, .. },
                            ),
                        )) => Some((time, id)),
                        P2pPeerStatus::Disconnected { time } => Some((time, id)),
                        _ => None,
                    }
                }
                _ => None,
            })
            .min_by_key(|(time, ..)| *time)
            .filter(|(_, id)| *id == &self.peer_id)
            .is_some()
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPeerReadyAction {
    pub peer_id: PeerId,
    pub incoming: bool,
}

impl redux::EnablingCondition<P2pState> for P2pPeerReadyAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |p| p.is_connecting_success())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPeerBestTipUpdateAction {
    pub peer_id: PeerId,
    pub best_tip: ArcBlockWithHash,
}

impl redux::EnablingCondition<P2pState> for P2pPeerBestTipUpdateAction {
    fn is_enabled(&self, state: &P2pState) -> bool {
        // TODO(binier): don't enable if block inferrior than existing peer's
        // best tip.
        state.get_ready_peer(&self.peer_id).is_some()
    }
}

impl From<P2pPeerReadyAction> for P2pAction {
    fn from(value: P2pPeerReadyAction) -> Self {
        Self::Peer(value.into())
    }
}

impl From<P2pPeerBestTipUpdateAction> for P2pAction {
    fn from(value: P2pPeerBestTipUpdateAction) -> Self {
        Self::Peer(value.into())
    }
}

impl From<P2pPeerAddLibP2pAction> for P2pAction {
    fn from(value: P2pPeerAddLibP2pAction) -> Self {
        Self::Peer(value.into())
    }
}

impl From<P2pPeerAddWebRTCAction> for P2pAction {
    fn from(value: P2pPeerAddWebRTCAction) -> Self {
        Self::Peer(value.into())
    }
}

impl From<P2pPeerReconnectAction> for P2pAction {
    fn from(value: P2pPeerReconnectAction) -> Self {
        Self::Peer(value.into())
    }
}
