use super::{RpcAction, RpcActionWithMetaRef, RpcState};

impl RpcState {
    pub fn reducer(&mut self, action: RpcActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            RpcAction::GlobalStateGet(_) => {}
            RpcAction::Finish(action) => {
                self.requests.remove(&action.rpc_id);
            }
        }
    }
}
