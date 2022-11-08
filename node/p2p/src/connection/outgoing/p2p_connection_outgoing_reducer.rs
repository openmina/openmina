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
                };
            }
            P2pConnectionOutgoingAction::Pending(_) => {
                *self = Self::Pending { time: meta.time() };
            }
            P2pConnectionOutgoingAction::Error(content) => {
                *self = Self::Error {
                    time: meta.time(),
                    error: content.error.clone(),
                };
            }
            P2pConnectionOutgoingAction::Success(_) => {
                *self = Self::Success { time: meta.time() };
            }
        }
    }
}
