use account::{create_account_loader, AccountLoader};
use block::{GraphQLBlock, GraphQLSnarkJob, GraphQLUserCommands};
use juniper::{graphql_value, EmptySubscription, FieldError, GraphQLEnum, RootNode};
use ledger::Account;
use mina_p2p_messages::v2::{
    conv, LedgerHash, MinaBaseSignedCommandStableV2, MinaBaseUserCommandStableV2,
    MinaBaseZkappCommandTStableV1WireStableV1, TokenIdKeyHash, TransactionHash,
};
use node::{
    account::AccountPublicKey,
    ledger::read::LedgerStatus,
    rpc::{
        AccountQuery, GetBlockQuery, PooledCommandsQuery, RpcGenesisBlockResponse,
        RpcGetBlockResponse, RpcPooledUserCommandsResponse, RpcPooledZkappCommandsResponse,
        RpcRequest, RpcSnarkPoolCompletedJobsResponse, RpcSnarkPoolPendingJobsGetResponse,
        RpcSyncStatsGetResponse, RpcTransactionInjectResponse, RpcTransactionStatusGetResponse,
        SyncStatsQuery, RpcStatusGetResponse, RpcNodeStatus, RpcBestChainResponse,
        RpcLedgerStatusGetResponse
    },
    stats::sync::SyncKind,
    BuildEnv,
};
use o1_utils::field_helpers::FieldHelpersError;
use openmina_core::{
    block::AppliedBlock, consensus::ConsensusConstants, constants::constraint_constants,
    NetworkConfig,
};
use openmina_node_common::rpc::RpcSender;
use snark::GraphQLPendingSnarkWork;
use std::str::FromStr;
use tokio::sync::OnceCell;
use transaction::GraphQLTransactionStatus;
use warp::{Filter, Rejection, Reply};
use zkapp::GraphQLZkapp;

