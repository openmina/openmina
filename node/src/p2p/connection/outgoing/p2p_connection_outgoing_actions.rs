use std::time::Duration;

use crate::p2p::connection::P2pConnectionState;
use crate::p2p::P2pPeerStatus;

use super::*;

impl redux::EnablingCondition<crate::State> for P2pConnectionOutgoingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        match self {
            P2pConnectionOutgoingAction::Reconnect { opts, .. } => {
                if !self.is_enabled(&state.p2p) {
                    return false;
                }

                let Some(peer) = state.p2p.peers.get(opts.peer_id()) else {
                    return false;
                };
                let delay_passed = match &peer.status {
                    P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                        P2pConnectionOutgoingState::Error { time, .. },
                    ))
                    | P2pPeerStatus::Disconnected { time, .. } => {
                        state.time().checked_sub(*time) >= Some(Duration::from_secs(30))
                    }
                    _ => true,
                };
                delay_passed
            }
            P2pConnectionOutgoingAction::Timeout { peer_id } => {
                let peer = state.p2p.peers.get(peer_id);
                let timed_out = peer
                    .and_then(|peer| peer.status.as_connecting()?.as_outgoing())
                    .map_or(false, |s| s.is_timed_out(state.time()));
                timed_out && self.is_enabled(&state.p2p)
            }
            _ => self.is_enabled(&state.p2p),
        }
    }
}
