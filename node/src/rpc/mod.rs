mod rpc_state;
use std::collections::BTreeMap;
use std::str::FromStr;

use ark_ff::fields::arithmetic::InvalidBigInt;
use ledger::scan_state::currency::{Amount, Balance, Fee, Nonce, Slot};
use ledger::scan_state::transaction_logic::signed_command::SignedCommandPayload;
use ledger::scan_state::transaction_logic::{self, signed_command, valid, Memo};
use ledger::transaction_pool::{diff, ValidCommandWithHash};
use ledger::Account;
use mina_p2p_messages::bigint::BigInt;
use mina_p2p_messages::v2::{
    MinaBaseSignedCommandPayloadBodyStableV2, MinaBaseTransactionStatusStableV2,
    MinaBaseUserCommandStableV2, MinaTransactionTransactionStableV2,
    SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse, StateHash, TransactionHash,
};
use openmina_core::block::ArcBlockWithHash;
use openmina_node_account::AccountPublicKey;
use p2p::bootstrap::P2pNetworkKadBootstrapStats;
pub use rpc_state::*;

mod rpc_actions;
pub use rpc_actions::*;

mod rpc_reducer;

mod rpc_effects;
pub use rpc_effects::*;

mod rpc_service;
pub use rpc_service::*;

mod rpc_impls;

pub use openmina_core::requests::{RpcId, RpcIdType};

use ledger::scan_state::scan_state::transaction_snark::OneOrTwo;
use ledger::scan_state::scan_state::AvailableJobMessage;
use mina_p2p_messages::v2::{CurrencyFeeStableV1, NonZeroCurvePoint};
use openmina_core::snark::SnarkJobId;
use redux::Timestamp;
use serde::{Deserialize, Serialize};

use crate::external_snark_worker::{
    ExternalSnarkWorkerError, ExternalSnarkWorkerWorkError, SnarkWorkSpecError,
};
use crate::p2p::connection::incoming::P2pConnectionIncomingInitOpts;
use crate::p2p::connection::outgoing::P2pConnectionOutgoingInitOpts;
use crate::p2p::PeerId;
use crate::snark_pool::{JobCommitment, JobSummary};
use crate::stats::actions::{ActionStatsForBlock, ActionStatsSnapshot};
use crate::stats::block_producer::{BlockProductionAttempt, BlockProductionAttemptWonSlot};
use crate::stats::sync::SyncStatsSnapshot;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RpcRequest {
    StateGet(Option<String>),
    StatusGet,
    ActionStatsGet(ActionStatsQuery),
    SyncStatsGet(SyncStatsQuery),
    BlockProducerStatsGet,
    MessageProgressGet,
    PeersGet,
    P2pConnectionOutgoing(P2pConnectionOutgoingInitOpts),
    P2pConnectionIncoming(P2pConnectionIncomingInitOpts),
    ScanStateSummaryGet(RpcScanStateSummaryGetQuery),
    SnarkPoolGet,
    SnarkPoolJobGet { job_id: SnarkJobId },
    SnarkerConfig,
    SnarkerJobCommit { job_id: SnarkJobId },
    SnarkerJobSpec { job_id: SnarkJobId },
    SnarkerWorkers,
    HealthCheck,
    ReadinessCheck,
    DiscoveryRoutingTable,
    DiscoveryBoostrapStats,
    TransactionPoolGet,
    LedgerAccountsGet(AccountQuery),
    TransactionInject(Vec<MinaBaseUserCommandStableV2>),
    TransitionFrontierUserCommandsGet,
    BestChain(MaxLength),
}

pub type MaxLength = u32;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcInjectPayment {
    fee: u64,
    amount: u64,
    to: AccountPublicKey,
    from: AccountPublicKey,
    memo: String,
    nonce: u32,
    valid_until: u32,
    signature_field: BigInt,
    signature_scalar: BigInt,
}
// MinaBaseUserCommandStableV2
impl TryFrom<RpcInjectPayment> for MinaBaseUserCommandStableV2 {
    type Error = InvalidBigInt;

