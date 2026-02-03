//! Discovery endpoints.
//!
//! - `GET /discovery/routing_table` - Get Kademlia routing table
//! - `GET /discovery/bootstrap_stats` - Get bootstrap statistics

use axum::{extract::State, Json};
use utoipa_axum::{router::OpenApiRouter, routes};

use mina_node::rpc::{
    RpcDiscoveryBoostrapStatsResponse, RpcDiscoveryRoutingTableResponse, RpcRequest,
};

use crate::http_server::{AppResult, AppState};

/// Discovery routes
pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(routing_table))
        .routes(routes!(bootstrap_stats))
}

/// Kademlia routing table
#[utoipa::path(
    get,
    path = "/discovery/routing_table",
    tag = "discovery",
    responses(
        (status = 200, description = "Routing table")
    )
)]
async fn routing_table(
    State(state): State<AppState>,
) -> AppResult<Json<RpcDiscoveryRoutingTableResponse>> {
    jsonify_rpc!(state, RpcRequest::DiscoveryRoutingTable)
}

/// Bootstrap statistics
#[utoipa::path(
    get,
    path = "/discovery/bootstrap_stats",
    tag = "discovery",
    responses(
        (status = 200, description = "Bootstrap statistics")
    )
)]
async fn bootstrap_stats(
    State(state): State<AppState>,
) -> AppResult<Json<RpcDiscoveryBoostrapStatsResponse>> {
    jsonify_rpc!(state, RpcRequest::DiscoveryBoostrapStats)
}
