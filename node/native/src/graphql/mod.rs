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
use openmina_core::block::ArcBlockWithHash;
use openmina_core::consensus::ConsensusConstants;
use openmina_core::constants::constraint_constants;
use openmina_node_common::rpc::RpcSender;
use warp::{Filter, Rejection, Reply};

pub mod account;
pub mod block;
pub mod constants;
pub mod zkapp;

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
    ) -> account::GraphQLAccount {
        // TODO(adonagy): error handling
        let token_id = TokenIdKeyHash::from_str(&token).unwrap();
        let public_key = AccountPublicKey::from_str(&public_key).unwrap();
        let accounts: Vec<Account> = context
            .0
            .oneshot_request(RpcRequest::LedgerAccountsGet(
                AccountQuery::PubKeyWithTokenId(public_key, token_id),
            ))
            .await
            .unwrap();

        // Error handling
        accounts.first().cloned().unwrap().into()
    }

    async fn sync_status(context: &Context) -> SyncStatus {
        let state: RpcSyncStatsGetResponse = context
            .0
            .oneshot_request(RpcRequest::SyncStatsGet(SyncStatsQuery { limit: Some(1) }))
            .await
            .unwrap();

        if let Some(state) = state.as_ref().and_then(|s| s.first()) {
            if state.synced.is_some() {
                SyncStatus::SYNCED
            } else {
                match &state.kind {
                    SyncKind::Bootstrap => SyncStatus::BOOTSTRAP,
                    SyncKind::Catchup => SyncStatus::CATCHUP,
                }
            }
        } else {
            SyncStatus::LISTENING
        }
    }
    async fn best_chain(max_length: i32, context: &Context) -> Vec<block::GraphQLBestChainBlock> {
        let best_chain: Vec<ArcBlockWithHash> = context
            .0
            .oneshot_request(RpcRequest::BestChain(max_length as u32))
            .await
            .unwrap();

        best_chain.into_iter().map(|v| v.into()).collect()
    }

    async fn daemon_status(context: &Context) -> constants::GraphQLDaemonStatus {
        let consensus_constants: ConsensusConstants = context
            .0
            .oneshot_request(RpcRequest::ConsensusConstantsGet)
            .await
            .unwrap();
        constants::GraphQLDaemonStatus {
            consensus_configuration: consensus_constants.into(),
        }
    }

    async fn genesis_constants(context: &Context) -> constants::GraphQLGenesisConstants {
        let consensus_constants: ConsensusConstants = context
            .0
            .oneshot_request(RpcRequest::ConsensusConstantsGet)
            .await
            .unwrap();
        let constraint_constants = constraint_constants();

        constants::GraphQLGenesisConstants::new(constraint_constants.clone(), consensus_constants)
    }

    async fn transaction_status(
        payment: Option<String>,
        zkapp_transaction: Option<String>,
        context: &Context,
    ) -> String {
        if payment.is_some() && zkapp_transaction.is_some() {
            panic!("Cannot provide both payment and zkapp transaction");
        }

        let tx = if let Some(payment) = payment {
            MinaBaseUserCommandStableV2::SignedCommand(
                MinaBaseSignedCommandStableV2::from_base64(&payment).unwrap(),
            )
        } else if let Some(zkapp_transaction) = zkapp_transaction {
            MinaBaseUserCommandStableV2::ZkappCommand(
                MinaBaseZkappCommandTStableV1WireStableV1::from_base64(&zkapp_transaction).unwrap(),
            )
        } else {
            panic!("Must provide either payment or zkapp transaction");
        };
        let res: RpcTransactionStatusGetResponse = context
            .0
            .oneshot_request(RpcRequest::TransactionStatusGet(tx))
            .await
            .unwrap();
        res.to_string()
    }
    // async fn best_chain(max_length: i32, context: &Context) -> Vec<BestChain> {
    //     let state: RpcSyncStatsGetResponse = context
    //         .0
    //         .oneshot_request(RpcRequest::SyncStatsGet(SyncStatsQuery {
    //             limit: Some(max_length as _),
    //         }))
    //         .await
    //         .unwrap();
    //     state
    //         .unwrap_or_default()
    //         .into_iter()
    //         .filter_map(|x| {
    //             let head = x.blocks.first()?;
    //             let snarked_ledger_hash = x.ledgers.root?.snarked.hash?;
    //             Some(BestChain {
    //                 state_hash: head.hash.to_string(),
    //                 protocol_state: ProtocolState {
    //                     consensus_state: ConsensusState {
    //                         block_height: head.height as _,
    //                     },
    //                     blockchain_state: BlockchainState {
    //                         snarked_ledger_hash: snarked_ledger_hash.to_string(),
    //                     },
    //                 },
    //             })
    //         })
    //         .collect()
    // }
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
            .oneshot_request(RpcRequest::TransactionInject(vec![input.into()]))
            .await
            .unwrap();

        match res {
            RpcTransactionInjectResponse::Success(res) => {
                let zkapp_cmd: MinaBaseUserCommandStableV2 = match res.first().cloned() {
                    Some(RpcTransactionInjectedCommand::Zkapp(zkapp_cmd)) => zkapp_cmd.into(),
                    _ => unreachable!(),
                };
                Ok(zkapp_cmd.into())
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