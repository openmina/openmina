use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

pub mod bigint;
pub mod char_;
pub mod p2p;
pub mod phantom;
pub mod rpc;
pub mod string;
pub mod utils;
pub mod versioned;

#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[serde(tag = "type", content = "message")]
pub enum GossipNetMessage {
    #[serde(rename = "external_transition")]
    NewState(p2p::MinaBlockExternalTransitionRawVersionedStable),
    #[serde(rename = "snark_pool_diff")]
    SnarkPoolDiff(p2p::NetworkPoolSnarkPoolDiffVersionedStable),
    #[serde(rename = "transaction_pool_diff")]
    TransactionPoolDiff(p2p::NetworkPoolTransactionPoolDiffVersionedStable),
}
