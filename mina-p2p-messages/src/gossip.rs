use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

use crate::{v1, v2};

#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[serde(tag = "type", content = "message")]
pub enum GossipNetMessageV1 {
    #[serde(rename = "external_transition")]
    NewState(v1::MinaBlockExternalTransitionRawVersionedStableV1Binable),
    #[serde(rename = "snark_pool_diff")]
    SnarkPoolDiff(v1::NetworkPoolSnarkPoolDiffVersionedStableV1Binable),
    #[serde(rename = "transaction_pool_diff")]
    TransactionPoolDiff(v1::NetworkPoolTransactionPoolDiffVersionedStableV1Binable),
}

#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite)]
#[serde(tag = "type", content = "message")]
pub enum GossipNetMessageV2 {
    #[serde(rename = "external_transition")]
    NewState(v2::MinaBlockBlockStableV2),
    #[serde(rename = "snark_pool_diff")]
    SnarkPoolDiff(v2::NetworkPoolSnarkPoolDiffVersionedStableV2),
    #[serde(rename = "transaction_pool_diff")]
    TransactionPoolDiff(v2::NetworkPoolTransactionPoolDiffVersionedStableV2),
}
