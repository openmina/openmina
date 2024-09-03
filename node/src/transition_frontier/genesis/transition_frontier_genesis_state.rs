use ledger::dummy::dummy_blockchain_proof;
use mina_p2p_messages::v2;
use openmina_core::{block::ArcBlockWithHash, constants::PROTOCOL_VERSION};
use serde::{Deserialize, Serialize};

use super::{empty_block_body, GenesisConfigLoaded};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransitionFrontierGenesisState {
    Idle,
    LedgerLoadPending {
        time: redux::Timestamp,
    },
    LedgerLoadSuccess {
        time: redux::Timestamp,
        data: GenesisConfigLoaded,
    },
    Produced {
        time: redux::Timestamp,
        negative_one: v2::MinaStateProtocolStateValueStableV2,
        genesis: v2::MinaStateProtocolStateValueStableV2,
        genesis_hash: v2::StateHash,
        genesis_producer_stake_proof: v2::MinaBaseSparseLedgerBaseStableV2,
    },
    ProvePending {
        time: redux::Timestamp,
        negative_one: v2::MinaStateProtocolStateValueStableV2,
        genesis: v2::MinaStateProtocolStateValueStableV2,
        genesis_hash: v2::StateHash,
        genesis_producer_stake_proof: v2::MinaBaseSparseLedgerBaseStableV2,
    },
    ProveSuccess {
        time: redux::Timestamp,
        genesis: ArcBlockWithHash,
    },
}

impl TransitionFrontierGenesisState {
    pub fn block_with_dummy_proof(&self) -> Option<ArcBlockWithHash> {
        let Self::Produced {
            genesis,
            genesis_hash,
            ..
        } = self
        else {
            return None;
        };
        Some(
            ArcBlockWithHash::try_new(
                v2::MinaBlockBlockStableV2 {
                    header: v2::MinaBlockHeaderStableV2 {
                        protocol_state: genesis.clone(),
                        protocol_state_proof: (*dummy_blockchain_proof()).clone(),
                        delta_block_chain_proof: (
                            genesis_hash.clone(),
                            std::iter::empty().collect(),
                        ),
                        current_protocol_version: PROTOCOL_VERSION.clone(),
                        proposed_protocol_version_opt: None,
                    },
                    body: v2::StagedLedgerDiffBodyStableV1 {
                        staged_ledger_diff: empty_block_body(),
                    },
                }
                .into(),
            )
            .ok()?,
        )
    }

    pub fn prove_pending_block_hash(&self) -> Option<v2::StateHash> {
        match self {
            Self::ProvePending { genesis_hash, .. } => Some(genesis_hash.clone()),
            _ => None,
        }
    }

    pub fn proven_block(&self) -> Option<&ArcBlockWithHash> {
        match self {
            Self::ProveSuccess { genesis, .. } => Some(genesis),
            _ => None,
        }
    }

    pub fn block_with_real_or_dummy_proof(&self) -> Option<ArcBlockWithHash> {
        self.proven_block()
            .cloned()
            .or_else(|| self.block_with_dummy_proof())
    }
}
