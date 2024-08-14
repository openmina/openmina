use std::{collections::LinkedList, sync::Arc};

use binprot_derive::{BinProtRead, BinProtWrite};
use derive_more::From;
use mina_p2p_messages::{
    list::List,
    number::{UInt32, UInt64},
    v2,
};
use serde::{Deserialize, Serialize};

use crate::channels::rpc::StagedLedgerAuxAndPendingCoinbases;

pub type StagedLedgerPartsResponseFull = Arc<StagedLedgerAuxAndPendingCoinbases>;

/// Smaller sized parts of the staged ledger parts packed in one message.
#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub struct StagedLedgerPartsBase {
    pub staged_ledger_hash: v2::LedgerHash,
    pub pending_coinbase: Box<v2::MinaBasePendingCoinbaseStableV2>,
    pub needed_blocks: List<v2::MinaStateProtocolStateValueStableV2>,
}

/// Scan state smaller fields + newly emitted scan state proof (if any).
#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Clone)]
pub struct ScanStateBase {
    pub trees: UInt32,
    pub acc: Option<(
        Box<v2::TransactionSnarkScanStateLedgerProofWithSokMessageStableV2>,
        List<v2::TransactionSnarkScanStateTransactionWithWitnessStableV2>,
    )>,
    pub curr_job_seq_no: UInt64,
    pub max_base_jobs: UInt64,
    pub delay: UInt64,
}

/// Scan state previous incomplete zkapp updates.
pub type PreviousIncompleteZkappUpdates = (
    List<v2::TransactionSnarkScanStateTransactionWithWitnessStableV2>,
    v2::TransactionSnarkScanStateStableV2PreviousIncompleteZkappUpdates1,
);

/// Individual scan state tree.
pub type ScanStateTree = v2::TransactionSnarkScanStateStableV2ScanStateTreesA;

