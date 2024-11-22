use std::collections::{BTreeMap, BTreeSet};

use ledger::{
    transaction_pool::{
        diff::{self, BestTipDiff, DiffVerified},
        ValidCommandWithHash,
    },
    Account, AccountId,
};
use mina_p2p_messages::{
    list::List,
    v2::{self, LedgerHash},
};
use openmina_core::{requests::RpcId, ActionEvent};
use redux::Callback;
use serde::{Deserialize, Serialize};

use super::PendingId;

pub type TransactionPoolActionWithMeta = redux::ActionWithMeta<TransactionPoolAction>;
pub type TransactionPoolActionWithMetaRef<'a> = redux::ActionWithMeta<&'a TransactionPoolAction>;

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(level = info)]
pub enum TransactionPoolAction {
    StartVerify {
        commands: List<v2::MinaBaseUserCommandStableV2>,
        from_rpc: Option<RpcId>,
    },
    StartVerifyWithAccounts {
        accounts: BTreeMap<AccountId, Account>,
        pending_id: PendingId,
        from_rpc: Option<RpcId>,
    },
    #[action_event(level = warn, fields(debug(errors)))]
    VerifyError {
        errors: Vec<String>,
    },
    BestTipChanged {
        best_tip_hash: LedgerHash,
    },
    BestTipChangedWithAccounts {
        accounts: BTreeMap<AccountId, Account>,
    },
    ApplyVerifiedDiff {
        best_tip_hash: LedgerHash,
        diff: DiffVerified,
        /// Diff was crearted locally, or from remote peer ?
        is_sender_local: bool,
        from_rpc: Option<RpcId>,
    },
    ApplyVerifiedDiffWithAccounts {
        accounts: BTreeMap<AccountId, Account>,
        pending_id: PendingId,
    },
    ApplyTransitionFrontierDiff {
        best_tip_hash: LedgerHash,
        diff: BestTipDiff,
    },
    ApplyTransitionFrontierDiffWithAccounts {
        accounts: BTreeMap<AccountId, Account>,
        pending_id: PendingId,
    },
    /// Rebroadcast locally generated pool items every 10 minutes. Do so for 50
    /// minutes - at most 5 rebroadcasts - before giving up.
    Rebroadcast {
        accepted: Vec<ValidCommandWithHash>,
        rejected: Vec<(ValidCommandWithHash, diff::Error)>,
    },
    CollectTransactionsByFee,
}

impl redux::EnablingCondition<crate::State> for TransactionPoolAction {}

type TransactionPoolEffectfulActionCallback = Callback<(
    BTreeMap<AccountId, Account>,
    Option<PendingId>,
    Option<RpcId>,
)>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransactionPoolEffectfulAction {
    FetchAccounts {
        account_ids: BTreeSet<AccountId>,
        ledger_hash: LedgerHash,
        on_result: TransactionPoolEffectfulActionCallback,
        pending_id: Option<PendingId>,
        from_rpc: Option<RpcId>,
    },
}

impl redux::EnablingCondition<crate::State> for TransactionPoolEffectfulAction {}
