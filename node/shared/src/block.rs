use std::sync::Arc;

use mina_p2p_messages::v2::LedgerHash;
use serde::{Deserialize, Serialize};

pub use mina_p2p_messages::v2::MinaBlockBlockStableV2 as Block;
pub use mina_p2p_messages::v2::MinaBlockHeaderStableV2 as BlockHeader;
pub use mina_p2p_messages::v2::StateHash as BlockHash;

pub type ArcBlock = Arc<Block>;
pub type ArcBlockWithHash = BlockWithHash<Arc<Block>>;

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

impl<T: AsRef<Block>> BlockWithHash<T> {
    pub fn new(block: T) -> Self {
        Self {
            hash: block.as_ref().hash(),
            block,
        }
    }

    pub fn header(&self) -> &BlockHeader {
        &self.block.as_ref().header
    }

    pub fn snarked_ledger_hash(&self) -> LedgerHash {
        snarked_ledger_hash(self.header())
    }
}

impl<T: AsRef<BlockHeader>> BlockHeaderWithHash<T> {
    pub fn new(header: T) -> Self {
        Self {
            hash: header.as_ref().hash(),
            header,
        }
    }

    pub fn snarked_ledger_hash(&self) -> LedgerHash {
        snarked_ledger_hash(self.header.as_ref())
    }
}

fn snarked_ledger_hash(header: &BlockHeader) -> LedgerHash {
    header
        .protocol_state
        .body
        .blockchain_state
        .ledger_proof_statement
        .target
        .first_pass_ledger
        .clone()
}
