use std::collections::{BTreeMap, VecDeque};

use itertools::Itertools;
use mina_p2p_messages::binprot::BinProtWrite;
use mina_p2p_messages::v2::{
    ConsensusBodyReferenceStableV1, MinaBlockBlockStableV2, StagedLedgerDiffDiffStableV2,
};

const BODY_TAG: u8 = 0;
const MAX_BLOCK_SIZE: usize = 262144;
const LINK_SIZE: usize = 32;
const ABSOLUTE_MAX_LINKS_PER_BLOCK: usize = u16::MAX as usize;

type Link = Box<[u8; LINK_SIZE]>;

#[derive(Debug)]
pub enum BlockBodyValidationError {
    HashMismatch {
        expected_from_header: String,
        got: String,
    },
    InvalidBlockProduced,
    InvalidState,
}

pub fn block_body_hash(
    body: &StagedLedgerDiffDiffStableV2,
) -> Result<ConsensusBodyReferenceStableV1, BlockBodyValidationError> {
    let bytes = serialize_with_len_and_tag(body);
    blocks_of_data(MAX_BLOCK_SIZE, &bytes)
        .map(|(_, hash)| hash)
        .map(|hash| hash.as_slice().into())
        .map(ConsensusBodyReferenceStableV1)
}

pub fn validate_block(block: &MinaBlockBlockStableV2) -> Result<(), BlockBodyValidationError> {
    let calculated = block_body_hash(&block.body.staged_ledger_diff)?;

    let expected = &block
        .header
        .protocol_state
        .body
        .blockchain_state
        .body_reference;

    if &calculated == expected {
        Ok(())
    } else {
        let hex = |bytes: &[u8]| bytes.iter().map(|b| format!("{:x}", b)).join("");

        Err(BlockBodyValidationError::HashMismatch {
            expected_from_header: hex(expected),
            got: hex(&calculated),
        })
    }
}

fn serialize_with_len_and_tag(block: &StagedLedgerDiffDiffStableV2) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(32 * 1024);
    block.binprot_write(&mut bytes).unwrap();
    let len = bytes.len();

    let mut bytes_with_header = Vec::with_capacity(len + 5);
    bytes_with_header.extend(((len + 1) as u32).to_le_bytes());
    bytes_with_header.extend(BODY_TAG.to_ne_bytes());
    bytes_with_header.append(&mut bytes);
    bytes_with_header
}

fn blake2(data: &[u8]) -> Link {
    use blake2::digest::{Update, VariableOutput};
    use blake2::Blake2bVar;

    let mut hasher = Blake2bVar::new(LINK_SIZE).unwrap();
    hasher.update(data);
    hasher.finalize_boxed().try_into().unwrap()
}

