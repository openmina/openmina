use serde::{Deserialize, Serialize};

use super::outgoing::P2pConnectionOutgoingAction;

pub type P2pConnectionActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pConnectionAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pConnectionAction {
    Outgoing(P2pConnectionOutgoingAction),
}

impl P2pConnectionAction {
    pub fn peer_id(&self) -> Option<&crate::PeerId> {
        match self {
            Self::Outgoing(v) => v.peer_id(),
        }
    }

    pub fn should_create_peer(&self) -> bool {
        match self {
            Self::Outgoing(P2pConnectionOutgoingAction::Init(_)) => true,
            Self::Outgoing(_) => false,
        }
    }

    pub fn dial_addrs(&self) -> &[libp2p::Multiaddr] {
        match self {
            Self::Outgoing(P2pConnectionOutgoingAction::Init(a)) => &a.opts.addrs,
            Self::Outgoing(_) => &[],
        }
    }
}
