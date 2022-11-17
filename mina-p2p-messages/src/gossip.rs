use binprot_derive::{BinProtRead, BinProtWrite};
use derive_more::{From, TryInto};
use serde::{Deserialize, Serialize};

use crate::{v1, v2};

#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite, PartialEq)]
#[serde(tag = "type", content = "message", rename_all = "snake_case")]
pub enum GossipNetMessageV1 {
    NewState(v1::MinaBlockExternalTransitionRawVersionedStableV1Versioned),
    SnarkPoolDiff(v1::NetworkPoolSnarkPoolDiffVersionedStableV1Versioned),
    TransactionPoolDiff(v1::NetworkPoolTransactionPoolDiffVersionedStableV1Versioned),
}

#[derive(Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite, PartialEq, From, TryInto)]
#[serde(tag = "type", content = "message", rename_all = "snake_case")]
pub enum GossipNetMessageV2 {
    NewState(v2::MinaBlockBlockStableV2),
    SnarkPoolDiff(v2::NetworkPoolSnarkPoolDiffVersionedStableV2),
    TransactionPoolDiff(v2::NetworkPoolTransactionPoolDiffVersionedStableV2),
}
