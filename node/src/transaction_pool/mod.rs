use ledger::{
    transaction_pool::{
        diff::{BestTipDiff, DiffVerified},
        ApplyDecision,
    },
    Mask,
};

pub enum TransactionPoolAction {
    BestTipChanged {
        best_tip: Mask,
    },
    ApplyVerifiedDiff {
        diff: DiffVerified,
        is_sender_local: bool,
    },
    ApplyTransitionFrontierDiff {
        best_tip: Mask,
        diff: BestTipDiff,
    },
    // Rebroadcast locally generated pool items every 10 minutes. Do so for 50
    // minutes - at most 5 rebroadcasts - before giving up.
    Rebroadcast,
}

impl redux::EnablingCondition<crate::State> for TransactionPoolAction {
    fn is_enabled(&self, _state: &crate::State, _time: redux::Timestamp) -> bool {
        true
    }
}

pub struct TransactionPoolState {
    pool: ledger::transaction_pool::TransactionPool,
}

type TransactionPoolActionWithMetaRef<'a> = redux::ActionWithMeta<&'a TransactionPoolAction>;

impl TransactionPoolState {
    pub fn reducer(&mut self, action: TransactionPoolActionWithMetaRef<'_>) {
        use TransactionPoolAction::*;

        let (action, meta) = action.split();
        match action {
            BestTipChanged { best_tip } => {
                self.pool.on_new_best_tip(best_tip.clone());
            }
            ApplyVerifiedDiff {
                diff,
                is_sender_local,
            } => match self.pool.unsafe_apply(diff, *is_sender_local) {
                Ok((ApplyDecision::Accept, accepted, rejected)) => todo!(),
                Ok((ApplyDecision::Reject, accepted, rejected)) => todo!(),
                Err(_) => todo!(),
            },
            ApplyTransitionFrontierDiff { best_tip, diff } => {
                self.pool
                    .handle_transition_frontier_diff(diff, best_tip.clone());
            }
            Rebroadcast => todo!(),
        }
    }
}
