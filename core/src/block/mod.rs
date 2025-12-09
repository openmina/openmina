mod applied_block;
mod block_with_hash;
pub mod genesis;
pub mod prevalidate;

pub use applied_block::AppliedBlock;
pub use block_with_hash::{BlockHeaderWithHash, BlockWithHash};
pub use mina_p2p_messages::v2::{
    MinaBlockBlockStableV2 as Block, MinaBlockHeaderStableV2 as BlockHeader, StateHash as BlockHash,
};

use std::sync::Arc;
pub type ArcBlock = Arc<Block>;
pub type ArcBlockWithHash = BlockWithHash<Arc<Block>>;
