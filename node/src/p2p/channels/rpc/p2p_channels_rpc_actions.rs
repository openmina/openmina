use super::*;

impl redux::EnablingCondition<crate::State> for P2pChannelsRpcAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        match self {
            Self::Timeout { peer_id, id } => {
                self.is_enabled(&state.p2p)
                    && state.p2p.is_peer_rpc_timed_out(peer_id, *id, state.time())
            }
            _ => self.is_enabled(&state.p2p),
        }
    }
}
