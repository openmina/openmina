use serde::{Deserialize, Serialize};

use super::ArcBlockWithHash;
use crate::constants::PROTOCOL_VERSION;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BlockPrevalidationError {
    GenesisNotReady,
    ReceivedTooEarly {
        current_global_slot: u32,
        block_global_slot: u32,
    },
    ReceivedTooLate {
        current_global_slot: u32,
        block_global_slot: u32,
        delta: u32,
    },
    InvalidGenesisProtocolState,
    InvalidProtocolVersion,
    MismatchedProtocolVersion,
    ConsantsMismatch,
    InvalidDeltaBlockChainProof,
}

impl BlockPrevalidationError {
    pub fn is_forever_invalid(&self) -> bool {
        !matches!(self, Self::ReceivedTooEarly { .. })
    }
}

pub fn validate_block_timing(
    block: &ArcBlockWithHash,
    genesis: &ArcBlockWithHash,
    cur_global_slot: u32,
    allow_block_too_late: bool,
) -> Result<(), BlockPrevalidationError> {
    let block_global_slot = block.global_slot();
    let delta = genesis.constants().delta.as_u32();

    if cur_global_slot < block_global_slot {
        return Err(BlockPrevalidationError::ReceivedTooEarly {
            current_global_slot: cur_global_slot,
            block_global_slot,
        });
    } else if !allow_block_too_late && cur_global_slot.saturating_sub(block_global_slot) > delta {
        return Err(BlockPrevalidationError::ReceivedTooLate {
            current_global_slot: cur_global_slot,
            block_global_slot,
            delta,
        });
    }

    Ok(())
}

pub fn validate_genesis_state(
    block: &ArcBlockWithHash,
    genesis: &ArcBlockWithHash,
) -> Result<(), BlockPrevalidationError> {
    if block.header().genesis_state_hash() != genesis.hash() {
        return Err(BlockPrevalidationError::InvalidGenesisProtocolState);
    }
    Ok(())
}

pub fn validate_protocol_versions(block: &ArcBlockWithHash) -> Result<(), BlockPrevalidationError> {
    let min_transaction_version = 1.into();
    let v = &block.header().current_protocol_version;
    let nv = block
        .header()
        .proposed_protocol_version_opt
        .as_ref()
        .unwrap_or(v);

    // Our version values are unsigned, so there is no need to check that the
    // other parts are not negative.
    let valid =
        v.transaction >= min_transaction_version && nv.transaction >= min_transaction_version;
    if !valid {
        return Err(BlockPrevalidationError::InvalidProtocolVersion);
    }

    let compatible =
        v.transaction == PROTOCOL_VERSION.transaction && v.network == PROTOCOL_VERSION.network;
    if !compatible {
        return Err(BlockPrevalidationError::MismatchedProtocolVersion);
    }

    Ok(())
}

pub fn validate_constants(
    block: &ArcBlockWithHash,
    genesis: &ArcBlockWithHash,
) -> Result<(), BlockPrevalidationError> {
    // NOTE: currently these cannot change between blocks, but that
    // may not always be true?
    if block.constants() != genesis.constants() {
        return Err(BlockPrevalidationError::ConsantsMismatch);
    }
    Ok(())
}

pub fn prevalidate_block(
    block: &ArcBlockWithHash,
    genesis: &ArcBlockWithHash,
    cur_global_slot: u32,
    allow_block_too_late: bool,
) -> Result<(), BlockPrevalidationError> {
    validate_block_timing(block, genesis, cur_global_slot, allow_block_too_late)?;
    validate_genesis_state(block, genesis)?;
    validate_protocol_versions(block)?;
    validate_constants(block, genesis)?;

    // TODO(tizoc): check for InvalidDeltaBlockChainProof
    // https://github.com/MinaProtocol/mina/blob/d800da86a764d8d37ffb8964dd8d54d9f522b358/src/lib/mina_block/validation.ml#L369
    // https://github.com/MinaProtocol/mina/blob/d800da86a764d8d37ffb8964dd8d54d9f522b358/src/lib/transition_chain_verifier/transition_chain_verifier.ml

    Ok(())
}
