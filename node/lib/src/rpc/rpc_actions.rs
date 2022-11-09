use serde::{Deserialize, Serialize};

use super::RpcId;

pub type RpcActionWithMeta = redux::ActionWithMeta<RpcAction>;
pub type RpcActionWithMetaRef<'a> = redux::ActionWithMeta<&'a RpcAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum RpcAction {
    GlobalStateGet(RpcGlobalStateGetAction),

    Finish(RpcFinishAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcGlobalStateGetAction {
    pub rpc_id: RpcId,
}

impl redux::EnablingCondition<crate::State> for RpcGlobalStateGetAction {}

/// Finish/Cleanup rpc request.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcFinishAction {
    pub rpc_id: RpcId,
}

impl redux::EnablingCondition<crate::State> for RpcFinishAction {}

impl From<RpcGlobalStateGetAction> for crate::Action {
    fn from(value: RpcGlobalStateGetAction) -> Self {
        Self::Rpc(value.into())
    }
}

impl From<RpcFinishAction> for crate::Action {
    fn from(value: RpcFinishAction) -> Self {
        Self::Rpc(value.into())
    }
}
