use std::str::FromStr;

use juniper::{EmptyMutation, EmptySubscription, GraphQLEnum, RootNode};
use ledger::Account;
use mina_p2p_messages::v2::TokenIdKeyHash;
use node::{
    account::AccountPublicKey,
    ledger::read::LedgerReadRequest,
    rpc::{AccountQuery, RpcRequest, RpcSyncStatsGetResponse, SyncStatsQuery},
    stats::sync::SyncKind,
};
use openmina_core::block::ArcBlockWithHash;
use openmina_node_common::rpc::RpcSender;
use warp::{Filter, Rejection, Reply};

pub mod account;
pub mod best_chain;
pub mod send_zkapp;

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
    async fn best_chain(
        max_length: i32,
        context: &Context,
    ) -> Vec<best_chain::GraphQLBestChainBlock> {
        let best_chain: Vec<ArcBlockWithHash> = context
            .0
            .oneshot_request(RpcRequest::BestChain(max_length as u32))
            .await
            .unwrap();

        best_chain.into_iter().map(|v| v.into()).collect()
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

 struct Mutation;

 #[juniper::graphql_object(context = Context)]
 impl Mutation {
    async fn send_zkapp(input: send_zkapp::SendZkappInput, context: &Context) -> Result<String, String> {
        Ok("".to_string())
    }
 }  

pub fn routes(
    rpc_sernder: RpcSender,
) -> impl Filter<Error = Rejection, Extract = impl Reply> + Clone {
    let state = warp::any().map(move || Context(rpc_sernder.clone()));
    let schema = RootNode::new(
        Query,
        EmptyMutation::<Context>::new(),
        EmptySubscription::<Context>::new(),
    );
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
