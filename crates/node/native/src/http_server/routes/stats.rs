//! Statistics endpoints.
//!
//! - `GET /stats/actions` - Get action statistics (optional `id` query param)
//! - `GET /stats/sync` - Get sync statistics (optional `limit` query param)
//! - `GET /stats/block_producer` - Get block producer statistics

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use mina_node::rpc::{
    ActionStatsQuery, RpcActionStatsGetResponse, RpcBlockProducerStatsGetResponse, RpcRequest,
    RpcSyncStatsGetResponse, SyncStatsQuery,
};

use crate::http_server::{AppError, AppResult, AppState};

/// Registers stats routes on the router.
pub fn routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/stats/actions", get(actions))
        .route("/stats/sync", get(sync))
        .route("/stats/block_producer", get(block_producer))
}

#[derive(Deserialize, Default)]
struct ActionQueryParams {
    /// Optional filter: "latest" for latest block, or a block ID (u64).
    id: Option<String>,
}

/// Returns action statistics.
///
/// Query params:
/// - `id` (optional): "latest" for latest block stats, or a numeric block ID.
///   If omitted, returns stats since node start.
///
/// TODO(axum-migration): Error returns bare JSON string for warp compatibility.
/// Migrate to structured error (e.g., `{"error": "...", "details": {...}}`).
async fn actions(
    State(state): State<AppState>,
    Query(params): Query<ActionQueryParams>,
) -> AppResult<Json<RpcActionStatsGetResponse>> {
    let query = match params.id.as_deref() {
        None => ActionStatsQuery::SinceStart,
        Some("latest") => ActionStatsQuery::ForLatestBlock,
        Some(id) => {
            let id: u64 = id.parse().map_err(|err| {
                AppError::Json(
                    StatusCode::BAD_REQUEST,
                    serde_json::json!(format!(
                        "'id' must be an u64 integer: {err}, instead passed: {id}"
                    )),
                )
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

/// Returns sync statistics.
async fn sync(
    State(state): State<AppState>,
    Query(SyncQueryParams { limit }): Query<SyncQueryParams>,
) -> AppResult<Json<RpcSyncStatsGetResponse>> {
    jsonify_rpc!(state, RpcRequest::SyncStatsGet(SyncStatsQuery { limit }))
}

/// Returns block producer statistics.
async fn block_producer(
    State(state): State<AppState>,
) -> AppResult<Json<RpcBlockProducerStatsGetResponse>> {
    jsonify_rpc!(state, RpcRequest::BlockProducerStatsGet)
}
