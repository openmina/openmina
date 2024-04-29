mod snark_block_verify_state;
pub use snark_block_verify_state::*;

mod snark_block_verify_actions;
pub use snark_block_verify_actions::*;

mod snark_block_verify_reducer;
pub use snark_block_verify_reducer::reducer;

pub use crate::block_verify_effectful::{
    SnarkBlockVerifyError, SnarkBlockVerifyId, SnarkBlockVerifyIdType,
};

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use openmina_core::block::{Block, BlockHash, BlockHeader, BlockHeaderWithHash, BlockWithHash};

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum VerifiableBlockWithHash {
    FullBox(BlockWithHash<Box<Block>>),
    FullArc(BlockWithHash<Arc<Block>>),
    HeaderBox(BlockHeaderWithHash<Box<BlockHeader>>),
    HeaderArc(BlockHeaderWithHash<Arc<BlockHeader>>),
}

impl VerifiableBlockWithHash {
    pub fn hash_ref(&self) -> &BlockHash {
        match self {
            Self::FullBox(v) => &v.hash,
            Self::FullArc(v) => &v.hash,
            Self::HeaderBox(v) => &v.hash,
            Self::HeaderArc(v) => &v.hash,
        }
    }

    pub fn full_ref(&self) -> Option<&Block> {
        match self {
            Self::FullBox(v) => Some(&v.block),
            Self::FullArc(v) => Some(&v.block),
            Self::HeaderBox(_) => None,
            Self::HeaderArc(_) => None,
        }
    }

    pub fn header_ref(&self) -> &BlockHeader {
        match self {
            Self::FullBox(v) => &v.block.header,
            Self::FullArc(v) => &v.block.header,
            Self::HeaderBox(v) => &v.header,
            Self::HeaderArc(v) => &v.header,
        }
    }
}

impl AsRef<BlockHeader> for VerifiableBlockWithHash {
    fn as_ref(&self) -> &BlockHeader {
        self.header_ref()
    }
}

impl From<(BlockHash, Box<Block>)> for VerifiableBlockWithHash {
    fn from((hash, block): (BlockHash, Box<Block>)) -> Self {
        BlockWithHash { hash, block }.into()
    }
}

impl From<(BlockHash, Arc<Block>)> for VerifiableBlockWithHash {
    fn from((hash, block): (BlockHash, Arc<Block>)) -> Self {
        BlockWithHash { hash, block }.into()
    }
}

impl From<(BlockHash, Box<BlockHeader>)> for VerifiableBlockWithHash {
    fn from((hash, header): (BlockHash, Box<BlockHeader>)) -> Self {
        BlockHeaderWithHash { hash, header }.into()
    }
}

impl From<(BlockHash, Arc<BlockHeader>)> for VerifiableBlockWithHash {
    fn from((hash, header): (BlockHash, Arc<BlockHeader>)) -> Self {
        BlockHeaderWithHash { hash, header }.into()
    }
}
