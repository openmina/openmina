use ledger::transaction_pool::{diff, ValidCommandWithHash};
use ledger::Account;
use mina_p2p_messages::v2::MinaBaseUserCommandStableV2;
use mina_p2p_messages::v2::TokenIdKeyHash;
use openmina_core::block::AppliedBlock;
use openmina_core::snark::SnarkJobId;
use openmina_core::ActionEvent;
use openmina_node_account::AccountPublicKey;
use p2p::PeerId;
use serde::{Deserialize, Serialize};

use crate::external_snark_worker::SnarkWorkId;
use crate::p2p::connection::incoming::P2pConnectionIncomingInitOpts;
use crate::p2p::connection::outgoing::{P2pConnectionOutgoingError, P2pConnectionOutgoingInitOpts};
use crate::p2p::connection::P2pConnectionResponse;

use super::{
    ActionStatsQuery, RpcId, RpcScanStateSummaryGetQuery, RpcScanStateSummaryScanStateJob,
    SyncStatsQuery,
};

pub type RpcActionWithMeta = redux::ActionWithMeta<RpcAction>;
pub type RpcActionWithMetaRef<'a> = redux::ActionWithMeta<&'a RpcAction>;

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum RpcAction {
    GlobalStateGet {
        rpc_id: RpcId,
        filter: Option<String>,
    },
    StatusGet {
        rpc_id: RpcId,
    },

    // Stats
    ActionStatsGet {
        rpc_id: RpcId,
        query: ActionStatsQuery,
    },
    SyncStatsGet {
        rpc_id: RpcId,
        query: SyncStatsQuery,
    },
    BlockProducerStatsGet {
        rpc_id: RpcId,
    },

    MessageProgressGet {
        rpc_id: RpcId,
    },

    PeersGet {
        rpc_id: RpcId,
    },

    P2pConnectionOutgoingInit {
        rpc_id: RpcId,
        opts: P2pConnectionOutgoingInitOpts,
    },
    P2pConnectionOutgoingPending {
        rpc_id: RpcId,
    },
    P2pConnectionOutgoingError {
        rpc_id: RpcId,
        error: P2pConnectionOutgoingError,
    },
    P2pConnectionOutgoingSuccess {
        rpc_id: RpcId,
    },

    P2pConnectionIncomingInit {
        rpc_id: RpcId,
        opts: P2pConnectionIncomingInitOpts,
    },
    P2pConnectionIncomingPending {
        rpc_id: RpcId,
    },
    P2pConnectionIncomingRespond {
        rpc_id: RpcId,
        response: P2pConnectionResponse,
    },
    P2pConnectionIncomingAnswerReady {
        rpc_id: RpcId,
        peer_id: PeerId,
        answer: P2pConnectionResponse,
    },
    P2pConnectionIncomingError {
        rpc_id: RpcId,
        error: String,
    },
    P2pConnectionIncomingSuccess {
        rpc_id: RpcId,
    },

    ScanStateSummaryGetInit {
        rpc_id: RpcId,
        query: RpcScanStateSummaryGetQuery,
    },
    ScanStateSummaryLedgerGetInit {
        rpc_id: RpcId,
    },
    ScanStateSummaryGetPending {
        rpc_id: RpcId,
        block: Option<AppliedBlock>,
    },
    ScanStateSummaryGetSuccess {
        rpc_id: RpcId,
        scan_state: Result<Vec<Vec<RpcScanStateSummaryScanStateJob>>, String>,
    },

    SnarkPoolAvailableJobsGet {
        rpc_id: RpcId,
    },
    SnarkPoolJobGet {
        job_id: SnarkWorkId,
        rpc_id: RpcId,
    },

    SnarkerConfigGet {
        rpc_id: RpcId,
    },
    SnarkerJobCommit {
        rpc_id: RpcId,
        job_id: SnarkJobId,
    },
    SnarkerJobSpec {
        rpc_id: RpcId,
        job_id: SnarkJobId,
    },

    SnarkerWorkersGet {
        rpc_id: RpcId,
    },

    HealthCheck {
        rpc_id: RpcId,
    },
    ReadinessCheck {
        rpc_id: RpcId,
    },

    DiscoveryRoutingTable {
        rpc_id: RpcId,
    },
    DiscoveryBoostrapStats {
        rpc_id: RpcId,
    },

    TransactionPool {
        rpc_id: RpcId,
    },
    #[action_event(level = info)]
    LedgerAccountsGetInit {
        rpc_id: RpcId,
        account_query: AccountQuery,
    },
    #[action_event(level = info)]
    LedgerAccountsGetPending {
        rpc_id: RpcId,
    },
    #[action_event(level = info)]
    LedgerAccountsGetSuccess {
        rpc_id: RpcId,
        accounts: Vec<Account>,
        account_query: AccountQuery,
    },
    #[action_event(level = info)]
    TransactionInjectInit {
        rpc_id: RpcId,
        commands: Vec<MinaBaseUserCommandStableV2>,
    },
    #[action_event(level = info)]
    TransactionInjectPending {
        rpc_id: RpcId,
    },
    #[action_event(level = info)]
    TransactionInjectSuccess {
        rpc_id: RpcId,
        response: Vec<ValidCommandWithHash>,
    },
    #[action_event(level = info)]
    TransactionInjectRejected {
        rpc_id: RpcId,
        response: Vec<(ValidCommandWithHash, diff::Error)>,
    },
    #[action_event(level = warn)]
    TransactionInjectFailure {
        rpc_id: RpcId,
        errors: Vec<String>,
    },
    #[action_event(level = info)]
    TransitionFrontierUserCommandsGet {
        rpc_id: RpcId,
    },

    BestChain {
        rpc_id: RpcId,
        max_length: u32,
    },
    ConsensusConstantsGet {
        rpc_id: RpcId,
    },

    TransactionStatusGet {
        rpc_id: RpcId,
        tx: MinaBaseUserCommandStableV2,
    },

    Finish {
        rpc_id: RpcId,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccountQuery {
    SinglePublicKey(AccountPublicKey),
    All,
    PubKeyWithTokenId(AccountPublicKey, TokenIdKeyHash),
}

impl redux::EnablingCondition<crate::State> for RpcAction {
    fn is_enabled(&self, state: &crate::State, _time: redux::Timestamp) -> bool {
        match self {
            RpcAction::GlobalStateGet { .. } => true,
            RpcAction::StatusGet { .. } => true,
            RpcAction::ActionStatsGet { .. } => true,
            RpcAction::SyncStatsGet { .. } => true,
            RpcAction::BlockProducerStatsGet { .. } => true,
            RpcAction::MessageProgressGet { .. } => true,
            RpcAction::PeersGet { .. } => true,
            RpcAction::P2pConnectionOutgoingInit { rpc_id, .. } => {
                !state.rpc.requests.contains_key(rpc_id)
            }
            RpcAction::P2pConnectionOutgoingPending { rpc_id } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_init()),
            RpcAction::P2pConnectionOutgoingError { rpc_id, .. } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_pending()),
            RpcAction::P2pConnectionOutgoingSuccess { rpc_id } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_pending()),
            RpcAction::P2pConnectionIncomingInit { rpc_id, .. } => {
                !state.rpc.requests.contains_key(rpc_id)
            }
            RpcAction::P2pConnectionIncomingPending { rpc_id } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_init()),
            RpcAction::P2pConnectionIncomingRespond { rpc_id, .. }
            | RpcAction::P2pConnectionIncomingAnswerReady { rpc_id, .. } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_init() || v.status.is_pending()),
            RpcAction::P2pConnectionIncomingError { rpc_id, .. } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_init() || v.status.is_pending()),
            RpcAction::P2pConnectionIncomingSuccess { rpc_id } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_pending()),
            RpcAction::ScanStateSummaryGetInit { .. } => true,
            RpcAction::ScanStateSummaryLedgerGetInit { rpc_id, .. } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_init()),
            RpcAction::ScanStateSummaryGetPending { rpc_id, .. } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_init()),
            RpcAction::ScanStateSummaryGetSuccess { rpc_id, .. } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_pending()),
            RpcAction::SnarkPoolAvailableJobsGet { .. } => true,
            RpcAction::SnarkPoolJobGet { .. } => true,
            RpcAction::SnarkerConfigGet { .. } => true,
            RpcAction::SnarkerJobCommit { .. } => true,
            RpcAction::SnarkerJobSpec { .. } => true,
            RpcAction::SnarkerWorkersGet { .. } => true,
            RpcAction::HealthCheck { .. } => true,
            RpcAction::ReadinessCheck { .. } => true,
            RpcAction::DiscoveryRoutingTable { .. } => true,
            RpcAction::DiscoveryBoostrapStats { .. } => true,
            RpcAction::TransactionPool { .. } => true,
            RpcAction::ConsensusConstantsGet { .. } => true,
            RpcAction::BestChain { .. } => state.transition_frontier.best_tip().is_some(),
            RpcAction::TransactionStatusGet { .. } => true,
            RpcAction::LedgerAccountsGetInit { .. } => {
                state.transition_frontier.best_tip().is_some()
            }
            RpcAction::LedgerAccountsGetPending { rpc_id, .. } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_init()),
            RpcAction::LedgerAccountsGetSuccess { rpc_id, .. } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_pending()),

            RpcAction::TransactionInjectInit { .. } => true,
            RpcAction::TransactionInjectPending { rpc_id } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_init()),
            RpcAction::TransactionInjectSuccess { rpc_id, .. } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_pending()),
            RpcAction::TransactionInjectRejected { rpc_id, .. } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_pending()),
            RpcAction::TransactionInjectFailure { rpc_id, .. } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_pending()),
            RpcAction::TransitionFrontierUserCommandsGet { .. } => true,
            RpcAction::Finish { rpc_id } => state
                .rpc
                .requests
                .get(rpc_id)
                .map_or(false, |v| v.status.is_finished()),
        }
    }
}
