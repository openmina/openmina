use binprot_derive::{BinProtRead, BinProtWrite};
use derive_more::{From, TryInto};
use serde::{Deserialize, Serialize};

use crate::{number::Int32, v2};

#[derive(
    Clone, Debug, Serialize, Deserialize, BinProtRead, BinProtWrite, PartialEq, From, TryInto,
)]
#[serde(tag = "type", content = "message", rename_all = "snake_case")]
pub enum GossipNetMessageV2 {
    NewState(v2::MinaBlockBlockStableV2),
    SnarkPoolDiff {
        message: v2::NetworkPoolSnarkPoolDiffVersionedStableV2,
        nonce: Int32,
    },
    TransactionPoolDiff {
        message: v2::NetworkPoolTransactionPoolDiffVersionedStableV2,
        nonce: Int32,
    },
}

impl TryFrom<GossipNetMessageV2> for v2::NetworkPoolSnarkPoolDiffVersionedStableV2 {
    type Error = ();

    fn try_from(value: GossipNetMessageV2) -> Result<Self, Self::Error> {
        match value {
            GossipNetMessageV2::SnarkPoolDiff { message, .. } => Ok(message),
            _ => Err(()),
        }
    }
}

impl TryFrom<GossipNetMessageV2> for v2::NetworkPoolTransactionPoolDiffVersionedStableV2 {
    type Error = ();

    fn try_from(value: GossipNetMessageV2) -> Result<Self, Self::Error> {
        match value {
            GossipNetMessageV2::TransactionPoolDiff { message, .. } => Ok(message),
            _ => Err(()),
        }
    }
}
