use juniper::{EmptyMutation, EmptySubscription, GraphQLEnum, RootNode};
use snarker::{
    rpc::{RpcRequest, RpcSyncStatsGetResponse, SyncStatsQuery},
    stats::sync::SyncKind,
};
use warp::{Filter, Rejection, Reply};

struct Context(super::RpcSender);

impl juniper::Context for Context {}

#[derive(Clone, Copy, Debug, GraphQLEnum)]
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

    async fn best_chain(max_length: i32, context: &Context) -> Vec<BestChain> {
        let state: RpcSyncStatsGetResponse = context
            .0
            .oneshot_request(RpcRequest::SyncStatsGet(SyncStatsQuery {
                limit: Some(max_length as _),
            }))
            .await
            .unwrap();
        state.unwrap_or_default()
            .into_iter()
            .filter_map(|x| {
                let head = x.blocks.first()?;
                let snarked_ledger_hash = x.ledgers.root?.snarked.hash?;
                Some(BestChain {
                    state_hash: head.hash.to_string(),
                    protocol_state: ProtocolState {
                        consensus_state: ConsensusState {
                            block_height: head.height as _,
                        },
                        blockchain_state: BlockchainState {
                            snarked_ledger_hash: snarked_ledger_hash.to_string(),
                        },
                    },
                })
            })
            .collect()
    }
}

pub fn routes(
    rpc_sernder: super::RpcSender,
) -> impl Filter<Error = Rejection, Extract = impl Reply> + Clone {
    let state = warp::any().map(move || Context(rpc_sernder.clone()));
    let schema = RootNode::new(
        Query,
        EmptyMutation::<Context>::new(),
        EmptySubscription::<Context>::new(),
    );
    let graphql_filter = juniper_warp::make_graphql_filter(schema, state.boxed());

    warp::get()
        .and(warp::path("graphiql"))
        .and(juniper_warp::graphiql_filter("/graphql", None))
        .or(warp::path("graphql").and(graphql_filter))
}
