use mina_p2p_messages::v2::{
    ConsensusProofOfStakeDataConsensusStateValueStableV2, LedgerHash,
    MinaBaseProtocolConstantsCheckedValueStableV1, MinaBaseStagedLedgerHashStableV1,
    NonZeroCurvePoint, StagedLedgerDiffBodyStableV1, StagedLedgerDiffDiffDiffStableV2,
    StagedLedgerDiffDiffFtStableV1, StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B,
    TransactionSnarkWorkTStableV2,
};
use redux::Timestamp;
use serde::{Deserialize, Serialize};

use crate::constants::constraint_constants;

use super::{Block, BlockHash, BlockHeader};

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

    pub fn hash(&self) -> &BlockHash {
        &self.hash
    }

    pub fn pred_hash(&self) -> &BlockHash {
        &self.header().protocol_state.previous_state_hash
    }

    pub fn header(&self) -> &BlockHeader {
        &self.block.as_ref().header
    }

    pub fn body(&self) -> &StagedLedgerDiffBodyStableV1 {
        &self.block.as_ref().body
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

    pub fn genesis_ledger_hash(&self) -> &LedgerHash {
        genesis_ledger_hash(self.header())
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

    pub fn is_genesis(&self) -> bool {
        self.height() == 1
            || constraint_constants()
                .fork
                .as_ref()
                .map_or(false, |fork| fork.blockchain_length + 1 == self.height())
    }

    pub fn root_block_height(&self) -> u32 {
        let k = self.constants().k.as_u32();
        self.height().saturating_sub(k).max(1)
    }

    pub fn staged_ledger_diff(&self) -> &StagedLedgerDiffDiffDiffStableV2 {
        self.body().diff()
    }

    pub fn commands_iter<'a>(
        &'a self,
    ) -> Box<dyn 'a + Iterator<Item = &'a StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B>>
    {
        self.body().commands_iter()
    }

    pub fn coinbases_iter(&self) -> impl Iterator<Item = &StagedLedgerDiffDiffFtStableV1> {
        self.body().coinbases_iter()
    }

    pub fn completed_works_iter<'a>(
        &'a self,
    ) -> Box<dyn 'a + Iterator<Item = &'a TransactionSnarkWorkTStableV2>> {
        self.body().completed_works_iter()
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
        self.header.as_ref()
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

    pub fn genesis_ledger_hash(&self) -> &LedgerHash {
        genesis_ledger_hash(self.header())
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

fn genesis_ledger_hash(header: &BlockHeader) -> &LedgerHash {
    &header
        .protocol_state
        .body
        .blockchain_state
        .genesis_ledger_hash
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