pub mod account;
pub mod block;
pub mod constants;
pub mod snark;
pub mod transaction;
pub mod user_command;
pub mod zkapp;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Conversion error: {0}")]
    Conversion(ConversionError),
    #[error("State machine empty response")]
    StateMachineEmptyResponse,
    #[error("Custom: {0}")]
    Custom(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Conversion(#[from] mina_p2p_messages::v2::conv::Error),
    #[error("Wrong variant")]
    WrongVariant,
    #[error("SerdeJson: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Base58Check: {0}")]
    Base58Check(#[from] mina_p2p_messages::b58::FromBase58CheckError),
    #[error("Base58 error: {0}")]
    Base58(#[from] bs58::decode::Error),
    #[error(transparent)]
    InvalidDecimalNumber(#[from] mina_p2p_messages::bigint::InvalidDecimalNumber),
    #[error("Invalid bigint")]
    InvalidBigInt,
    #[error("Invalid hex")]
    InvalidHex,
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error(transparent)]
    EnumParse(#[from] strum::ParseError),
    #[error(transparent)]
    TryFromInt(#[from] std::num::TryFromIntError),
    #[error("Missing field: {0}")]
    MissingField(String),
    #[error("Invalid length")]
    InvalidLength,
    #[error("Custom: {0}")]
    Custom(String),
    #[error(transparent)]
    FieldHelpers(#[from] FieldHelpersError),
    #[error("Failed to convert integer to i32")]
    Integer,
}

impl From<ConversionError> for Error {
    fn from(value: ConversionError) -> Self {
        Error::Conversion(value)
    }
}

/// Context for the GraphQL API
///
/// This is used to share state between the GraphQL queries and mutations.
///
/// The caching used here is only valid for the lifetime of the context
/// i.e. for one request which is the goal as we can have multiple sources for one request.
/// This optimizes the number of request to the state machine
pub(crate) struct Context {
    rpc_sender: RpcSender,
    statemachine_status_cache: OnceCell<Option<RpcNodeStatus>>,
    best_tip_cache: OnceCell<Option<AppliedBlock>>,
    ledger_status_cache: OnceCell<Option<LedgerStatus>>,
    account_loader: AccountLoader,
}

impl juniper::Context for Context {}

impl Context {
    pub fn new(rpc_sender: RpcSender) -> Self {
        Self {
            rpc_sender: rpc_sender.clone(),
            statemachine_status_cache: OnceCell::new(),
            best_tip_cache: OnceCell::new(),
            ledger_status_cache: OnceCell::new(),
            account_loader: create_account_loader(rpc_sender),
        }
    }

    pub(crate) async fn get_or_fetch_status(&self) -> RpcStatusGetResponse {
        self.statemachine_status_cache
            .get_or_init(|| async {
                self.rpc_sender
                    .oneshot_request(RpcRequest::StatusGet)
                    .await
                    .flatten()
            })
            .await
            .clone()
    }

    pub(crate) async fn get_or_fetch_best_tip(&self) -> Option<AppliedBlock> {
        self.best_tip_cache
            .get_or_init(|| async {
                self.rpc_sender
                    .oneshot_request(RpcRequest::BestChain(1))
                    .await
                    .and_then(|blocks: RpcBestChainResponse| blocks.first().cloned())
            })
            .await
            .clone()
    }

    pub(crate) async fn get_or_fetch_ledger_status(
        &self,
        ledger_hash: &LedgerHash,
    ) -> RpcLedgerStatusGetResponse {
        self.ledger_status_cache
            .get_or_init(|| async {
                self.rpc_sender
                    .oneshot_request(RpcRequest::LedgerStatusGet(ledger_hash.clone()))
                    .await
                    .flatten()
            })
            .await
            .clone()
    }
}

#[derive(Clone, Copy, Debug, GraphQLEnum)]
#[allow(clippy::upper_case_acronyms)]
enum SyncStatus {
    CONNECTING,
    LISTENING,
    OFFLINE,
    BOOTSTRAP,
    SYNCED,
    CATCHUP,
}

#[derive(Clone, Debug)]
struct ProtocolState {
    consensus_state: ConsensusState,
    blockchain_state: BlockchainState,
}

#[juniper::graphql_object(context = Context)]
impl ProtocolState {
    fn consensus_state(&self) -> &ConsensusState {
        &self.consensus_state
    }

    fn blockchain_state(&self) -> &BlockchainState {
        &self.blockchain_state
    }
}

#[derive(Clone, Debug)]
struct ConsensusState {
    block_height: i32,
}

#[juniper::graphql_object(context = Context)]
impl ConsensusState {
    fn block_height(&self) -> i32 {
        self.block_height
    }
}

#[derive(Clone, Debug)]
struct BlockchainState {
    snarked_ledger_hash: String,
}

#[juniper::graphql_object(context = Context)]
impl BlockchainState {
    fn snarked_ledger_hash(&self) -> &str {
        &self.snarked_ledger_hash
    }
}

#[derive(Clone, Debug)]
struct BestChain {
    state_hash: String,
    protocol_state: ProtocolState,
}

#[juniper::graphql_object(context = Context)]
impl BestChain {
    fn state_hash(&self) -> &str {
        &self.state_hash
    }

    fn protocol_state(&self) -> &ProtocolState {
        &self.protocol_state
    }
}

#[derive(Clone, Copy, Debug)]
struct Query;

#[juniper::graphql_object(context = Context)]
impl Query {
    async fn account(
        public_key: String,
        token: Option<String>,
        context: &Context,
    ) -> juniper::FieldResult<account::GraphQLAccount> {
        let public_key = AccountPublicKey::from_str(&public_key)?;
        let req = match token {
            None => AccountQuery::SinglePublicKey(public_key),
            Some(token) => {
                let token_id = TokenIdKeyHash::from_str(&token)?;
                AccountQuery::PubKeyWithTokenId(public_key, token_id)
            }
        };
        let accounts: Vec<Account> = context
            .rpc_sender
            .oneshot_request(RpcRequest::LedgerAccountsGet(req))
            .await
            .ok_or(Error::StateMachineEmptyResponse)?;

        Ok(accounts
            .first()
            .cloned()
            .ok_or(Error::StateMachineEmptyResponse)?
            .try_into()?)
    }

    async fn sync_status(context: &Context) -> juniper::FieldResult<SyncStatus> {
        let state: RpcSyncStatsGetResponse = context
            .rpc_sender
            .oneshot_request(RpcRequest::SyncStatsGet(SyncStatsQuery { limit: Some(1) }))
            .await
            .ok_or(Error::StateMachineEmptyResponse)?;

        if let Some(state) = state.as_ref().and_then(|s| s.first()) {
            if state.synced.is_some() {
                Ok(SyncStatus::SYNCED)
            } else {
                match &state.kind {
                    SyncKind::Bootstrap => Ok(SyncStatus::BOOTSTRAP),
                    SyncKind::Catchup => Ok(SyncStatus::CATCHUP),
                }
            }
        } else {
            Ok(SyncStatus::LISTENING)
        }
    }

    async fn best_chain(
        max_length: i32,
        context: &Context,
    ) -> juniper::FieldResult<Vec<GraphQLBlock>> {
        let best_chain: Vec<AppliedBlock> = context
            .rpc_sender
            .oneshot_request(RpcRequest::BestChain(max_length as u32))
            .await
            .ok_or(Error::StateMachineEmptyResponse)?;

        Ok(best_chain
            .into_iter()
            .map(|v| v.try_into())
            .collect::<Result<Vec<_>, _>>()?)
    }

    async fn daemon_status(
        _context: &Context,
    ) -> juniper::FieldResult<constants::GraphQLDaemonStatus> {
        Ok(constants::GraphQLDaemonStatus)
    }

    async fn genesis_constants(
        context: &Context,
    ) -> juniper::FieldResult<constants::GraphQLGenesisConstants> {
        let consensus_constants: ConsensusConstants = context
            .rpc_sender
            .oneshot_request(RpcRequest::ConsensusConstantsGet)
            .await
            .ok_or(Error::StateMachineEmptyResponse)?;
        let constraint_constants = constraint_constants();

        Ok(constants::GraphQLGenesisConstants::try_new(
            constraint_constants.clone(),
            consensus_constants,
        )?)
    }

    async fn transaction_status(
        payment: Option<String>,
        zkapp_transaction: Option<String>,
        context: &Context,
    ) -> juniper::FieldResult<GraphQLTransactionStatus> {
        if payment.is_some() && zkapp_transaction.is_some() {
            return Err(Error::Custom(
                "Cannot provide both payment and zkapp transaction".to_string(),
            )
            .into());
        }

        let tx = if let Some(payment) = payment {
            MinaBaseUserCommandStableV2::SignedCommand(MinaBaseSignedCommandStableV2::from_base64(
                &payment,
            )?)
        } else if let Some(zkapp_transaction) = zkapp_transaction {
            MinaBaseUserCommandStableV2::ZkappCommand(
                MinaBaseZkappCommandTStableV1WireStableV1::from_base64(&zkapp_transaction)?,
            )
        } else {
            return Err(Error::Custom(
                "Must provide either payment or zkapp transaction".to_string(),
            )
            .into());
        };
        let res: RpcTransactionStatusGetResponse = context
            .rpc_sender
            .oneshot_request(RpcRequest::TransactionStatusGet(tx))
            .await
            .ok_or(Error::StateMachineEmptyResponse)?;

        Ok(GraphQLTransactionStatus::from(res))
    }

    /// Retrieve a block with the given state hash or height, if contained in the transition frontier
    async fn block(
        height: Option<i32>,
        state_hash: Option<String>,
        context: &Context,
    ) -> juniper::FieldResult<GraphQLBlock> {
        let query = match (height, state_hash) {
            (Some(height), None) => GetBlockQuery::Height(height.try_into().unwrap_or(u32::MAX)),
            (None, Some(state_hash)) => GetBlockQuery::Hash(state_hash.parse()?),
            _ => {
                return Err(Error::Custom(
                    "Must provide exactly one of state hash, height".to_owned(),
                )
                .into());
            }
        };

        let res: Option<RpcGetBlockResponse> = context
            .rpc_sender
            .oneshot_request(RpcRequest::GetBlock(query.clone()))
            .await;

        match res {
            None => Err(Error::Custom("response channel dropped".to_owned()).into()),
            Some(None) => match query {
                GetBlockQuery::Hash(hash) => Err(Error::Custom(format!(
                    "Could not find block with hash: `{}` in transition frontier",
                    hash
                ))
                .into()),
                GetBlockQuery::Height(height) => Err(Error::Custom(format!(
                    "Could not find block with height: `{}` in transition frontier",
                    height
                ))
                .into()),
            },
            Some(Some(block)) => Ok(GraphQLBlock::try_from(block)?),
        }
    }

    /// Retrieve all the scheduled user commands for a specified sender that
    /// the current daemon sees in its transaction pool. All scheduled
    /// commands are queried if no sender is specified
    ///
    /// Arguments:
    /// - `public_key`: base58 encoded [`AccountPublicKey`]
    /// - `hashes`: list of base58 encoded [`TransactionHash`]es
    /// - `ids`: list of base64 encoded [`MinaBaseZkappCommandTStableV1WireStableV1`]
    async fn pooled_user_commands(
        &self,
        public_key: Option<String>,
        hashes: Option<Vec<String>>,
        ids: Option<Vec<String>>,
        context: &Context,
    ) -> juniper::FieldResult<Vec<GraphQLUserCommands>> {
        let query = parse_pooled_commands_query(
            public_key,
            hashes,
            ids,
            MinaBaseSignedCommandStableV2::from_base64,
        )?;

        let res: RpcPooledUserCommandsResponse = context
            .rpc_sender
            .oneshot_request(RpcRequest::PooledUserCommands(query))
            .await
            .ok_or(Error::StateMachineEmptyResponse)?;

        Ok(res
            .into_iter()
            .map(GraphQLUserCommands::try_from)
            .collect::<Result<Vec<_>, _>>()?)
    }

    /// Retrieve all the scheduled zkApp commands for a specified sender that
    ///  the current daemon sees in its transaction pool. All scheduled
    ///  commands are queried if no sender is specified
    ///
    /// Arguments:
    /// - `public_key`: base58 encoded [`AccountPublicKey`]
    /// - `hashes`: list of base58 encoded [`TransactionHash`]es
    /// - `ids`: list of base64 encoded [`MinaBaseZkappCommandTStableV1WireStableV1`]
    async fn pooled_zkapp_commands(
        public_key: Option<String>,
        hashes: Option<Vec<String>>,
        ids: Option<Vec<String>>,
        context: &Context,
    ) -> juniper::FieldResult<Vec<GraphQLZkapp>> {
        let query = parse_pooled_commands_query(
            public_key,
            hashes,
            ids,
            MinaBaseZkappCommandTStableV1WireStableV1::from_base64,
        )?;

        let res: RpcPooledZkappCommandsResponse = context
            .rpc_sender
            .oneshot_request(RpcRequest::PooledZkappCommands(query))
            .await
            .ok_or(Error::StateMachineEmptyResponse)?;

        Ok(res
            .into_iter()
            .map(GraphQLZkapp::try_from)
            .collect::<Result<Vec<_>, _>>()?)
    }

    async fn genesis_block(context: &Context) -> juniper::FieldResult<GraphQLBlock> {
        let block = context
            .rpc_sender
            .oneshot_request::<RpcGenesisBlockResponse>(RpcRequest::GenesisBlockGet)
            .await
            .ok_or(Error::StateMachineEmptyResponse)?
            .ok_or(Error::StateMachineEmptyResponse)?;

        Ok(GraphQLBlock::try_from(AppliedBlock {
            block,
            just_emitted_a_proof: false,
        })?)
    }

    async fn snark_pool(context: &Context) -> juniper::FieldResult<Vec<GraphQLSnarkJob>> {
        let jobs: RpcSnarkPoolCompletedJobsResponse = context
            .rpc_sender
            .oneshot_request(RpcRequest::SnarkPoolCompletedJobsGet)
            .await
            .ok_or(Error::StateMachineEmptyResponse)?;

        Ok(jobs.iter().map(GraphQLSnarkJob::from).collect())
    }

    async fn pending_snark_work(
        context: &Context,
    ) -> juniper::FieldResult<Vec<GraphQLPendingSnarkWork>> {
        let jobs: RpcSnarkPoolPendingJobsGetResponse = context
            .rpc_sender
            .oneshot_request(RpcRequest::SnarkPoolPendingJobsGet)
            .await
            .ok_or(Error::StateMachineEmptyResponse)?;

        Ok(jobs
            .into_iter()
            .map(GraphQLPendingSnarkWork::try_from)
            .collect::<Result<Vec<_>, _>>()?)
    }

    /// The chain-agnostic identifier of the network
    #[graphql(name = "networkID")]
    async fn network_id(_context: &Context) -> juniper::FieldResult<String> {
        let res = format!("mina:{}", NetworkConfig::global().name);
        Ok(res)
    }

    /// The version of the node (git commit hash)
    async fn version(_context: &Context) -> juniper::FieldResult<String> {
        let res = BuildEnv::get().git.commit_hash;
        Ok(res)
    }
}

async fn inject_tx<R>(
    cmd: MinaBaseUserCommandStableV2,
    context: &Context,
) -> juniper::FieldResult<R>
where
    R: TryFrom<MinaBaseUserCommandStableV2>,
{
    let res: RpcTransactionInjectResponse = context
        .rpc_sender
        .oneshot_request(RpcRequest::TransactionInject(vec![cmd]))
        .await
        .ok_or(Error::StateMachineEmptyResponse)?;

    match res {
        RpcTransactionInjectResponse::Success(res) => {
            let cmd: MinaBaseUserCommandStableV2 = match res.first().cloned() {
                Some(cmd) => cmd.into(),
                _ => unreachable!(),
            };
            cmd.try_into().map_err(|_| {
                FieldError::new(
                    "Failed to convert transaction to the required type".to_string(),
                    graphql_value!(null),
                )
            })
        }
        RpcTransactionInjectResponse::Rejected(rejected) => {
            let error_list = rejected
                .into_iter()
                .map(|(_, err)| graphql_value!({ "message": err.to_string() }))
                .collect::<Vec<_>>();

            Err(FieldError::new(
                "Transaction rejected",
                graphql_value!(juniper::Value::List(error_list)),
            ))
        }
        RpcTransactionInjectResponse::Failure(failure) => {
            let error_list = failure
                .into_iter()
                .map(|err| graphql_value!({ "message": err.to_string() }))
                .collect::<Vec<_>>();

            Err(FieldError::new(
                "Transaction failed",
                graphql_value!(juniper::Value::List(error_list)),
            ))
        }
    }
}

#[derive(Clone, Debug)]
struct Mutation;

#[juniper::graphql_object(context = Context)]
impl Mutation {
    async fn send_zkapp(
        input: zkapp::SendZkappInput,
        context: &Context,
    ) -> juniper::FieldResult<zkapp::GraphQLSendZkappResponse> {
        inject_tx(input.try_into()?, context).await
    }

    async fn send_payment(
        input: user_command::InputGraphQLPayment,
        signature: user_command::UserCommandSignature,
        context: &Context,
    ) -> juniper::FieldResult<user_command::GraphQLSendPaymentResponse> {
        // Grab the sender's account to get the infered nonce
        let token_id = TokenIdKeyHash::default();
        let public_key = AccountPublicKey::from_str(&input.from)
            .map_err(|e| Error::Conversion(ConversionError::Base58Check(e)))?;

        let accounts: Vec<Account> = context
            .rpc_sender
            .oneshot_request(RpcRequest::LedgerAccountsGet(
                AccountQuery::PubKeyWithTokenId(public_key, token_id),
            ))
            .await
            .ok_or(Error::StateMachineEmptyResponse)?;

        let infered_nonce = accounts
            .first()
            .ok_or(Error::StateMachineEmptyResponse)?
            .nonce;

        let command = input
            .create_user_command(infered_nonce, signature)
            .map_err(Error::Conversion)?;

        inject_tx(command, context).await
    }

    async fn send_delegation(
        input: user_command::InputGraphQLDelegation,
        signature: user_command::UserCommandSignature,
        context: &Context,
    ) -> juniper::FieldResult<user_command::GraphQLSendDelegationResponse> {
        // Payment commands are always for the default (MINA) token
        let token_id = TokenIdKeyHash::default();
        let public_key = AccountPublicKey::from_str(&input.from)?;

        // Grab the sender's account to get the infered nonce
        let accounts: Vec<Account> = context
            .rpc_sender
            .oneshot_request(RpcRequest::LedgerAccountsGet(
                AccountQuery::PubKeyWithTokenId(public_key, token_id),
            ))
            .await
            .ok_or(Error::StateMachineEmptyResponse)?;

        let infered_nonce = accounts
            .first()
            .ok_or(Error::StateMachineEmptyResponse)?
            .nonce;
        let command = input.create_user_command(infered_nonce, signature)?;

        inject_tx(command, context).await
    }
}

pub fn routes(
    rpc_sernder: RpcSender,
) -> impl Filter<Error = Rejection, Extract = impl Reply> + Clone {
    let state = warp::any().map(move || Context::new(rpc_sernder.clone()));
    let schema = RootNode::new(Query, Mutation, EmptySubscription::<Context>::new());
    let graphql_filter = juniper_warp::make_graphql_filter(schema, state.boxed());
    let graphiql_filter = juniper_warp::graphiql_filter("/graphql", None);
    let playground_filter = juniper_warp::playground_filter("/graphql", None);

    (warp::post().and(warp::path("graphql")).and(graphql_filter))
        .or(warp::get()
            .and(warp::path("playground"))
            .and(playground_filter))
        .or(warp::get().and(warp::path("graphiql")).and(graphiql_filter))

    // warp::get()
    //     .and(warp::path("graphiql"))
    //     .and(juniper_warp::graphiql_filter("/graphql", None))
    //     .or(warp::path("graphql").and(graphql_filter))
}

// let routes = (warp::post()
//         .and(warp::path("graphql"))
//         .and(juniper_warp::make_graphql_filter(
//             schema.clone(),
//             warp::any().map(|| Context),
//         )))
//     .or(
//         warp::path("subscriptions").and(juniper_warp::subscriptions::make_ws_filter(
//             schema,
//             ConnectionConfig::new(Context),
//         )),
//     )
//     .or(warp::get()
//         .and(warp::path("playground"))
//         .and(juniper_warp::playground_filter(
//             "/graphql",
//             Some("/subscriptions"),
//         )))
//     .or(warp::get()
//         .and(warp::path("graphiql"))
//         .and(juniper_warp::graphiql_filter(
//             "/graphql",
//             Some("/subscriptions"),
//         )))
//     .or(homepage)
//     .with(log);

/// Helper function used by [`Query::pooled_user_commands`] and [`Query::pooled_zkapp_commands`] to parse public key, transaction hashes and command ids
fn parse_pooled_commands_query<ID, F>(
    public_key: Option<String>,
    hashes: Option<Vec<String>>,
    ids: Option<Vec<String>>,
    id_map_fn: F,
) -> Result<PooledCommandsQuery<ID>, ConversionError>
where
    F: Fn(&str) -> Result<ID, conv::Error>,
{
    let public_key = match public_key {
        Some(public_key) => Some(AccountPublicKey::from_str(&public_key)?),
        None => None,
    };

    let hashes = match hashes {
        Some(hashes) => Some(
            hashes
                .into_iter()
                .map(|tx| TransactionHash::from_str(tx.as_str()))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        None => None,
    };

    let ids = match ids {
        Some(ids) => Some(
            ids.into_iter()
                .map(|id| id_map_fn(id.as_str()))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        None => None,
    };

    Ok(PooledCommandsQuery {
        public_key,
        hashes,
        ids,
    })
}
