use super::*;

impl redux::EnablingCondition<crate::State> for P2pConnectionIncomingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        match self {
            Self::Timeout { peer_id } => {
                let peer = state.p2p.peers.get(peer_id);
                let timed_out = peer
                    .and_then(|peer| peer.status.as_connecting()?.as_incoming())
                    .map_or(false, |s| s.is_timed_out(state.time()));
                timed_out && self.is_enabled(&state.p2p)
            }
            _ => self.is_enabled(&state.p2p),
        }
    }
}
