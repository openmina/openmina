mod transaction_info;
pub use transaction_info::TransactionInfo;

mod transaction_with_hash;
pub use transaction_with_hash::*;

pub use mina_p2p_messages::v2::{MinaBaseUserCommandStableV2 as Transaction, TransactionHash};

use crate::{p2p::P2pNetworkPubsubMessageCacheId, requests::RpcId};

/// TODO: Types and methods bellow, should be moved to `node` crate, they are here because they are used in `snark` crates callbacks
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, Default)]
pub enum TransactionPoolMessageSource {
    Rpc {
        id: RpcId,
    },
    Pubsub {
        id: P2pNetworkPubsubMessageCacheId,
    },
    #[default]
    None,
}

impl TransactionPoolMessageSource {
    pub fn rpc(id: RpcId) -> Self {
        Self::Rpc { id }
    }

    pub fn pubsub(id: P2pNetworkPubsubMessageCacheId) -> Self {
        Self::Pubsub { id }
    }

    pub fn is_sender_local(&self) -> bool {
        matches!(self, Self::Rpc { .. })
    }

    pub fn is_libp2p(&self) -> bool {
        matches!(self, Self::Pubsub { .. })
    }
}
