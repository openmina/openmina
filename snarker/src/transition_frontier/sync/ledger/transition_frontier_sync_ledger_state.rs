use mina_p2p_messages::v2::LedgerHash;
use redux::Timestamp;
use serde::{Deserialize, Serialize};
use shared::block::ArcBlockWithHash;

use super::snarked::TransitionFrontierSyncLedgerSnarkedState;
use super::staged::TransitionFrontierSyncLedgerStagedState;

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum TransitionFrontierSyncLedgerState {
    Init {
        time: Timestamp,
        block: ArcBlockWithHash,
    },
    #[from]
    Snarked(TransitionFrontierSyncLedgerSnarkedState),
    #[from]
    Staged(TransitionFrontierSyncLedgerStagedState),
    Success {
        time: Timestamp,
        block: ArcBlockWithHash,
    },
}

impl TransitionFrontierSyncLedgerState {
    pub fn snarked(&self) -> Option<&TransitionFrontierSyncLedgerSnarkedState> {
        match self {
            Self::Snarked(v) => Some(v),
            _ => None,
        }
    }

    pub fn staged(&self) -> Option<&TransitionFrontierSyncLedgerStagedState> {
        match self {
            Self::Staged(v) => Some(v),
            _ => None,
        }
    }

    pub fn is_snarked_ledger_synced(&self) -> bool {
        match self {
            Self::Init { .. } => false,
            Self::Snarked(TransitionFrontierSyncLedgerSnarkedState::Pending { .. }) => false,
            _ => true,
        }
    }

    pub fn block(&self) -> &ArcBlockWithHash {
        match self {
            Self::Init { block, .. } => block,
            Self::Snarked(s) => s.block(),
            Self::Staged(s) => s.block(),
            Self::Success { block, .. } => block,
        }
    }

    pub fn snarked_ledger_hash(&self) -> &LedgerHash {
        self.block().snarked_ledger_hash()
    }

    pub fn update_block(&mut self, time: Timestamp, new_block: ArcBlockWithHash) {
        match self {
            Self::Snarked(TransitionFrontierSyncLedgerSnarkedState::Pending { block, .. }) => {
                if block.snarked_ledger_hash() == new_block.snarked_ledger_hash() {
                    *block = new_block;
                } else {
                    *self = Self::Init {
                        time,
                        block: new_block.clone(),
                    };
                }
            }
            Self::Staged(staged) => {
                let block = staged.block();
                if block.snarked_ledger_hash() == new_block.snarked_ledger_hash() {
                    *self = TransitionFrontierSyncLedgerSnarkedState::Success {
                        time,
                        block: new_block.clone(),
                    }
                    .into();
                } else {
                    *self = Self::Init {
                        time,
                        block: new_block.clone(),
                    };
                }
            }
            _ => {
                // should be impossible.
            }
        }
    }
}
