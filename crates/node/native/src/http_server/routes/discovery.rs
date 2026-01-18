//! Discovery endpoints.
//!
//! - `GET /discovery/routing_table` - Get Kademlia routing table
//! - `GET /discovery/bootstrap_stats` - Get bootstrap statistics

use axum::{extract::State, routing::get, Json, Router};

use mina_node::rpc::{
    RpcDiscoveryBoostrapStatsResponse, RpcDiscoveryRoutingTableResponse, RpcRequest,
};

use crate::http_server::{AppResult, AppState};

/// Registers discovery routes on the router.
pub fn routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/discovery/routing_table", get(routing_table))
        .route("/discovery/bootstrap_stats", get(bootstrap_stats))
}

/// Returns the Kademlia routing table.
async fn routing_table(
    State(state): State<AppState>,
) -> AppResult<Json<RpcDiscoveryRoutingTableResponse>> {
    jsonify_rpc!(state, RpcRequest::DiscoveryRoutingTable)
}

/// Returns bootstrap statistics.
async fn bootstrap_stats(
    State(state): State<AppState>,
) -> AppResult<Json<RpcDiscoveryBoostrapStatsResponse>> {
    jsonify_rpc!(state, RpcRequest::DiscoveryBoostrapStats)
}
