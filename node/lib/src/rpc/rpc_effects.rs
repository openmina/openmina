use crate::Store;

use super::{RpcAction, RpcActionWithMeta, RpcService};

pub fn rpc_effects<S: RpcService>(store: &mut Store<S>, action: RpcActionWithMeta) {
    let (action, meta) = action.split();

    match action {
        RpcAction::GlobalStateGet(action) => {
            store
                .service
                .respond_state_get(action.rpc_id, store.state.get());
        }
        RpcAction::Finish(_) => {}
    }
}
