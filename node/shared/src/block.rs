use std::sync::Arc;

use mina_p2p_messages::v2::{LedgerHash, MinaBaseStagedLedgerHashStableV1};
use redux::Timestamp;
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

    pub fn hash(&self) -> &BlockHash {
        &self.hash
    }

    pub fn pred_hash(&self) -> &BlockHash {
        &self.header().protocol_state.previous_state_hash
    }

    pub fn height(&self) -> u32 {
        height(self.header())
    }

    pub fn global_slot(&self) -> u32 {
        global_slot(self.header())
    }

    pub fn timestamp(&self) -> Timestamp {
        timestamp(self.header())
    }

    pub fn snarked_ledger_hash(&self) -> &LedgerHash {
        snarked_ledger_hash(self.header())
    }

    pub fn staged_ledger_hash(&self) -> &LedgerHash {
        staged_ledger_hash(self.header())
    }

    pub fn staged_ledger_hashes(&self) -> &MinaBaseStagedLedgerHashStableV1 {
        staged_ledger_hashes(self.header())
    }

    pub fn root_block_height(&self) -> u32 {
        let k = self.header().protocol_state.body.constants.k.as_u32();
        self.height().saturating_sub(k).max(1)
    }
}

impl<T: AsRef<BlockHeader>> BlockHeaderWithHash<T> {
    pub fn new(header: T) -> Self {
        Self {
            hash: header.as_ref().hash(),
            header,
        }
    }

    pub fn header(&self) -> &BlockHeader {
        &self.header.as_ref()
    }

    pub fn hash(&self) -> &BlockHash {
        &self.hash
    }

    pub fn pred_hash(&self) -> &BlockHash {
        &self.header().protocol_state.previous_state_hash
    }

    pub fn height(&self) -> u32 {
        height(self.header())
    }

    pub fn global_slot(&self) -> u32 {
        global_slot(self.header())
    }

    pub fn timestamp(&self) -> Timestamp {
        timestamp(self.header())
    }

    pub fn snarked_ledger_hash(&self) -> &LedgerHash {
        snarked_ledger_hash(self.header())
    }

    pub fn staged_ledger_hash(&self) -> &LedgerHash {
        staged_ledger_hash(self.header())
    }

    pub fn staged_ledger_hashes(&self) -> &MinaBaseStagedLedgerHashStableV1 {
        staged_ledger_hashes(self.header())
    }
}

fn height(header: &BlockHeader) -> u32 {
    header
        .protocol_state
        .body
        .consensus_state
        .blockchain_length
        .0
        .as_u32()
}

fn global_slot(header: &BlockHeader) -> u32 {
    header
        .protocol_state
        .body
        .consensus_state
        .global_slot_since_genesis
        .as_u32()
}

fn timestamp(header: &BlockHeader) -> Timestamp {
    let genesis_timestamp = header
        .protocol_state
        .body
        .constants
        .genesis_state_timestamp
        .0
        .as_u64();
    let slot = global_slot(header) as u64;
    let time_ms = genesis_timestamp + slot * 3 * 60 * 1000;
    Timestamp::new(time_ms * 1_000_000)
}

fn snarked_ledger_hash(header: &BlockHeader) -> &LedgerHash {
    &header
        .protocol_state
        .body
        .blockchain_state
        .ledger_proof_statement
        .target
        .first_pass_ledger
}

fn staged_ledger_hash(header: &BlockHeader) -> &LedgerHash {
    &staged_ledger_hashes(header).non_snark.ledger_hash
}

fn staged_ledger_hashes(header: &BlockHeader) -> &MinaBaseStagedLedgerHashStableV1 {
    &header
        .protocol_state
        .body
        .blockchain_state
        .staged_ledger_hash
}