    fn try_from(value: RpcInjectPayment) -> Result<Self, Self::Error> {
        let signature = mina_signer::Signature {
            rx: value.signature_field.try_into()?,
            s: value.signature_scalar.try_into()?,
        };
        println!("Signature: {signature}");
        let sc = signed_command::SignedCommand {
            payload: SignedCommandPayload::create(
                Fee::from_u64(value.fee),
                value.from.clone().try_into().map_err(|_| InvalidBigInt)?,
                Nonce::from_u32(value.nonce),
                Some(Slot::from_u32(value.valid_until)),
                Memo::from_str(&value.memo).unwrap(),
                signed_command::Body::Payment(signed_command::PaymentPayload {
                    receiver_pk: value.to.try_into().map_err(|_| InvalidBigInt)?,
                    amount: Amount::from_u64(value.amount),
                }),
            ),
            signer: value.from.try_into().map_err(|_| InvalidBigInt)?,
            signature,
        };

        Ok(MinaBaseUserCommandStableV2::SignedCommand(sc.into()))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActionStatsQuery {
    SinceStart,
    ForLatestBlock,
    ForBlockWithId(u64),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncStatsQuery {
    pub limit: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RpcScanStateSummaryGetQuery {
    ForBestTip,
    ForBlockWithHash(StateHash),
    ForBlockWithHeight(u32),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum ActionStatsResponse {
    SinceStart { stats: ActionStatsSnapshot },
    ForBlock(ActionStatsForBlock),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PeerConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcPeerInfo {
    pub peer_id: PeerId,
    pub best_tip: Option<StateHash>,
    pub best_tip_height: Option<u32>,
    pub best_tip_global_slot: Option<u32>,
    pub best_tip_timestamp: Option<u64>,
    pub connection_status: PeerConnectionStatus,
    pub address: Option<String>,
    pub time: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcScanStateSummary {
    pub block: RpcScanStateSummaryBlock,
    pub scan_state: Vec<Vec<RpcScanStateSummaryScanStateJob>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcScanStateSummaryBlock {
    pub hash: StateHash,
    pub height: u32,
    pub global_slot: u32,
    pub transactions: Vec<RpcScanStateSummaryBlockTransaction>,
    pub completed_works: Vec<SnarkJobId>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcScanStateSummaryBlockTransaction {
    /// None if hashing fails.
    pub hash: Option<TransactionHash>,
    pub kind: RpcScanStateSummaryBlockTransactionKind,
    pub status: MinaBaseTransactionStatusStableV2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RpcScanStateSummaryBlockTransactionKind {
    Payment,
    StakeDelegation,
    Zkapp,
    FeeTransfer,
    Coinbase,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "status")]
pub enum RpcScanStateSummaryScanStateJob {
    Empty,
    Todo {
        job_id: SnarkJobId,
        bundle_job_id: SnarkJobId,
        job: RpcScanStateSummaryScanStateJobKind,
        seq_no: u64,
    },
    Pending {
        job_id: SnarkJobId,
        bundle_job_id: SnarkJobId,
        job: Box<RpcScanStateSummaryScanStateJobKind>,
        seq_no: u64,
        commitment: Option<Box<JobCommitment>>,
        snark: Option<Box<RpcSnarkPoolJobSnarkWork>>,
    },
    Done {
        job_id: SnarkJobId,
        bundle_job_id: SnarkJobId,
        job: Box<RpcScanStateSummaryScanStateJobKind>,
        seq_no: u64,
        snark: Box<RpcSnarkPoolJobSnarkWorkDone>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum RpcScanStateSummaryScanStateJobKind {
    Base(RpcScanStateSummaryBlockTransaction),
    Merge,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RpcScanStateSummaryScanStateJobStatus {
    Todo,
    Done,
}

#[derive(Serialize, Debug, Clone)]
pub struct RpcSnarkPoolJobSummary {
    pub time: Timestamp,
    pub id: SnarkJobId,
    pub commitment: Option<JobCommitment>,
    pub snark: Option<RpcSnarkPoolJobSnarkWork>,
}

#[derive(Serialize, Debug, Clone)]
pub struct RpcSnarkPoolJobFull {
    pub time: Timestamp,
    pub id: SnarkJobId,
    pub job: OneOrTwo<AvailableJobMessage>,
    pub commitment: Option<JobCommitment>,
    pub snark: Option<RpcSnarkPoolJobSnarkWork>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcSnarkPoolJobSnarkWork {
    pub snarker: NonZeroCurvePoint,
    pub fee: CurrencyFeeStableV1,
    pub received_t: Timestamp,
    pub sender: PeerId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcSnarkPoolJobSnarkWorkDone {
    pub snarker: NonZeroCurvePoint,
    pub fee: CurrencyFeeStableV1,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum RpcSnarkerJobCommitResponse {
    Ok,
    JobNotFound,
    JobTaken,
    SnarkerBusy,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum RpcSnarkerJobSpecResponse {
    Ok(SnarkWorkerWorkerRpcsVersionedGetWorkV2TResponse),
    Err(SnarkWorkSpecError),
    JobNotFound,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcMessageProgressResponse {
    pub messages_stats: BTreeMap<PeerId, MessagesStats>,
    pub staking_ledger_sync: Option<LedgerSyncProgress>,
    pub next_epoch_ledger_sync: Option<LedgerSyncProgress>,
    pub root_ledger_sync: Option<LedgerSyncProgress>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessagesStats {
    pub current_request: Option<CurrentMessageProgress>,
    pub responses: BTreeMap<String, usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LedgerSyncProgress {
    pub fetched: u64,
    pub estimation: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CurrentMessageProgress {
    pub name: String,
    pub received_bytes: usize,
    pub total_bytes: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, thiserror::Error)]
pub enum RpcStateGetError {
    #[error("failed to parse filter expression: {0}")]
    FilterError(String),
}

pub type RpcStateGetResponse = Result<serde_json::Value, RpcStateGetError>;
pub type RpcStatusGetResponse = Option<RpcNodeStatus>;
pub type RpcActionStatsGetResponse = Option<ActionStatsResponse>;
pub type RpcSyncStatsGetResponse = Option<Vec<SyncStatsSnapshot>>;
pub type RpcBlockProducerStatsGetResponse = Option<RpcBlockProducerStats>;
pub type RpcPeersGetResponse = Vec<RpcPeerInfo>;
pub type RpcP2pConnectionOutgoingResponse = Result<(), String>;
pub type RpcScanStateSummaryGetResponse = Result<RpcScanStateSummary, String>;
pub type RpcSnarkPoolGetResponse = Vec<RpcSnarkPoolJobSummary>;
pub type RpcSnarkPoolJobGetResponse = Option<RpcSnarkPoolJobFull>;
pub type RpcSnarkerConfigGetResponse = Option<RpcSnarkerConfig>;
pub type RpcTransactionPoolResponse = Vec<ValidCommandWithHash>;
pub type RpcLedgerSlimAccountsResponse = Vec<AccountSlim>;
pub type RpcLedgerAccountsResponse = Vec<Account>;
pub type RpcTransitionFrontierUserCommandsResponse = Vec<MinaBaseUserCommandStableV2>;
pub type RpcBestChainResponse = Vec<ArcBlockWithHash>;

// TODO(adonagy): rework this to handle all the possible user commands (enum..)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcTransactionInjectedPayment {
    pub amount: Amount,
    pub fee: Fee,
    // pub fee_token: TokenId,
    pub from: AccountPublicKey,
    pub to: AccountPublicKey,
    pub hash: String, // TODO(adonagy)
    // pub id: String, // TODO(adonagy)
    pub is_delegation: bool,
    pub memo: String, // TODO(adonagy)
    // pub memo: Memo, // TODO(adonagy)
    pub nonce: Nonce,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RpcTransactionInjectedCommand {
    Payment(RpcTransactionInjectedPayment),
    Delegation,
    Zkapp(valid::UserCommand),
}

pub type RpcTransactionInjectSuccess = Vec<RpcTransactionInjectedCommand>;
pub type RpcTransactionInjectRejected = Vec<(RpcTransactionInjectedCommand, diff::Error)>;
/// Errors
pub type RpcTransactionInjectFailure = Vec<String>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RpcTransactionInjectResponse {
    Success(RpcTransactionInjectSuccess),
    Rejected(RpcTransactionInjectRejected),
    Failure(RpcTransactionInjectFailure),
}

impl From<ValidCommandWithHash> for RpcTransactionInjectedCommand {
    fn from(value: ValidCommandWithHash) -> Self {
        match value.data {
            transaction_logic::valid::UserCommand::SignedCommand(signedcmd) => {
                match signedcmd.payload.body {
                    transaction_logic::signed_command::Body::Payment(ref payment) => {
                        Self::Payment(RpcTransactionInjectedPayment {
                            amount: payment.amount,
                            fee: signedcmd.fee(),
                            // fee_token: signedcmd.fee_token(),
                            from: signedcmd.fee_payer_pk().clone().into(),
                            to: payment.receiver_pk.clone().into(),
                            hash: TransactionHash::from(value.hash.as_ref()).to_string(),
                            is_delegation: false,
                            // memo: signedcmd.payload.common.memo.clone(),
                            memo: signedcmd.payload.common.memo.to_string(),
                            nonce: signedcmd.nonce(),
                        })
                    }
                    transaction_logic::signed_command::Body::StakeDelegation(_) => {
                        todo!("inject stake delegation")
                    }
                }
            }
            transaction_logic::valid::UserCommand::ZkAppCommand(_) => {
                Self::Zkapp(value.data.clone())
            }
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct AccountSlim {
    pub public_key: AccountPublicKey,
    pub balance: Balance,
    pub nonce: Nonce,
}

impl From<Account> for AccountSlim {
    fn from(value: Account) -> Self {
        Self {
            public_key: AccountPublicKey::from(value.public_key),
            balance: value.balance,
            nonce: value.nonce,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct RpcNodeStatus {
    pub chain_id: Option<String>,
    pub transition_frontier: RpcNodeStatusTransitionFrontier,
    pub peers: Vec<RpcPeerInfo>,
    pub snark_pool: RpcNodeStatusSnarkPool,
    pub transaction_pool: RpcNodeStatusTransactionPool,
}

#[derive(Serialize, Debug, Clone)]
pub struct RpcNodeStatusTransitionFrontier {
    pub best_tip: Option<RpcNodeStatusTransitionFrontierBlockSummary>,
    pub sync: RpcNodeStatusTransitionFrontierSync,
}

#[derive(Serialize, Debug, Clone)]
pub struct RpcNodeStatusTransitionFrontierSync {
    pub time: Option<redux::Timestamp>,
    pub status: String,
    pub phase: String,
    pub target: Option<RpcNodeStatusTransitionFrontierBlockSummary>,
}

#[derive(Serialize, Debug, Clone)]
pub struct RpcNodeStatusTransitionFrontierBlockSummary {
    pub hash: StateHash,
    pub height: u32,
    pub global_slot: u32,
}

#[derive(Serialize, Debug, Default, Clone)]
pub struct RpcNodeStatusTransactionPool {
    pub transactions: usize,
}

#[derive(Serialize, Debug, Default, Clone)]
pub struct RpcNodeStatusSnarkPool {
    pub total_jobs: usize,
    pub snarks: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcBlockProducerStats {
    pub current_time: redux::Timestamp,
    pub current_global_slot: Option<u32>,
    pub epoch_start: Option<u32>,
    pub epoch_end: Option<u32>,
    pub attempts: Vec<BlockProductionAttempt>,
    pub future_won_slots: Vec<BlockProductionAttemptWonSlot>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcSnarkerConfig {
    public_key: NonZeroCurvePoint,
    fee: CurrencyFeeStableV1,
}

#[derive(Serialize, Debug, Clone)]
pub struct RpcSnarkWorker {
    pub time: Option<Timestamp>,
    pub id: Option<String>,
    pub status: RpcSnarkWorkerStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum RpcSnarkWorkerStatus {
    None,
    Starting,
    Idle,
    Working {
        job_id: SnarkJobId,
        summary: JobSummary,
    },
    WorkReady {
        job_id: SnarkJobId,
    },
    WorkError {
        job_id: SnarkJobId,
        error: ExternalSnarkWorkerWorkError,
    },
    Cancelling {
        job_id: SnarkJobId,
    },
    Cancelled {
        job_id: SnarkJobId,
    },
    Error {
        error: ExternalSnarkWorkerError,
        permanent: bool,
    },
    Killing,
}

pub type RpcSnarkerWorkersResponse = Vec<RpcSnarkWorker>;

impl From<&MinaTransactionTransactionStableV2> for RpcScanStateSummaryBlockTransactionKind {
    fn from(value: &MinaTransactionTransactionStableV2) -> Self {
        match value {
            MinaTransactionTransactionStableV2::Command(v) => (&**v).into(),
            MinaTransactionTransactionStableV2::FeeTransfer(_) => Self::FeeTransfer,
            MinaTransactionTransactionStableV2::Coinbase(_) => Self::Coinbase,
        }
    }
}

impl From<&MinaBaseUserCommandStableV2> for RpcScanStateSummaryBlockTransactionKind {
    fn from(value: &MinaBaseUserCommandStableV2) -> Self {
        match value {
            MinaBaseUserCommandStableV2::SignedCommand(v) => match &v.payload.body {
                MinaBaseSignedCommandPayloadBodyStableV2::Payment(_) => Self::Payment,
                MinaBaseSignedCommandPayloadBodyStableV2::StakeDelegation(_) => {
                    Self::StakeDelegation
                }
            },
            MinaBaseUserCommandStableV2::ZkappCommand(_) => Self::Zkapp,
        }
    }
}

pub type RpcHealthCheckResponse = Result<(), String>;
pub type RpcReadinessCheckResponse = Result<(), String>;

pub type RpcDiscoveryRoutingTableResponse = Option<discovery::RpcDiscoveryRoutingTable>;
pub type RpcDiscoveryBoostrapStatsResponse = Option<P2pNetworkKadBootstrapStats>;

pub mod discovery {
    use p2p::{
        libp2p_identity::DecodingError, ConnectionType, P2pNetworkKadBucket, P2pNetworkKadDist,
        P2pNetworkKadEntry, P2pNetworkKadKey, P2pNetworkKadRoutingTable, PeerId,
    };
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct RpcDiscoveryRoutingTable {
        this_key: P2pNetworkKadKey,
        buckets: Vec<RpcKBucket>,
    }

    impl TryFrom<&P2pNetworkKadRoutingTable> for RpcDiscoveryRoutingTable {
        type Error = DecodingError;

        fn try_from(value: &P2pNetworkKadRoutingTable) -> Result<Self, Self::Error> {
            let mut buckets = Vec::new();

            for (i, b) in value.buckets.iter().enumerate() {
                buckets.push((b, P2pNetworkKadDist::from(i), &value.this_key).try_into()?);
            }

            Ok(RpcDiscoveryRoutingTable {
                this_key: value.this_key.clone(),
                buckets,
            })
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct RpcKBucket {
        max_dist: P2pNetworkKadDist,
        entries: Vec<RpcEntry>,
    }

    impl<const K: usize>
        TryFrom<(
            &P2pNetworkKadBucket<K>,
            P2pNetworkKadDist,
            &P2pNetworkKadKey,
        )> for RpcKBucket
    {
        type Error = DecodingError;

        fn try_from(
            (bucket, max_dist, this_key): (
                &P2pNetworkKadBucket<K>,
                P2pNetworkKadDist,
                &P2pNetworkKadKey,
            ),
        ) -> Result<Self, Self::Error> {
            let mut entries = Vec::new();

            for entry in bucket.iter() {
                entries.push((entry, this_key).try_into()?);
            }
            Ok(RpcKBucket { max_dist, entries })
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct RpcEntry {
        peer_id: PeerId,
        libp2p: p2p::libp2p_identity::PeerId,
        key: P2pNetworkKadKey,
        dist: P2pNetworkKadDist,
        addrs: Vec<p2p::multiaddr::Multiaddr>,
        connection: ConnectionType,
    }

    impl TryFrom<(&P2pNetworkKadEntry, &P2pNetworkKadKey)> for RpcEntry {
        type Error = DecodingError;

        fn try_from(
            (value, this_key): (&P2pNetworkKadEntry, &P2pNetworkKadKey),
        ) -> Result<Self, Self::Error> {
            Ok(RpcEntry {
                peer_id: value.peer_id,
                libp2p: value.peer_id.try_into()?,
                key: value.key.clone(),
                dist: this_key - &value.key,
                addrs: value.addresses().clone(),
                connection: value.connection,
            })
        }
    }
}
