use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

pub mod bigint;
pub mod char_;
pub mod phantom;
pub mod rpc;
pub mod string;
pub mod utils;
pub mod v1;
pub mod versioned;

#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[serde(tag = "type", content = "message")]
pub enum GossipNetMessage {
    #[serde(rename = "external_transition")]
    NewState(v1::MinaBlockExternalTransitionRawVersionedStableV1Binable),
    #[serde(rename = "snark_pool_diff")]
    SnarkPoolDiff(v1::NetworkPoolSnarkPoolDiffVersionedStableV1Binable),
    #[serde(rename = "transaction_pool_diff")]
    TransactionPoolDiff(v1::NetworkPoolTransactionPoolDiffVersionedStableV1Binable),
}
