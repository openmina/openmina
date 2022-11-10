use super::{
    P2pConnectionOutgoingAction, P2pConnectionOutgoingActionWithMetaRef, P2pConnectionOutgoingState,
};

impl P2pConnectionOutgoingState {
    pub fn reducer(&mut self, action: P2pConnectionOutgoingActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            P2pConnectionOutgoingAction::Init(content) => {
                *self = Self::Init {
                    time: meta.time(),
                    addrs: content.opts.addrs.clone(),
                    rpc_id: content.rpc_id,
                };
            }
            P2pConnectionOutgoingAction::Pending(_) => {
                let rpc_id = self.rpc_id();
                *self = Self::Pending {
                    time: meta.time(),
                    rpc_id,
                };
            }
            P2pConnectionOutgoingAction::Error(content) => {
                let rpc_id = self.rpc_id();
                *self = Self::Error {
                    time: meta.time(),
                    error: content.error.clone(),
                    rpc_id,
                };
            }
            P2pConnectionOutgoingAction::Success(_) => {
                let rpc_id = self.rpc_id();
                *self = Self::Success {
                    time: meta.time(),
                    rpc_id,
                };
            }
        }
    }
}