#[derive(BinProtWrite, BinProtRead, Serialize, Deserialize, From, Debug, Clone)]
pub enum StagedLedgerPartsResponse {
    Base(StagedLedgerPartsBase),
    ScanStateBase(ScanStateBase),
    PreviousIncompleteZkappUpdates(PreviousIncompleteZkappUpdates),
    ScanStateTree(ScanStateTree),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StagedLedgerPartsSendProgress {
    /// Haven't yet requested from ledger to fetch the parts.
    LedgerGetIdle {
        time: redux::Timestamp,
    },
    LedgerGetPending {
        time: redux::Timestamp,
    },
    LedgerGetSuccess {
        time: redux::Timestamp,
        data: Option<StagedLedgerPartsResponseFull>,
    },
    BaseSent {
        time: redux::Timestamp,
        data: StagedLedgerPartsResponseFull,
    },
    ScanStateBaseSent {
        time: redux::Timestamp,
        data: StagedLedgerPartsResponseFull,
    },
    PreviousIncompleteZkappUpdatesSent {
        time: redux::Timestamp,
        data: StagedLedgerPartsResponseFull,
    },
    ScanStateTreesSending {
        time: redux::Timestamp,
        data: StagedLedgerPartsResponseFull,
        tree_index: usize,
    },
    Success {
        time: redux::Timestamp,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StagedLedgerPartsReceiveProgress {
    BasePending {
        time: redux::Timestamp,
    },
    BaseSuccess {
        time: redux::Timestamp,
        base: StagedLedgerPartsBase,
    },
    ScanStateBasePending {
        time: redux::Timestamp,
        base: StagedLedgerPartsBase,
    },
    ScanStateBaseSuccess {
        time: redux::Timestamp,
        base: StagedLedgerPartsBase,
        scan_state_base: ScanStateBase,
    },
    PreviousIncompleteZkappUpdatesPending {
        time: redux::Timestamp,
        base: StagedLedgerPartsBase,
        scan_state_base: ScanStateBase,
    },
    PreviousIncompleteZkappUpdatesSuccess {
        time: redux::Timestamp,
        base: StagedLedgerPartsBase,
        scan_state_base: ScanStateBase,
        previous_incomplete_zkapp_updates: PreviousIncompleteZkappUpdates,
    },
    ScanStateTreesPending {
        time: redux::Timestamp,
        base: StagedLedgerPartsBase,
        scan_state_base: ScanStateBase,
        previous_incomplete_zkapp_updates: PreviousIncompleteZkappUpdates,
        trees: LinkedList<ScanStateTree>,
        is_next_tree_requested: bool,
    },
    Success {
        time: redux::Timestamp,
        data: StagedLedgerPartsResponseFull,
    },
}

impl StagedLedgerPartsSendProgress {
    pub fn next_msg(&self) -> Option<StagedLedgerPartsResponse> {
        match self {
            Self::LedgerGetSuccess { data, .. } => data
                .as_ref()
                .map(|data| StagedLedgerPartsBase {
                    staged_ledger_hash: data.staged_ledger_hash.clone(),
                    pending_coinbase: Box::new(data.pending_coinbase.clone()),
                    needed_blocks: data.needed_blocks.clone(),
                })
                .map(Into::into),
            Self::BaseSent { data, .. } => Some(
                ScanStateBase {
                    trees: {
                        let total = data.scan_state.scan_state.trees.1.len() + 1;
                        (total as u32).into()
                    },
                    acc: data
                        .scan_state
                        .scan_state
                        .acc
                        .as_ref()
                        .map(|(a, b)| (a.clone().into(), b.clone())),
                    curr_job_seq_no: data.scan_state.scan_state.curr_job_seq_no,
                    max_base_jobs: data.scan_state.scan_state.max_base_jobs,
                    delay: data.scan_state.scan_state.delay,
                }
                .into(),
            ),
            Self::ScanStateBaseSent { data, .. } => Some(
                data.scan_state
                    .previous_incomplete_zkapp_updates
                    .clone()
                    .into(),
            ),
            Self::PreviousIncompleteZkappUpdatesSent { data, .. } => {
                Some(data.scan_state.scan_state.trees.0.clone().into())
            }
            Self::ScanStateTreesSending {
                data, tree_index, ..
            } => data
                .scan_state
                .scan_state
                .trees
                .1
                .iter()
                .nth(*tree_index)
                .cloned()
                .map(Into::into),
            _ => None,
        }
    }
}

impl Default for StagedLedgerPartsSendProgress {
    fn default() -> Self {
        Self::LedgerGetIdle {
            time: redux::Timestamp::ZERO,
        }
    }
}

impl StagedLedgerPartsReceiveProgress {
    pub fn update(&mut self, time: redux::Timestamp, resp: &StagedLedgerPartsResponse) -> bool {
        match (std::mem::take(self), resp) {
            (Self::BasePending { .. }, StagedLedgerPartsResponse::Base(base)) => {
                *self = Self::BaseSuccess {
                    time,
                    base: base.clone(),
                };
                true
            }
            (
                Self::ScanStateBasePending { base, .. },
                StagedLedgerPartsResponse::ScanStateBase(data),
            ) => {
                *self = Self::ScanStateBaseSuccess {
                    time,
                    base,
                    scan_state_base: data.clone(),
                };
                true
            }
            (
                Self::PreviousIncompleteZkappUpdatesPending {
                    base,
                    scan_state_base,
                    ..
                },
                StagedLedgerPartsResponse::PreviousIncompleteZkappUpdates(data),
            ) => {
                *self = Self::PreviousIncompleteZkappUpdatesSuccess {
                    time,
                    base,
                    scan_state_base,
                    previous_incomplete_zkapp_updates: data.clone(),
                };
                true
            }
            (
                Self::ScanStateTreesPending {
                    base,
                    scan_state_base,
                    previous_incomplete_zkapp_updates,
                    mut trees,
                    ..
                },
                StagedLedgerPartsResponse::ScanStateTree(tree),
            ) => {
                trees.extend(std::iter::once(tree.clone()));

                *self = if trees.len() >= scan_state_base.trees.as_u32() as usize {
                    // base, scan_state_base, previous_incomplete_zkapp_updates, trees
                    let data = StagedLedgerAuxAndPendingCoinbases {
                        scan_state: v2::TransactionSnarkScanStateStableV2 {
                            scan_state: v2::TransactionSnarkScanStateStableV2ScanState {
                                trees: {
                                    let first = trees.pop_front().unwrap();
                                    (first, trees.into())
                                },
                                acc: scan_state_base.acc.map(|(a, b)| (*a, b)),
                                curr_job_seq_no: scan_state_base.curr_job_seq_no,
                                max_base_jobs: scan_state_base.max_base_jobs,
                                delay: scan_state_base.delay,
                            },
                            previous_incomplete_zkapp_updates,
                        },
                        staged_ledger_hash: base.staged_ledger_hash,
                        pending_coinbase: *base.pending_coinbase,
                        needed_blocks: base.needed_blocks,
                    };
                    Self::Success {
                        time,
                        data: data.into(),
                    }
                } else {
                    Self::ScanStateTreesPending {
                        time,
                        base,
                        scan_state_base,
                        previous_incomplete_zkapp_updates,
                        trees,
                        is_next_tree_requested: false,
                    }
                };
                true
            }
            (old_state, _) => {
                *self = old_state;
                false
            }
        }
    }

    pub fn is_part_pending(&self) -> bool {
        // must match code below.
        match self {
            Self::BaseSuccess { .. }
            | Self::ScanStateBaseSuccess { .. }
            | Self::PreviousIncompleteZkappUpdatesSuccess { .. }
            | Self::Success { .. } => false,
            Self::ScanStateTreesPending {
                is_next_tree_requested,
                ..
            } => *is_next_tree_requested,
            _ => true,
        }
    }

    pub fn set_next_pending(&mut self, time: redux::Timestamp) -> bool {
        match std::mem::take(self) {
            Self::BaseSuccess { base, .. } => {
                *self = Self::ScanStateBasePending { time, base };
                true
            }
            Self::ScanStateBaseSuccess {
                base,
                scan_state_base,
                ..
            } => {
                *self = Self::PreviousIncompleteZkappUpdatesPending {
                    time,
                    base,
                    scan_state_base,
                };
                true
            }
            Self::PreviousIncompleteZkappUpdatesSuccess {
                base,
                scan_state_base,
                previous_incomplete_zkapp_updates,
                ..
            } => {
                *self = Self::ScanStateTreesPending {
                    time,
                    base,
                    scan_state_base,
                    previous_incomplete_zkapp_updates,
                    trees: Default::default(),
                    is_next_tree_requested: true,
                };
                true
            }
            Self::ScanStateTreesPending {
                time,
                base,
                scan_state_base,
                previous_incomplete_zkapp_updates,
                trees,
                ..
            } => {
                *self = Self::ScanStateTreesPending {
                    time,
                    base,
                    scan_state_base,
                    previous_incomplete_zkapp_updates,
                    trees,
                    is_next_tree_requested: true,
                };
                true
            }
            old_state => {
                *self = old_state;
                false
            }
        }
    }
}

impl Default for StagedLedgerPartsReceiveProgress {
    fn default() -> Self {
        Self::BasePending {
            time: redux::Timestamp::ZERO,
        }
    }
}
