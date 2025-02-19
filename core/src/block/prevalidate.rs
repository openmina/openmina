use serde::{Deserialize, Serialize};

use crate::constants::PROTOCOL_VERSION;

use super::ArcBlockWithHash;

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

pub fn prevalidate_block(
    block: &ArcBlockWithHash,
    genesis: &ArcBlockWithHash,
    cur_global_slot: u32,
    allow_block_too_late: bool,
) -> Result<(), BlockPrevalidationError> {
    let block_global_slot = block.global_slot();

    let delta = genesis.constants().delta.as_u32();
    if cur_global_slot < block_global_slot {
        // Too_early
        return Err(BlockPrevalidationError::ReceivedTooEarly {
            current_global_slot: cur_global_slot,
            block_global_slot,
        });
    } else if !allow_block_too_late && cur_global_slot.saturating_sub(block_global_slot) > delta {
        // Too_late
        return Err(BlockPrevalidationError::ReceivedTooLate {
            current_global_slot: cur_global_slot,
            block_global_slot,
            delta,
        });
    }

    if block.header().genesis_state_hash() != genesis.hash() {
        return Err(BlockPrevalidationError::InvalidGenesisProtocolState);
    }

    let (protocol_versions_are_valid, protocol_version_matches_daemon) = {
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
        let compatible =
            v.transaction == PROTOCOL_VERSION.transaction && v.network == PROTOCOL_VERSION.network;

        (valid, compatible)
    };

    if !protocol_versions_are_valid {
        return Err(BlockPrevalidationError::InvalidProtocolVersion);
    } else if !protocol_version_matches_daemon {
        return Err(BlockPrevalidationError::MismatchedProtocolVersion);
    }

    // NOTE: currently these cannot change between blocks, but that
    // may not always be true?
    if block.constants() != genesis.constants() {
        return Err(BlockPrevalidationError::ConsantsMismatch);
    }

    // TODO(tizoc): check for InvalidDeltaBlockChainProof
    // https://github.com/MinaProtocol/mina/blob/d800da86a764d8d37ffb8964dd8d54d9f522b358/src/lib/mina_block/validation.ml#L369
    // https://github.com/MinaProtocol/mina/blob/d800da86a764d8d37ffb8964dd8d54d9f522b358/src/lib/transition_chain_verifier/transition_chain_verifier.ml

    Ok(())
}
