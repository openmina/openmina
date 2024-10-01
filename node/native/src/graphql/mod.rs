use std::str::FromStr;

use juniper::{graphql_value, FieldError};
use juniper::{EmptySubscription, GraphQLEnum, RootNode};
use ledger::Account;
use mina_p2p_messages::v2::MinaBaseSignedCommandStableV2;
use mina_p2p_messages::v2::MinaBaseUserCommandStableV2;
use mina_p2p_messages::v2::MinaBaseZkappCommandTStableV1WireStableV1;
use mina_p2p_messages::v2::TokenIdKeyHash;
use node::rpc::RpcTransactionInjectResponse;
use node::rpc::RpcTransactionInjectedCommand;
use node::rpc::RpcTransactionStatusGetResponse;
use node::{
    account::AccountPublicKey,
    rpc::{AccountQuery, RpcRequest, RpcSyncStatsGetResponse, SyncStatsQuery},
    stats::sync::SyncKind,
};
use openmina_core::block::AppliedBlock;
use openmina_core::consensus::ConsensusConstants;
use openmina_core::constants::constraint_constants;
use openmina_node_common::rpc::RpcSender;
use warp::{Filter, Rejection, Reply};

pub mod account;
pub mod block;
pub mod constants;
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
    #[error(transparent)]
    InvalidDecimalNumber(#[from] mina_p2p_messages::bigint::InvalidDecimalNumber),
    #[error("Invalid bigint")]
    InvalidBigInt,
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
}

struct Context(RpcSender);

impl juniper::Context for Context {}

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
        token: String,
        context: &Context,
    ) -> juniper::FieldResult<account::GraphQLAccount> {
        let token_id = TokenIdKeyHash::from_str(&token)?;
        let public_key = AccountPublicKey::from_str(&public_key)?;
        let accounts: Vec<Account> = context
            .0
            .oneshot_request(RpcRequest::LedgerAccountsGet(
                AccountQuery::PubKeyWithTokenId(public_key, token_id),
            ))
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
            .0
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
    ) -> juniper::FieldResult<Vec<block::GraphQLBestChainBlock>> {
        let best_chain: Vec<AppliedBlock> = context
            .0
            .oneshot_request(RpcRequest::BestChain(max_length as u32))
            .await
            .ok_or(Error::StateMachineEmptyResponse)?;

        Ok(best_chain
            .into_iter()
            .map(|v| v.try_into())
            .collect::<Result<Vec<_>, _>>()?)
    }

    async fn daemon_status(
        context: &Context,
    ) -> juniper::FieldResult<constants::GraphQLDaemonStatus> {
        let consensus_constants: ConsensusConstants = context
            .0
            .oneshot_request(RpcRequest::ConsensusConstantsGet)
            .await
            .ok_or(Error::StateMachineEmptyResponse)?;
        Ok(constants::GraphQLDaemonStatus {
            consensus_configuration: consensus_constants.into(),
        })
    }

    async fn genesis_constants(
        context: &Context,
    ) -> juniper::FieldResult<constants::GraphQLGenesisConstants> {
        let consensus_constants: ConsensusConstants = context
            .0
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
    ) -> juniper::FieldResult<String> {
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
            .0
            .oneshot_request(RpcRequest::TransactionStatusGet(tx))
            .await
            .ok_or(Error::StateMachineEmptyResponse)?;
        Ok(res.to_string())
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
        let res: RpcTransactionInjectResponse = context
            .0
            .oneshot_request(RpcRequest::TransactionInject(vec![input.try_into()?]))
            .await
            .ok_or(Error::StateMachineEmptyResponse)?;

        match res {
            RpcTransactionInjectResponse::Success(res) => {
                let zkapp_cmd: MinaBaseUserCommandStableV2 = match res.first().cloned() {
                    Some(RpcTransactionInjectedCommand::Zkapp(zkapp_cmd)) => zkapp_cmd.into(),
                    _ => unreachable!(),
                };
                Ok(zkapp_cmd.try_into()?)
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
}

pub fn routes(
    rpc_sernder: RpcSender,
) -> impl Filter<Error = Rejection, Extract = impl Reply> + Clone {
    let state = warp::any().map(move || Context(rpc_sernder.clone()));
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
