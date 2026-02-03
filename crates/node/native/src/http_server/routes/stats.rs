//! Statistics endpoints.
//!
//! - `GET /stats/actions` - Get action statistics (optional `id` query param)
//! - `GET /stats/sync` - Get sync statistics (optional `limit` query param)
//! - `GET /stats/block_producer` - Get block producer statistics

use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use utoipa_axum::{router::OpenApiRouter, routes};

use mina_node::rpc::{
    ActionStatsQuery, RpcActionStatsGetResponse, RpcBlockProducerStatsGetResponse, RpcRequest,
    RpcSyncStatsGetResponse, SyncStatsQuery,
};

use crate::http_server::{AppError, AppResult, AppState, JsonErrorResponse};

/// Stats routes
pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(actions))
        .routes(routes!(sync))
        .routes(routes!(block_producer))
}

#[derive(Deserialize, Default)]
struct ActionQueryParams {
    /// Optional filter: "latest" for latest block, or a block ID (u64).
    id: Option<String>,
}

/// Action statistics
#[utoipa::path(
    get,
    path = "/stats/actions",
    tag = "stats",
    params(
        ("id" = Option<String>, Query, description = "\"latest\" for latest block, or numeric block ID")
    ),
    responses(
        // inline: type alias for Option<T> would register as "Option"
        (status = 200, description = "Action statistics", body = inline(RpcActionStatsGetResponse)),
        (status = 400, description = "Invalid id parameter", body = JsonErrorResponse,
            example = json!({"error": "'id' must be an u64 integer: invalid digit found in string, instead passed: foo"}))
    )
)]
async fn actions(
    State(state): State<AppState>,
    Query(params): Query<ActionQueryParams>,
) -> AppResult<Json<RpcActionStatsGetResponse>> {
    let query = match params.id.as_deref() {
        None => ActionStatsQuery::SinceStart,
        Some("latest") => ActionStatsQuery::ForLatestBlock,
        Some(id) => {
            let id: u64 = id.parse().map_err(|err| {
                AppError::BadRequest(format!(
                    "'id' must be an u64 integer: {err}, instead passed: {id}"
                ))
            })?;
            ActionStatsQuery::ForBlockWithId(id)
        }
    };

    jsonify_rpc!(state, RpcRequest::ActionStatsGet(query))
}

#[derive(Deserialize, Default)]
struct SyncQueryParams {
    /// Optional limit on the number of sync snapshots to return.
    limit: Option<usize>,
}

/// Sync statistics
#[utoipa::path(
    get,
    path = "/stats/sync",
    tag = "stats",
    params(
        ("limit" = Option<usize>, Query, description = "Max number of sync snapshots to return")
    ),
    responses(
        // inline: type alias for Option<Vec<T>> would register as "Option"
        (status = 200, description = "Sync statistics", body = inline(RpcSyncStatsGetResponse))
    )
)]
async fn sync(
    State(state): State<AppState>,
    Query(SyncQueryParams { limit }): Query<SyncQueryParams>,
) -> AppResult<Json<RpcSyncStatsGetResponse>> {
    jsonify_rpc!(state, RpcRequest::SyncStatsGet(SyncStatsQuery { limit }))
}

/// Block producer statistics
#[utoipa::path(
    get,
    path = "/stats/block_producer",
    tag = "stats",
    responses(
        // inline: type alias for Option<T> would register as "Option"
        (status = 200, description = "Block producer statistics", body = inline(RpcBlockProducerStatsGetResponse))
    )
)]
async fn block_producer(
    State(state): State<AppState>,
) -> AppResult<Json<RpcBlockProducerStatsGetResponse>> {
    jsonify_rpc!(state, RpcRequest::BlockProducerStatsGet)
}
