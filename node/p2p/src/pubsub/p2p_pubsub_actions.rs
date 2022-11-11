use serde::{Deserialize, Serialize};

use shared::requests::RpcId;

use super::{GossipNetMessageV1, PubsubTopic};

pub type P2pPubsubActionWithMeta = redux::ActionWithMeta<P2pPubsubAction>;
pub type P2pPubsubActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pPubsubAction>;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum P2pPubsubAction {
    MessagePublish(P2pPubsubMessagePublishAction),
    BytesPublish(P2pPubsubBytesPublishAction),

    BytesReceived(P2pPubsubBytesReceivedAction),
    MessageReceived(P2pPubsubMessageReceivedAction),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPubsubMessagePublishAction {
    pub topic: PubsubTopic,
    pub message: GossipNetMessageV1,
    pub rpc_id: Option<RpcId>,
}

impl redux::EnablingCondition<crate::P2pState> for P2pPubsubMessagePublishAction {
    fn is_enabled(&self, _: &crate::P2pState) -> bool {
        true
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPubsubBytesPublishAction {
    pub topic: PubsubTopic,
    pub bytes: Vec<u8>,
    pub rpc_id: Option<RpcId>,
}

impl redux::EnablingCondition<crate::P2pState> for P2pPubsubBytesPublishAction {
    fn is_enabled(&self, _: &crate::P2pState) -> bool {
        true
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPubsubBytesReceivedAction {
    pub author: crate::PeerId,
    pub sender: crate::PeerId,
    pub topic: PubsubTopic,
    pub bytes: Vec<u8>,
}

impl redux::EnablingCondition<crate::P2pState> for P2pPubsubBytesReceivedAction {
    fn is_enabled(&self, _: &crate::P2pState) -> bool {
        true
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct P2pPubsubMessageReceivedAction {
    pub author: crate::PeerId,
    pub sender: crate::PeerId,
    pub topic: PubsubTopic,
    pub message: GossipNetMessageV1,
}

impl redux::EnablingCondition<crate::P2pState> for P2pPubsubMessageReceivedAction {
    fn is_enabled(&self, _: &crate::P2pState) -> bool {
        true
    }
}

macro_rules! impl_into_p2p_action {
    ($a:ty) => {
        impl From<$a> for crate::P2pAction {
            fn from(value: $a) -> Self {
                Self::Pubsub(value.into())
            }
        }
    };
}

impl_into_p2p_action!(P2pPubsubMessagePublishAction);
impl_into_p2p_action!(P2pPubsubBytesPublishAction);
impl_into_p2p_action!(P2pPubsubBytesReceivedAction);
impl_into_p2p_action!(P2pPubsubMessageReceivedAction);