/// https://github.com/MinaProtocol/mina/blob/850309dad6293c3b7b15ef682d38e1e26c1d2e13/src/lib/staged_ledger_diff/bitswap_block.ml#L78
fn blocks_of_data(
    max_block_size: usize,
    data: &[u8],
) -> Result<(BTreeMap<Link, Vec<u8>>, Link), BlockBodyValidationError> {
    if max_block_size <= 2 + LINK_SIZE {
        panic!("Max block size too small");
    }
    let max_data_chunk_size = max_block_size - 2;
    let data_length = data.len();
    let schema = create_schema(max_block_size, data_length);

    let mut remaining_data = data_length;
    let mut blocks = BTreeMap::<Link, Vec<u8>>::default();
    let mut link_queue = VecDeque::<Link>::with_capacity(128);

    let mut dequeue_chunk = |chunk_size: usize| {
        assert!(!remaining_data >= chunk_size);
        let pos = remaining_data - chunk_size;
        let chunk = data.get(pos..pos + chunk_size).unwrap();
        remaining_data -= chunk_size;
        chunk
    };

    let dequeue_links = |num_links: usize, link_queue: &mut VecDeque<Link>| {
        assert!(link_queue.len() >= num_links);
        let mut links = Vec::with_capacity(num_links);
        for _ in 1..=num_links {
            let front = link_queue.pop_front().unwrap();
            links.push(front);
        }
        links.reverse();
        links
    };

    let mut create_block =
        |links: Vec<Link>, chunk_size: usize, link_queue: &mut VecDeque<Link>| {
            let chunk = dequeue_chunk(chunk_size);
            let num_links = links.len();
            let size = 2 + (num_links * LINK_SIZE) + chunk_size;
            if num_links > ABSOLUTE_MAX_LINKS_PER_BLOCK || size > max_block_size {
                return Err(BlockBodyValidationError::InvalidBlockProduced);
            }

            let mut block = Vec::with_capacity(size);
            block.extend((num_links as u16).to_le_bytes());
            for link in links.iter() {
                let link: &[u8; LINK_SIZE] = link;
                block.extend(link);
            }
            block.extend(chunk);

            let hash = blake2(&block);
            blocks.insert(hash.clone(), block);
            link_queue.push_back(hash);
            Ok(())
        };

    // create the last block
    create_block(vec![], schema.last_leaf_block_data_size, &mut link_queue)?;

    if schema.num_total_blocks > 1 {
        // create the data-only blocks
        let num_data_only_blocks = schema.num_total_blocks
            - schema.num_full_branch_blocks
            - 1
            - if schema.num_links_in_partial_branch_block > 0 {
                1
            } else {
                0
            };
        for _ in 1..=num_data_only_blocks {
            create_block(vec![], max_data_chunk_size, &mut link_queue)?;
        }
        // create the non max link block, if there is one
        if schema.num_links_in_partial_branch_block > 0 {
            let chunk_size =
                max_block_size - 2 - (schema.num_links_in_partial_branch_block * LINK_SIZE);
            let link = dequeue_links(schema.num_links_in_partial_branch_block, &mut link_queue);
            create_block(link, chunk_size, &mut link_queue)?;
        }

        // create the max link blocks
        let full_link_chunk_size = max_block_size - 2 - (schema.max_links_per_block * LINK_SIZE);

        for _ in 1..=schema.num_full_branch_blocks {
            create_block(
                dequeue_links(schema.max_links_per_block, &mut link_queue),
                full_link_chunk_size,
                &mut link_queue,
            )?;
        }
    }
    if remaining_data != 0 {
        return Err(BlockBodyValidationError::InvalidState);
    }
    if link_queue.len() != 1 {
        return Err(BlockBodyValidationError::InvalidState);
    }

    Ok((blocks, link_queue.pop_back().unwrap()))
}

fn required_bitswap_block_count(max_block_size: usize, data_length: usize) -> usize {
    if data_length <= max_block_size - 2 {
        1
    } else {
        let n1 = data_length - LINK_SIZE;
        let n2 = max_block_size - LINK_SIZE - 2;
        (n1 + n2 - 1) / n2
    }
}

fn max_links_per_block(max_block_size: usize) -> usize {
    let links_per_block = (max_block_size - 2) / LINK_SIZE;
    links_per_block.min(ABSOLUTE_MAX_LINKS_PER_BLOCK)
}

#[derive(Debug)]
struct Schema {
    num_total_blocks: usize,
    num_full_branch_blocks: usize,
    last_leaf_block_data_size: usize,
    num_links_in_partial_branch_block: usize,
    max_block_data_size: usize,
    max_links_per_block: usize,
}

fn create_schema(max_block_size: usize, data_length: usize) -> Schema {
    let num_total_blocks = required_bitswap_block_count(max_block_size, data_length);
    let last_leaf_block_data_size =
        data_length - ((max_block_size - LINK_SIZE - 2) * (num_total_blocks - 1));
    let max_links_per_block = max_links_per_block(max_block_size);
    let num_full_branch_blocks = (num_total_blocks - 1) / max_links_per_block;
    let num_links_in_partial_branch_block =
        num_total_blocks - 1 - (num_full_branch_blocks * max_links_per_block);

    Schema {
        num_total_blocks,
        num_full_branch_blocks,
        last_leaf_block_data_size,
        num_links_in_partial_branch_block,
        max_block_data_size: max_block_size,
        max_links_per_block,
    }
}
