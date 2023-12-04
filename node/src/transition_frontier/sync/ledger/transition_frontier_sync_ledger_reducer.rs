use std::collections::BTreeMap;

use super::snarked::{
    TransitionFrontierSyncLedgerSnarkedAction, TransitionFrontierSyncLedgerSnarkedState,
};
use super::staged::{
    TransitionFrontierSyncLedgerStagedAction, TransitionFrontierSyncLedgerStagedState,
};
use super::{
    TransitionFrontierSyncLedgerAction, TransitionFrontierSyncLedgerActionWithMetaRef,
    TransitionFrontierSyncLedgerState,
};

impl TransitionFrontierSyncLedgerState {
    pub fn reducer(&mut self, action: TransitionFrontierSyncLedgerActionWithMetaRef<'_>) {
        let (action, meta) = action.split();
        match action {
            TransitionFrontierSyncLedgerAction::Init(_) => {
                println!("++++++ TransitionFrontierSyncLedgerAction::Init")
            }
            TransitionFrontierSyncLedgerAction::Snarked(action) => {
                if let TransitionFrontierSyncLedgerSnarkedAction::Pending(_) = action {
                    let Self::Init { block, .. } = self else {
                        println!("+++++ STATE IS NOT INIT");
                        return;
                    };
                    let s = TransitionFrontierSyncLedgerSnarkedState::pending(
                        meta.time(),
                        block.clone(),
                    );
                    println!("+++++ STATE SET TO SNARKED");
                    *self = Self::Snarked(s);
                } else {
                    let Self::Snarked(state) = self else {
                        println!("+++++ TransitionFrontierSyncLedgerSnarkedAction that is not Pending or Snarked {:?}", self);
                        return;
                    };
                    state.reducer(meta.with_action(action));
                }
            }
            TransitionFrontierSyncLedgerAction::Staged(action) => {
                if let TransitionFrontierSyncLedgerStagedAction::PartsFetchPending(_) = action {
                    let Self::Snarked(TransitionFrontierSyncLedgerSnarkedState::Success {
                        block,
                        ..
                    }) = self
                    else {
                        return;
                    };
                    let s = TransitionFrontierSyncLedgerStagedState::pending(
                        meta.time(),
                        block.clone(),
                    );
                    *self = Self::Staged(s);
                } else {
                    match self {
                        Self::Snarked(TransitionFrontierSyncLedgerSnarkedState::Success {
                            block,
                            ..
                        }) if matches!(
                            action,
                            TransitionFrontierSyncLedgerStagedAction::ReconstructEmpty(_)
                        ) =>
                        {
                            let s = TransitionFrontierSyncLedgerStagedState::ReconstructEmpty {
                                time: meta.time(),
                                block: block.clone(),
                            };
                            *self = Self::Staged(s);
                        }
                        Self::Staged(state) => state.reducer(meta.with_action(action)),
                        _ => return,
                    }
                }
            }
            TransitionFrontierSyncLedgerAction::Success(_) => {
                // TODO(tizoc): target may be snarked instead of staged depending
                // on which ledger we are syncing, this check doesn't account for that
                match self {
                    Self::Staged(TransitionFrontierSyncLedgerStagedState::Success {
                        block,
                        needed_protocol_states,
                        ..
                    }) => {
                        *self = Self::Success {
                            time: meta.time(),
                            block: block.clone(),
                            needed_protocol_states: std::mem::take(needed_protocol_states),
                        };
                    }
                    Self::Snarked(TransitionFrontierSyncLedgerSnarkedState::Success {
                        block,
                        ..
                    }) => {
                        // TODO(tizoc): is this success good enough? it is meant for staged,
                        // so needed_protocol_states doesn't make sense
                        // Block may not be right for staking ledgers
                        *self = Self::Success {
                            time: meta.time(),
                            block: block.clone(),
                            needed_protocol_states: BTreeMap::new(),
                        };
                    }
                    _ => {
                        println!("++++ Dispatched TransitionFrontierSyncLedgerSuccessAction without state being Staged or Snarked");
                    }
                }
            }
        }
    }
}
