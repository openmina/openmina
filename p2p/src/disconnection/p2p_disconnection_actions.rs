use std::time::Duration;

use openmina_core::ActionEvent;
use serde::{Deserialize, Serialize};

use super::P2pDisconnectionReason;
use crate::{P2pPeerStatus, P2pState, PeerId};

pub type P2pDisconnectionActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pDisconnectionAction>;

const RANDOM_DISCONNECTION_TRY_FREQUENCY: Duration = Duration::from_secs(10);

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(level = debug)]
pub enum P2pDisconnectionAction {
    RandomTry,
    /// Initialize disconnection.
    #[action_event(fields(display(peer_id), display(reason)), level = info)]
    Init {
        peer_id: PeerId,
        reason: P2pDisconnectionReason,
    },
    /// Peer disconnection.
    #[action_event(fields(display(peer_id)), level = info)]
    PeerClosed {
        peer_id: PeerId,
    },
    #[action_event(fields(display(peer_id)), level = info)]
    FailedCleanup {
        peer_id: PeerId,
    },
    /// Finish disconnecting from a peer.
    #[action_event(fields(display(peer_id)), level = debug)]
    Finish {
        peer_id: PeerId,
    },
}

impl redux::EnablingCondition<P2pState> for P2pDisconnectionAction {
    fn is_enabled(&self, state: &P2pState, time: redux::Timestamp) -> bool {
        match self {
            P2pDisconnectionAction::RandomTry => time
                .checked_sub(state.last_random_disconnection_try)
                .map_or(false, |dur| dur >= RANDOM_DISCONNECTION_TRY_FREQUENCY),
            P2pDisconnectionAction::Init { peer_id, .. }
            | P2pDisconnectionAction::PeerClosed { peer_id, .. }
            | P2pDisconnectionAction::Finish { peer_id } => {
                state.peers.get(peer_id).map_or(false, |peer| {
                    !matches!(peer.status, P2pPeerStatus::Disconnected { .. })
                        && !peer.status.is_error()
                })
            }
            P2pDisconnectionAction::FailedCleanup { peer_id } => state
                .peers
                .get(peer_id)
                .map_or(false, |peer| !peer.is_libp2p() && peer.status.is_error()),
        }
    }
}
