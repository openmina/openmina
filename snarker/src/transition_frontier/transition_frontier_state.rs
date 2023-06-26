use std::collections::VecDeque;

use mina_p2p_messages::v2::StateHash;
use redux::Timestamp;
use serde::{Deserialize, Serialize};
use shared::block::ArcBlockWithHash;

use super::{sync::ledger::TransitionFrontierSyncLedgerState, TransitionFrontierConfig};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransitionFrontierState {
    pub config: TransitionFrontierConfig,
    pub best_chain: VecDeque<ArcBlockWithHash>,
    pub sync: TransitionFrontierSyncState,
}

impl TransitionFrontierState {
    pub fn new(config: TransitionFrontierConfig) -> Self {
        let k = config.protocol_constants.k.0.as_u32() as usize;
        Self {
            config,
            // TODO(binier): add genesis_block as initial best_tip.
            best_chain: VecDeque::with_capacity(k),
            sync: TransitionFrontierSyncState::Idle,
        }
    }

    pub fn best_tip(&self) -> Option<&ArcBlockWithHash> {
        self.best_chain.back()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransitionFrontierSyncState {
    Idle,
    Init {
        time: Timestamp,
        best_tip: ArcBlockWithHash,
        root_block: ArcBlockWithHash,
        missing_blocks: Vec<StateHash>,
    },
    RootLedgerSyncPending {
        time: Timestamp,
        best_tip: ArcBlockWithHash,
        missing_blocks: Vec<StateHash>,
        root_ledger: TransitionFrontierSyncLedgerState,
    },
    RootLedgerSyncSuccess {
        time: Timestamp,
        best_tip: ArcBlockWithHash,
        root_block: ArcBlockWithHash,
        missing_blocks: Vec<StateHash>,
    },
    // BlocksFetchAndApplyPending {
    //     time: Timestamp,
    //     best_tip: ArcBlockWithHash,
    //     root_block: ArcBlockWithHash,
    //     missing_blocks: Vec<StateHash>,
    // },
    Synced {
        time: Timestamp,
    },
}

impl TransitionFrontierSyncState {
    pub fn root_block(&self) -> Option<&ArcBlockWithHash> {
        match self {
            Self::Idle => None,
            Self::Init { root_block, .. } => Some(root_block),
            Self::RootLedgerSyncPending { root_ledger, .. } => Some(root_ledger.block()),
            Self::RootLedgerSyncSuccess { root_block, .. } => Some(root_block),
            Self::Synced { .. } => None,
        }
    }

    pub fn root_ledger(&self) -> Option<&TransitionFrontierSyncLedgerState> {
        match self {
            Self::RootLedgerSyncPending { root_ledger, .. } => Some(root_ledger),
            _ => None,
        }
    }
}
