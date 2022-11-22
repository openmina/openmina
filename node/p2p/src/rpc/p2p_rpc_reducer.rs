use super::{P2pRpcAction, P2pRpcActionWithMetaRef, P2pRpcState};

impl P2pRpcState {
    pub fn reducer(&mut self, action: P2pRpcActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        todo!()
        // match action {}
    }
}
