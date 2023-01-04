use mina_p2p_messages::v2::NonZeroCurvePoint;
use serde::{Deserialize, Serialize};

use p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
use p2p::pubsub::{GossipNetMessageV2, PubsubTopic};

use super::RpcId;

pub type RpcActionWithMeta = redux::ActionWithMeta<RpcAction>;
pub type RpcActionWithMetaRef<'a> = redux::ActionWithMeta<&'a RpcAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum RpcAction {
    GlobalStateGet(RpcGlobalStateGetAction),

    P2pConnectionOutgoingInit(RpcP2pConnectionOutgoingInitAction),
    P2pConnectionOutgoingPending(RpcP2pConnectionOutgoingPendingAction),
    P2pConnectionOutgoingError(RpcP2pConnectionOutgoingErrorAction),
    P2pConnectionOutgoingSuccess(RpcP2pConnectionOutgoingSuccessAction),

    P2pPubsubMessagePublish(RpcP2pPubsubMessagePublishAction),

    WatchedAccountsAdd(RpcWatchedAccountsAddAction),
    WatchedAccountsGet(RpcWatchedAccountsGetAction),

    Finish(RpcFinishAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcGlobalStateGetAction {
    pub rpc_id: RpcId,
}

impl redux::EnablingCondition<crate::State> for RpcGlobalStateGetAction {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcP2pConnectionOutgoingInitAction {
    pub rpc_id: RpcId,
    pub opts: P2pConnectionOutgoingInitOpts,
}

impl redux::EnablingCondition<crate::State> for RpcP2pConnectionOutgoingInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        !state.rpc.requests.contains_key(&self.rpc_id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcP2pConnectionOutgoingPendingAction {
    pub rpc_id: RpcId,
}

impl redux::EnablingCondition<crate::State> for RpcP2pConnectionOutgoingPendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .rpc
            .requests
            .get(&self.rpc_id)
            .map_or(false, |v| v.status.is_init())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcP2pConnectionOutgoingErrorAction {
    pub rpc_id: RpcId,
    pub error: String,
}

impl redux::EnablingCondition<crate::State> for RpcP2pConnectionOutgoingErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .rpc
            .requests
            .get(&self.rpc_id)
            .map_or(false, |v| v.status.is_pending())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcP2pConnectionOutgoingSuccessAction {
    pub rpc_id: RpcId,
}

impl redux::EnablingCondition<crate::State> for RpcP2pConnectionOutgoingSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .rpc
            .requests
            .get(&self.rpc_id)
            .map_or(false, |v| v.status.is_pending())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcP2pPubsubMessagePublishAction {
    pub rpc_id: RpcId,
    pub topic: PubsubTopic,
    pub message: GossipNetMessageV2,
}

impl redux::EnablingCondition<crate::State> for RpcP2pPubsubMessagePublishAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        true
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcWatchedAccountsAddAction {
    pub rpc_id: RpcId,
    pub pub_key: NonZeroCurvePoint,
}

impl redux::EnablingCondition<crate::State> for RpcWatchedAccountsAddAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        true
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcWatchedAccountsGetAction {
    pub rpc_id: RpcId,
    pub pub_key: NonZeroCurvePoint,
}

impl redux::EnablingCondition<crate::State> for RpcWatchedAccountsGetAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        true
    }
}

/// Finish/Cleanup rpc request.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcFinishAction {
    pub rpc_id: RpcId,
}

impl redux::EnablingCondition<crate::State> for RpcFinishAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        state
            .rpc
            .requests
            .get(&self.rpc_id)
            .map_or(false, |v| v.status.is_finished())
    }
}

macro_rules! impl_into_global_action {
    ($a:ty) => {
        impl From<$a> for crate::Action {
            fn from(value: $a) -> Self {
                Self::Rpc(value.into())
            }
        }
    };
}

impl_into_global_action!(RpcGlobalStateGetAction);

impl_into_global_action!(RpcP2pConnectionOutgoingInitAction);
impl_into_global_action!(RpcP2pConnectionOutgoingPendingAction);
impl_into_global_action!(RpcP2pConnectionOutgoingErrorAction);
impl_into_global_action!(RpcP2pConnectionOutgoingSuccessAction);

impl_into_global_action!(RpcP2pPubsubMessagePublishAction);

impl_into_global_action!(RpcWatchedAccountsAddAction);
impl_into_global_action!(RpcWatchedAccountsGetAction);

impl_into_global_action!(RpcFinishAction);
