use std::sync::Arc;

use mina_hasher::Fp;
use mina_p2p_messages::v2::{
    ConsensusProofOfStakeDataConsensusStateValueStableV2, LedgerHash,
    MinaBaseProtocolConstantsCheckedValueStableV1, MinaBaseStagedLedgerHashStableV1,
    MinaStateProtocolStateValueStableV2, NonZeroCurvePoint, StagedLedgerDiffDiffFtStableV1,
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
    pub fn new<F>(block: T, hash_fn: F) -> Self
    where
        F: Fn(&MinaStateProtocolStateValueStableV2) -> Fp,
    {
        Self {
            hash: BlockHash::from_fp(hash_fn(&block.as_ref().header.protocol_state)),
            block,
        }
    }

    pub fn hash(&self) -> &BlockHash {
        &self.hash
    }

    pub fn pred_hash(&self) -> &BlockHash {
        &self.header().protocol_state.previous_state_hash
    }

    pub fn header(&self) -> &BlockHeader {
        &self.block.as_ref().header
    }

    pub fn consensus_state(&self) -> &ConsensusProofOfStakeDataConsensusStateValueStableV2 {
        consensus_state(self.header())
    }

    pub fn height(&self) -> u32 {
        height(self.header())
    }

    pub fn global_slot(&self) -> u32 {
        global_slot(self.header())
    }

    pub fn global_slot_since_genesis(&self) -> u32 {
        global_slot_since_genesis(self.header())
    }

    pub fn global_slot_diff(&self) -> u32 {
        global_slot_diff(self.header())
    }

    pub fn timestamp(&self) -> Timestamp {
        timestamp(self.header())
    }

    pub fn genesis_timestamp(&self) -> Timestamp {
        genesis_timestamp(self.header())
    }

    pub fn constants(&self) -> &MinaBaseProtocolConstantsCheckedValueStableV1 {
        constants(self.header())
    }

    pub fn producer(&self) -> &NonZeroCurvePoint {
        producer(self.header())
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
        let k = self.constants().k.as_u32();
        self.height().saturating_sub(k).max(1)
    }

    pub fn staged_ledger_diff(&self) -> &StagedLedgerDiffDiffDiffStableV2 {
        &self.block.as_ref().body.staged_ledger_diff.diff
    }

    pub fn commands_iter<'a>(
        &'a self,
    ) -> Box<dyn 'a + Iterator<Item = &'a StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B>>
    {
        let diff = self.staged_ledger_diff();
        let iter = diff.0.commands.iter();
        if let Some(_1) = diff.1.as_ref() {
            Box::new(iter.chain(_1.commands.iter()))
        } else {
            Box::new(iter)
        }
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

    pub fn completed_works_iter<'a>(
        &'a self,
    ) -> Box<dyn 'a + Iterator<Item = &'a TransactionSnarkWorkTStableV2>> {
        let diff = self.staged_ledger_diff();
        let _0 = &diff.0;
        if let Some(_1) = diff.1.as_ref() {
            Box::new(_0.completed_works.iter().chain(_1.completed_works.iter()))
        } else {
            Box::new(_0.completed_works.iter())
        }
    }
}

impl<T: AsRef<BlockHeader>> BlockHeaderWithHash<T> {
    pub fn new(header: T) -> Self {
        Self {
            hash: header.as_ref().hash(),
            header,
        }
    }

    pub fn hash(&self) -> &BlockHash {
        &self.hash
    }

    pub fn pred_hash(&self) -> &BlockHash {
        &self.header().protocol_state.previous_state_hash
    }

    pub fn header(&self) -> &BlockHeader {
        &self.header.as_ref()
    }

    pub fn consensus_state(&self) -> &ConsensusProofOfStakeDataConsensusStateValueStableV2 {
        consensus_state(self.header())
    }

    pub fn height(&self) -> u32 {
        height(self.header())
    }

    pub fn global_slot(&self) -> u32 {
        global_slot(self.header())
    }

    pub fn global_slot_since_genesis(&self) -> u32 {
        global_slot_since_genesis(self.header())
    }

    pub fn global_slot_diff(&self) -> u32 {
        global_slot_diff(self.header())
    }

    pub fn timestamp(&self) -> Timestamp {
        timestamp(self.header())
    }

    pub fn genesis_timestamp(&self) -> Timestamp {
        genesis_timestamp(self.header())
    }

    pub fn constants(&self) -> &MinaBaseProtocolConstantsCheckedValueStableV1 {
        constants(self.header())
    }

    pub fn producer(&self) -> &NonZeroCurvePoint {
        producer(self.header())
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

fn consensus_state(header: &BlockHeader) -> &ConsensusProofOfStakeDataConsensusStateValueStableV2 {
    &header.protocol_state.body.consensus_state
}

fn height(header: &BlockHeader) -> u32 {
    consensus_state(header).blockchain_length.as_u32()
}

fn global_slot(header: &BlockHeader) -> u32 {
    consensus_state(header).global_slot()
}

fn global_slot_since_genesis(header: &BlockHeader) -> u32 {
    consensus_state(header).global_slot_since_genesis.as_u32()
}

fn global_slot_diff(header: &BlockHeader) -> u32 {
    let s = consensus_state(header);
    s.global_slot_since_genesis
        .as_u32()
        .saturating_sub(s.global_slot())
}

fn timestamp(header: &BlockHeader) -> Timestamp {
    let genesis_timestamp = constants(header).genesis_state_timestamp.0.as_u64();
    let slot = global_slot_since_genesis(header) as u64;
    // FIXME: this calculation must use values from the protocol constants,
    // now it assumes 3 minutes blocks.
    let time_ms = genesis_timestamp + slot * 3 * 60 * 1000;
    Timestamp::new(time_ms * 1_000_000)
}

fn genesis_timestamp(header: &BlockHeader) -> Timestamp {
    let genesis_timestamp = constants(header).genesis_state_timestamp.0.as_u64();
    Timestamp::new(genesis_timestamp * 1_000_000)
}

fn constants(header: &BlockHeader) -> &MinaBaseProtocolConstantsCheckedValueStableV1 {
    &header.protocol_state.body.constants
}

fn producer(header: &BlockHeader) -> &NonZeroCurvePoint {
    &header.protocol_state.body.consensus_state.block_creator
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
    &consensus_state(header).staking_epoch_data.ledger.hash
}

fn next_epoch_ledger_hash(header: &BlockHeader) -> &LedgerHash {
    &consensus_state(header).next_epoch_data.ledger.hash
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
