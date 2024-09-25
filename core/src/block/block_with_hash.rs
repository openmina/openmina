use ark_ff::fields::arithmetic::InvalidBigInt;
use mina_p2p_messages::v2;
use redux::Timestamp;
use serde::{Deserialize, Serialize};

use crate::constants::constraint_constants;

use super::{Block, BlockHash, BlockHeader};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockWithHash<T: AsRef<Block>> {
    pub hash: BlockHash,
    pub block: T,
}

impl<T: AsRef<Block>> std::cmp::PartialEq for BlockWithHash<T> {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeaderWithHash<T: AsRef<BlockHeader>> {
    pub hash: BlockHash,
    pub header: T,
}

impl<T: AsRef<Block>> BlockWithHash<T> {
    pub fn try_new(block: T) -> Result<Self, InvalidBigInt> {
        Ok(Self {
            hash: block.as_ref().try_hash()?,
            block,
        })
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

    pub fn body(&self) -> &v2::StagedLedgerDiffBodyStableV1 {
        &self.block.as_ref().body
    }

    pub fn consensus_state(&self) -> &v2::ConsensusProofOfStakeDataConsensusStateValueStableV2 {
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

    pub fn curr_global_slot_since_hard_fork(&self) -> &v2::ConsensusGlobalSlotStableV1 {
        curr_global_slot_since_hard_fork(self.header())
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

    pub fn constants(&self) -> &v2::MinaBaseProtocolConstantsCheckedValueStableV1 {
        constants(self.header())
    }

    pub fn producer(&self) -> &v2::NonZeroCurvePoint {
        producer(self.header())
    }

    pub fn genesis_ledger_hash(&self) -> &v2::LedgerHash {
        genesis_ledger_hash(self.header())
    }

    pub fn snarked_ledger_hash(&self) -> &v2::LedgerHash {
        snarked_ledger_hash(self.header())
    }

    pub fn staking_epoch_ledger_hash(&self) -> &v2::LedgerHash {
        staking_epoch_ledger_hash(self.header())
    }

    pub fn next_epoch_ledger_hash(&self) -> &v2::LedgerHash {
        next_epoch_ledger_hash(self.header())
    }

    pub fn merkle_root_hash(&self) -> &v2::LedgerHash {
        merkle_root_hash(self.header())
    }

    pub fn staged_ledger_hashes(&self) -> &v2::MinaBaseStagedLedgerHashStableV1 {
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

    pub fn staged_ledger_diff(&self) -> &v2::StagedLedgerDiffDiffDiffStableV2 {
        self.body().diff()
    }

    pub fn commands_iter<'a>(
        &'a self,
    ) -> Box<
        dyn 'a + Iterator<Item = &'a v2::StagedLedgerDiffDiffPreDiffWithAtMostTwoCoinbaseStableV2B>,
    > {
        self.body().commands_iter()
    }

    pub fn coinbases_iter(&self) -> impl Iterator<Item = &v2::StagedLedgerDiffDiffFtStableV1> {
        self.body().coinbases_iter()
    }

    pub fn completed_works_iter<'a>(
        &'a self,
    ) -> Box<dyn 'a + Iterator<Item = &'a v2::TransactionSnarkWorkTStableV2>> {
        self.body().completed_works_iter()
    }
}

impl<T: AsRef<BlockHeader>> BlockHeaderWithHash<T> {
    pub fn hash(&self) -> &BlockHash {
        &self.hash
    }

    pub fn pred_hash(&self) -> &BlockHash {
        &self.header().protocol_state.previous_state_hash
    }

    pub fn header(&self) -> &BlockHeader {
        self.header.as_ref()
    }

    pub fn consensus_state(&self) -> &v2::ConsensusProofOfStakeDataConsensusStateValueStableV2 {
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

    pub fn curr_global_slot_since_hard_fork(&self) -> &v2::ConsensusGlobalSlotStableV1 {
        curr_global_slot_since_hard_fork(self.header())
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

    pub fn constants(&self) -> &v2::MinaBaseProtocolConstantsCheckedValueStableV1 {
        constants(self.header())
    }

    pub fn producer(&self) -> &v2::NonZeroCurvePoint {
        producer(self.header())
    }

    pub fn genesis_ledger_hash(&self) -> &v2::LedgerHash {
        genesis_ledger_hash(self.header())
    }

    pub fn snarked_ledger_hash(&self) -> &v2::LedgerHash {
        snarked_ledger_hash(self.header())
    }

    pub fn staking_epoch_ledger_hash(&self) -> &v2::LedgerHash {
        staking_epoch_ledger_hash(self.header())
    }

    pub fn next_epoch_ledger_hash(&self) -> &v2::LedgerHash {
        next_epoch_ledger_hash(self.header())
    }

    pub fn merkle_root_hash(&self) -> &v2::LedgerHash {
        merkle_root_hash(self.header())
    }

    pub fn staged_ledger_hashes(&self) -> &v2::MinaBaseStagedLedgerHashStableV1 {
        staged_ledger_hashes(self.header())
    }
}

fn consensus_state(
    header: &BlockHeader,
) -> &v2::ConsensusProofOfStakeDataConsensusStateValueStableV2 {
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

fn curr_global_slot_since_hard_fork(header: &BlockHeader) -> &v2::ConsensusGlobalSlotStableV1 {
    &consensus_state(header).curr_global_slot_since_hard_fork
}

fn global_slot_diff(header: &BlockHeader) -> u32 {
    let s = consensus_state(header);
    s.global_slot_since_genesis
        .as_u32()
        .saturating_sub(s.global_slot())
}

fn timestamp(header: &BlockHeader) -> Timestamp {
    let ms = header
        .protocol_state
        .body
        .blockchain_state
        .timestamp
        .as_u64();
    Timestamp::new(ms * 1_000_000)
}

fn genesis_timestamp(header: &BlockHeader) -> Timestamp {
    let genesis_timestamp = constants(header).genesis_state_timestamp.0.as_u64();
    Timestamp::new(genesis_timestamp * 1_000_000)
}

fn constants(header: &BlockHeader) -> &v2::MinaBaseProtocolConstantsCheckedValueStableV1 {
    &header.protocol_state.body.constants
}

fn producer(header: &BlockHeader) -> &v2::NonZeroCurvePoint {
    &header.protocol_state.body.consensus_state.block_creator
}

fn genesis_ledger_hash(header: &BlockHeader) -> &v2::LedgerHash {
    &header
        .protocol_state
        .body
        .blockchain_state
        .genesis_ledger_hash
}

fn snarked_ledger_hash(header: &BlockHeader) -> &v2::LedgerHash {
    &header
        .protocol_state
        .body
        .blockchain_state
        .ledger_proof_statement
        .target
        .first_pass_ledger
}

fn staking_epoch_ledger_hash(header: &BlockHeader) -> &v2::LedgerHash {
    &consensus_state(header).staking_epoch_data.ledger.hash
}

fn next_epoch_ledger_hash(header: &BlockHeader) -> &v2::LedgerHash {
    &consensus_state(header).next_epoch_data.ledger.hash
}

fn merkle_root_hash(header: &BlockHeader) -> &v2::LedgerHash {
    &staged_ledger_hashes(header).non_snark.ledger_hash
}

fn staged_ledger_hashes(header: &BlockHeader) -> &v2::MinaBaseStagedLedgerHashStableV1 {
    &header
        .protocol_state
        .body
        .blockchain_state
        .staged_ledger_hash
}
