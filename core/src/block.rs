use std::sync::Arc;

use mina_p2p_messages::v2::{
    LedgerHash, MinaBaseStagedLedgerHashStableV1, StagedLedgerDiffDiffFtStableV1,
    StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2Coinbase,
};
use mina_p2p_messages::v2::{
    StagedLedgerDiffDiffDiffStableV2,
    StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2Coinbase,
    StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B, TransactionSnarkWorkTStableV2,
};
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

    pub fn staking_epoch_ledger_hash(&self) -> &LedgerHash {
        staking_epoch_ledger_hash(self.header())
    }

    pub fn next_epoch_ledger_hash(&self) -> &LedgerHash {
        next_epoch_ledger_hash(self.header())
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

    pub fn staged_ledger_diff(&self) -> &StagedLedgerDiffDiffDiffStableV2 {
        &self.block.as_ref().body.staged_ledger_diff.diff
    }

    pub fn commands_iter(
        &self,
    ) -> impl Iterator<Item = &StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B> {
        let diff = self.staged_ledger_diff();
        diff.0.commands.iter().chain(match &diff.1.as_ref() {
            None => &[],
            Some(v) => &v.commands[..],
        })
    }

    pub fn coinbases_iter(&self) -> impl Iterator<Item = &StagedLedgerDiffDiffFtStableV1> {
        let diff = self.staged_ledger_diff();
        let mut coinbases = Vec::with_capacity(4);
        match &diff.0.coinbase {
            StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2Coinbase::Zero => {}
            StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2Coinbase::One(v) => {
                coinbases.push(v.as_ref());
            }
            StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2Coinbase::Two(v) => {
                match v.as_ref() {
                    None => {}
                    Some((v1, v2)) => {
                        coinbases.push(Some(v1));
                        coinbases.push(v2.as_ref());
                    }
                }
            }
        }
        match diff.1.as_ref() {
            Some(v) => match &v.coinbase {
                StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2Coinbase::Zero => {}
                StagedLedgerDiffDiffPreDiffWithAtMostOneCoinbaseStableV2Coinbase::One(v) => {
                    coinbases.push(v.as_ref());
                }
            },
            _ => {}
        }
        coinbases.into_iter().filter_map(|v| v)
    }

    pub fn completed_works_iter(&self) -> impl Iterator<Item = &TransactionSnarkWorkTStableV2> {
        let diff = self.staged_ledger_diff();
        diff.0.completed_works.iter().chain(match &diff.1.as_ref() {
            None => &[],
            Some(v) => &v.completed_works[..],
        })
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

    pub fn staking_epoch_ledger_hash(&self) -> &LedgerHash {
        staking_epoch_ledger_hash(self.header())
    }

    pub fn next_epoch_ledger_hash(&self) -> &LedgerHash {
        next_epoch_ledger_hash(self.header())
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
    // FIXME: this calculation must use values from the protocol constants,
    // now it assumes 3 minutes blocks.
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

fn staking_epoch_ledger_hash(header: &BlockHeader) -> &LedgerHash {
    &header
        .protocol_state
        .body
        .consensus_state
        .staking_epoch_data
        .ledger
        .hash
}

fn next_epoch_ledger_hash(header: &BlockHeader) -> &LedgerHash {
    &header
        .protocol_state
        .body
        .consensus_state
        .next_epoch_data
        .ledger
        .hash
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
