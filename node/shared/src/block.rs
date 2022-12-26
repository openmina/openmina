use serde::{Deserialize, Serialize};

pub use mina_p2p_messages::v2::MinaBlockBlockStableV2 as Block;
pub use mina_p2p_messages::v2::MinaBlockHeaderStableV2 as BlockHeader;
pub use mina_p2p_messages::v2::StateHash as BlockHash;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockWithHash<T: AsRef<Block>> {
    pub hash: BlockHash,
    pub block: T,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeaderWithHash<T: AsRef<BlockHeader>> {
    pub hash: BlockHash,
    pub header: T,
}
