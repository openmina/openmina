use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use crate::{connection::outgoing::P2pConnectionOutgoingInitOpts, P2pState, PeerId};

// use super::{incoming::P2pConnectionIncomingAction, outgoing::P2pConnectionOutgoingAction};

pub type P2pDiscoveryActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pDiscoveryAction>;

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(fields(display(peer_id), debug(peers), debug(addresses), description))]
pub enum P2pDiscoveryAction {
    Init {
        peer_id: PeerId,
    },
    Success {
        peer_id: PeerId,
        peers: Vec<P2pConnectionOutgoingInitOpts>,
    },
}

impl redux::EnablingCondition<P2pState> for P2pDiscoveryAction {
    fn is_enabled(&self, state: &P2pState, _time: redux::Timestamp) -> bool {
        match self {
            Self::Init { peer_id } => state.get_ready_peer(peer_id).is_some(),
            Self::Success { .. } => true,
        }
    }
}
