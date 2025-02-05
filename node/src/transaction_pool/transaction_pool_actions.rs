use std::collections::{BTreeMap, BTreeSet};

use ledger::{
    scan_state::transaction_logic::valid,
    transaction_pool::{
        diff::{self, BestTipDiff, DiffVerified},
        ValidCommandWithHash,
    },
    Account, AccountId,
};
use mina_p2p_messages::{
    list::List,
    v2::{self, TransactionHash},
};
use openmina_core::{requests::RpcId, transaction::TransactionWithHash, ActionEvent};
use redux::Callback;
use serde::{Deserialize, Serialize};

use super::{candidate::TransactionPoolCandidateAction, PendingId};

pub type TransactionPoolActionWithMeta = redux::ActionWithMeta<TransactionPoolAction>;
pub type TransactionPoolActionWithMetaRef<'a> = redux::ActionWithMeta<&'a TransactionPoolAction>;

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
#[action_event(level = info)]
pub enum TransactionPoolAction {
    Candidate(TransactionPoolCandidateAction),
    StartVerify {
        commands: List<TransactionWithHash>,
        from_rpc: Option<RpcId>,
    },
    StartVerifyWithAccounts {
        accounts: BTreeMap<AccountId, Account>,
        pending_id: PendingId,
        from_rpc: Option<RpcId>,
    },
    VerifySuccess {
        valids: Vec<valid::UserCommand>,
        from_rpc: Option<RpcId>,
    },
    #[action_event(level = warn, fields(debug(errors)))]
    VerifyError {
        errors: Vec<String>,
        tx_hashes: Vec<TransactionHash>,
    },
    BestTipChanged {
        best_tip_hash: v2::LedgerHash,
    },
    BestTipChangedWithAccounts {
        accounts: BTreeMap<AccountId, Account>,
    },
    ApplyVerifiedDiff {
        best_tip_hash: v2::LedgerHash,
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
        best_tip_hash: v2::LedgerHash,
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
        is_local: bool,
    },
    CollectTransactionsByFee,
    #[action_event(level = trace)]
    P2pSendAll,
    #[action_event(level = debug)]
    P2pSend {
        peer_id: p2p::PeerId,
    },
}

impl redux::EnablingCondition<crate::State> for TransactionPoolAction {
    fn is_enabled(&self, state: &crate::State, time: redux::Timestamp) -> bool {
        match self {
            TransactionPoolAction::Candidate(a) => a.is_enabled(state, time),
            TransactionPoolAction::StartVerify { commands, .. } => {
                !commands.is_empty()
                    && commands
                        .iter()
                        .any(|cmd| !state.transaction_pool.contains(cmd.hash()))
            }
            TransactionPoolAction::P2pSendAll => true,
            TransactionPoolAction::P2pSend { peer_id } => state
                .p2p
                .get_ready_peer(peer_id)
                // can't propagate empty transaction pool
                .filter(|_| !state.transaction_pool.dpool.is_empty())
                // Only send transactions if peer has the same best tip,
                // or its best tip is extension of our best tip.
                .and_then(|p| {
                    let peer_best_tip = p.best_tip.as_ref()?;
                    let our_best_tip = state.transition_frontier.best_tip()?.hash();
                    Some(p).filter(|_| {
                        peer_best_tip.hash() == our_best_tip
                            || peer_best_tip.pred_hash() == our_best_tip
                    })
                })
                .is_some_and(|p| {
                    let check =
                        |(next_index, limit), last_index| limit > 0 && next_index <= last_index;
                    let last_index = state.transaction_pool.dpool.last_index();

                    check(
                        p.channels.transaction.next_send_index_and_limit(),
                        last_index,
                    )
                }),
            TransactionPoolAction::Rebroadcast {
                accepted, rejected, ..
            } => !(accepted.is_empty() && rejected.is_empty()),
            _ => true,
        }
    }
}

type TransactionPoolEffectfulActionCallback = Callback<(
    BTreeMap<AccountId, Account>,
    Option<PendingId>,
    Option<RpcId>,
)>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransactionPoolEffectfulAction {
    FetchAccounts {
        account_ids: BTreeSet<AccountId>,
        ledger_hash: v2::LedgerHash,
        on_result: TransactionPoolEffectfulActionCallback,
        pending_id: Option<PendingId>,
        from_rpc: Option<RpcId>,
    },
}

impl redux::EnablingCondition<crate::State> for TransactionPoolEffectfulAction {}
