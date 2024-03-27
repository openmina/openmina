use std::collections::BTreeMap;

use ledger::{
    transaction_pool::diff::{BestTipDiff, DiffVerified},
    Account, AccountId,
};
use mina_p2p_messages::v2::LedgerHash;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransactionPoolAction {
    BestTipChanged {
        best_tip_hash: LedgerHash,
    },
    BestTipChangedWithAccounts {
        accounts: BTreeMap<AccountId, Account>,
    },
    ApplyVerifiedDiff {
        diff: DiffVerified,
        is_sender_local: bool,
    },
    ApplyTransitionFrontierDiff {
        best_tip_hash: LedgerHash,
        // best_tip: Mask,
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
