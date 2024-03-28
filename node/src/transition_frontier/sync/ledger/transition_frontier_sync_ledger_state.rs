use std::collections::BTreeMap;

use mina_p2p_messages::v2::{MinaStateProtocolStateValueStableV2, StateHash};
use redux::Timestamp;
use serde::{Deserialize, Serialize};

use super::snarked::TransitionFrontierSyncLedgerSnarkedState;
use super::staged::TransitionFrontierSyncLedgerStagedState;
use super::{SyncLedgerTarget, SyncLedgerTargetKind};

#[derive(derive_more::From, Serialize, Deserialize, Debug, Clone)]
pub enum TransitionFrontierSyncLedgerState {
    Init {
        time: Timestamp,
        target: SyncLedgerTarget,
    },
    #[from]
    Snarked(TransitionFrontierSyncLedgerSnarkedState),
    #[from]
    Staged(TransitionFrontierSyncLedgerStagedState),
    Success {
        time: Timestamp,
        target: SyncLedgerTarget,
        needed_protocol_states: BTreeMap<StateHash, MinaStateProtocolStateValueStableV2>,
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
            Self::Snarked(s) if s.is_pending() => false,
            _ => true,
        }
    }

    // TODO(binier): maybe avoid extra cloning.
    pub fn target(&self) -> SyncLedgerTarget {
        match self {
            Self::Init { target, .. } | Self::Success { target, .. } => target.clone(),
            Self::Snarked(s) => s.target().clone(),
            Self::Staged(s) => s.target().clone().into(),
        }
    }

    pub fn target_kind(&self) -> SyncLedgerTargetKind {
        match self {
            Self::Init { target, .. } | Self::Success { target, .. } => target.kind,
            Self::Snarked(s) => s.target().kind,
            Self::Staged(s) => s.target().kind,
        }
    }

    pub fn update_target(&mut self, time: Timestamp, new_target: SyncLedgerTarget) {
        match self {
            Self::Snarked(
                TransitionFrontierSyncLedgerSnarkedState::NumAccountsPending { target, .. }
                | TransitionFrontierSyncLedgerSnarkedState::MerkleTreeSyncPending { target, .. },
            ) => {
                if target.snarked_ledger_hash == new_target.snarked_ledger_hash {
                    *target = new_target;
                } else {
                    *self = Self::Init {
                        time,
                        target: new_target,
                    };
                }
            }
            Self::Staged(staged) => {
                let target = staged.target();
                if target.snarked_ledger_hash == new_target.snarked_ledger_hash {
                    *self = TransitionFrontierSyncLedgerSnarkedState::Success {
                        time,
                        target: target.clone().into(),
                    }
                    .into();
                } else {
                    *self = Self::Init {
                        time,
                        target: new_target,
                    };
                }
            }
            _ => {
                // should be impossible.
            }
        }
    }
}
