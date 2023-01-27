use serde::{Deserialize, Serialize};

use shared::requests::RpcId;

pub type P2pConnectionOutgoingActionWithMetaRef<'a> =
    redux::ActionWithMeta<&'a P2pConnectionOutgoingAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionOutgoingAction {
    RandomInit(P2pConnectionOutgoingRandomInitAction),
    Init(P2pConnectionOutgoingInitAction),
    Reconnect(P2pConnectionOutgoingReconnectAction),
    Pending(P2pConnectionOutgoingPendingAction),
    Error(P2pConnectionOutgoingErrorAction),
    Success(P2pConnectionOutgoingSuccessAction),
}

impl P2pConnectionOutgoingAction {
    pub fn peer_id(&self) -> Option<&crate::PeerId> {
        match self {
            Self::RandomInit(_) => None,
            Self::Init(v) => Some(&v.opts.peer_id),
            Self::Reconnect(v) => Some(&v.opts.peer_id),
            Self::Pending(v) => Some(&v.peer_id),
            Self::Error(v) => Some(&v.peer_id),
            Self::Success(v) => Some(&v.peer_id),
        }
    }
}

fn already_connected_or_connecting(state: &crate::P2pState) -> bool {
    state.peers.iter().any(|(_, p)| match &p.status {
        P2pPeerStatus::Connecting(s) => match s {
            P2pConnectionState::Outgoing(s) => match s {
                P2pConnectionOutgoingState::Pending { .. } => true,
                _ => false,
            },
        },
        P2pPeerStatus::Ready(_) => true,
        _ => false,
    })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingRandomInitAction {}

impl redux::EnablingCondition<crate::P2pState> for P2pConnectionOutgoingRandomInitAction {
    fn is_enabled(&self, state: &crate::P2pState) -> bool {
        !already_connected_or_connecting(state) && !state.initial_unused_peers().is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingInitAction {
    pub opts: P2pConnectionOutgoingInitOpts,
    pub rpc_id: Option<RpcId>,
}

impl redux::EnablingCondition<crate::P2pState> for P2pConnectionOutgoingInitAction {
    fn is_enabled(&self, state: &crate::P2pState) -> bool {
        !already_connected_or_connecting(state) && !state.peers.contains_key(&self.opts.peer_id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingReconnectAction {
    pub opts: P2pConnectionOutgoingInitOpts,
    pub rpc_id: Option<RpcId>,
}

impl redux::EnablingCondition<crate::P2pState> for P2pConnectionOutgoingReconnectAction {
    fn is_enabled(&self, state: &crate::P2pState) -> bool {
        if already_connected_or_connecting(state) {
            return false;
        }

        state
            .peers
            .iter()
            .filter_map(|(id, p)| match &p.status {
                P2pPeerStatus::Connecting(s) => match s {
                    P2pConnectionState::Outgoing(P2pConnectionOutgoingState::Error {
                        time,
                        ..
                    }) => Some((*time, id, &p.dial_addrs)),
                    _ => None,
                },
                P2pPeerStatus::Disconnected { time } => Some((*time, id, &p.dial_addrs)),
                P2pPeerStatus::Ready(_) => None,
            })
            .min_by_key(|(time, ..)| *time)
            .filter(|(_, id, _)| *id == &self.opts.peer_id)
            .filter(|(.., addrs)| !addrs.is_empty() && *addrs == &self.opts.addrs)
            .is_some()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingInitOpts {
    pub peer_id: crate::PeerId,
    pub addrs: Vec<libp2p::Multiaddr>,
}

impl TryFrom<libp2p::Multiaddr> for P2pConnectionOutgoingInitOpts {
    // TODO(binier): replace with newtype error.
    type Error = &'static str;

    fn try_from(value: libp2p::Multiaddr) -> Result<Self, Self::Error> {
        use libp2p::multiaddr::Protocol;
        let hash = value
            .iter()
            .find_map(|p| match p {
                Protocol::P2p(hash) => Some(hash),
                _ => None,
            })
            .ok_or("peer_id not set in multiaddr. Missing `../p2p/<peer_id>`")?;
        let peer_id = match crate::PeerId::from_multihash(hash) {
            Ok(v) => v,
            Err(_) => return Err("invalid peer_id"),
        };

        Ok(Self {
            peer_id,
            addrs: vec![value],
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingPendingAction {
    pub peer_id: crate::PeerId,
}

impl redux::EnablingCondition<crate::P2pState> for P2pConnectionOutgoingPendingAction {
    fn is_enabled(&self, state: &crate::P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::Init { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingErrorAction {
    pub peer_id: crate::PeerId,
    pub error: String,
}

impl redux::EnablingCondition<crate::P2pState> for P2pConnectionOutgoingErrorAction {
    fn is_enabled(&self, state: &crate::P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::Pending { .. },
                )) => true,
                _ => false,
            })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pConnectionOutgoingSuccessAction {
    pub peer_id: crate::PeerId,
}

impl redux::EnablingCondition<crate::P2pState> for P2pConnectionOutgoingSuccessAction {
    fn is_enabled(&self, state: &crate::P2pState) -> bool {
        state
            .peers
            .get(&self.peer_id)
            .map_or(false, |peer| match &peer.status {
                P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                    P2pConnectionOutgoingState::Pending { .. },
                )) => true,
                _ => false,
            })
    }
}

// --- From<LeafAction> for Action impls.
use crate::{
    connection::{P2pConnectionAction, P2pConnectionState},
    P2pPeerStatus,
};

use super::P2pConnectionOutgoingState;

impl From<P2pConnectionOutgoingRandomInitAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingRandomInitAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}

impl From<P2pConnectionOutgoingInitAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingInitAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}

impl From<P2pConnectionOutgoingReconnectAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingReconnectAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}

impl From<P2pConnectionOutgoingPendingAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingPendingAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}

impl From<P2pConnectionOutgoingErrorAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingErrorAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}

impl From<P2pConnectionOutgoingSuccessAction> for crate::P2pAction {
    fn from(a: P2pConnectionOutgoingSuccessAction) -> Self {
        Self::Connection(P2pConnectionAction::Outgoing(a.into()))
    }
}
