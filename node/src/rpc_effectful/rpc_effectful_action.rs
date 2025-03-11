use crate::{
    external_snark_worker::{ExternalSnarkWorker, SnarkWorkId},
    p2p::connection::P2pConnectionResponse,
    rpc::{
        discovery::RpcDiscoveryRoutingTable, AccountQuery, ActionStatsQuery, RpcBestChainResponse,
        RpcGetBlockResponse, RpcPeerInfo, RpcPooledUserCommandsResponse,
        RpcScanStateSummaryScanStateJob, RpcSnarkerConfig, RpcTransactionInjectFailure,
        RpcTransactionInjectRejected, RpcTransactionInjectSuccess, SyncStatsQuery,
    },
};
use ledger::{
    scan_state::transaction_logic::{valid::UserCommand, zkapp_command::WithHash},
    Account,
};
use mina_p2p_messages::v2::{self, MinaBaseUserCommandStableV2};
use openmina_core::{
    consensus::ConsensusConstants, requests::RpcId, snark::SnarkJobId, ActionEvent,
};
use p2p::bootstrap::P2pNetworkKadBootstrapStats;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ActionEvent)]
pub enum RpcEffectfulAction {
    GlobalStateGet {
        rpc_id: RpcId,
        filter: Option<String>,
    },
    StatusGet {
        rpc_id: RpcId,
    },
    HeartbeatGet {
        rpc_id: RpcId,
    },
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
        peers: Vec<RpcPeerInfo>,
    },
    P2pConnectionOutgoingError {
        rpc_id: RpcId,
        error: String,
    },
    P2pConnectionOutgoingSuccess {
        rpc_id: RpcId,
    },
    P2pConnectionIncomingRespond {
        rpc_id: RpcId,
        response: P2pConnectionResponse,
    },
    P2pConnectionIncomingError {
        rpc_id: RpcId,
        error: String,
    },
    P2pConnectionIncomingSuccess {
        rpc_id: RpcId,
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
        config: Option<RpcSnarkerConfig>,
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
        snark_worker: ExternalSnarkWorker,
    },
    HealthCheck {
        rpc_id: RpcId,
        has_peers: Result<(), String>,
    },
    ReadinessCheck {
        rpc_id: RpcId,
    },
    DiscoveryRoutingTable {
        rpc_id: RpcId,
        response: Option<RpcDiscoveryRoutingTable>,
    },
    DiscoveryBoostrapStats {
        rpc_id: RpcId,
        response: Option<P2pNetworkKadBootstrapStats>,
    },
    TransactionPool {
        rpc_id: RpcId,
        response: Vec<WithHash<UserCommand, v2::TransactionHash>>,
    },
    LedgerAccountsGetSuccess {
        rpc_id: RpcId,
        accounts: Vec<Account>,
        account_query: AccountQuery,
    },
    TransactionInjectSuccess {
        rpc_id: RpcId,
        response: RpcTransactionInjectSuccess,
    },
    TransactionInjectRejected {
        rpc_id: RpcId,
        response: RpcTransactionInjectRejected,
    },
    TransactionInjectFailure {
        rpc_id: RpcId,
        errors: RpcTransactionInjectFailure,
    },
    TransitionFrontierUserCommandsGet {
        rpc_id: RpcId,
        commands: Vec<MinaBaseUserCommandStableV2>,
    },
    BestChain {
        rpc_id: RpcId,
        best_chain: RpcBestChainResponse,
    },
    ConsensusConstantsGet {
        rpc_id: RpcId,
        response: ConsensusConstants,
    },
    TransactionStatusGet {
        rpc_id: RpcId,
        tx: MinaBaseUserCommandStableV2,
    },
    BlockGet {
        rpc_id: RpcId,
        block: RpcGetBlockResponse,
    },
    PooledUserCommands {
        rpc_id: RpcId,
        user_commands: RpcPooledUserCommandsResponse,
    },
}

impl redux::EnablingCondition<crate::State> for RpcEffectfulAction {
    fn is_enabled(&self, _state: &crate::State, _time: redux::Timestamp) -> bool {
        true
    }
}
