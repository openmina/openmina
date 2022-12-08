use serde::{Deserialize, Serialize};

pub use mina_p2p_messages::v1::StateHashStable as BlockHash;
pub use mina_p2p_messages::v2::MinaBlockBlockStableV2 as Block;
pub use mina_p2p_messages::v2::MinaBlockHeaderStableV2 as BlockHeader;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockWithHash {
    pub hash: BlockHash,
    pub block: Block,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeaderWithHash {
    pub hash: BlockHash,
    pub header: BlockHeader,
}
