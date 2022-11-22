use serde::{Deserialize, Serialize};

pub type P2pRpcActionWithMetaRef<'a> = redux::ActionWithMeta<&'a P2pRpcAction>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum P2pRpcAction {}

impl P2pRpcAction {
    pub fn peer_id(&self) -> &crate::PeerId {
        todo!()
        // match self {}
    }
}
