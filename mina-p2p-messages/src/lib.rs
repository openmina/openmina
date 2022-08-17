use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Serialize, Deserialize};

pub mod p2p;
pub mod versioned;
pub mod bigint;
pub mod phantom;
pub mod char_;
pub mod string;

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
