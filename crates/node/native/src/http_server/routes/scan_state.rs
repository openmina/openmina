//! Scan state endpoints.
//!
//! - `GET /scan-state/summary` - Get scan state summary for best tip
//! - `GET /scan-state/summary/{block}` - Get scan state summary for specific block

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};

use mina_node::rpc::{
    RpcRequest, RpcScanStateSummary, RpcScanStateSummaryGetQuery, RpcScanStateSummaryGetResponse,
};

use crate::http_server::{AppError, AppResult, AppState};

/// Registers scan state routes on the router.
pub fn routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/scan-state/summary", get(summary))
        .route("/scan-state/summary/{block}", get(summary_for_block))
}

/// Returns scan state summary for best tip.
///
/// TODO(axum-migration): "target block not found" should arguably be 404, not 500.
/// Keeping 500 for warp compatibility. Also returns bare JSON string for errors
/// to match warp; should migrate to `{"error": "..."}` format.
async fn summary(State(state): State<AppState>) -> AppResult<Json<RpcScanStateSummary>> {
    let result: Option<RpcScanStateSummaryGetResponse> = state
        .rpc_sender()
        .oneshot_request(RpcRequest::ScanStateSummaryGet(
            RpcScanStateSummaryGetQuery::ForBestTip,
        ))
        .await;

    match result {
        None => Err(AppError::ChannelDropped),
        Some(Ok(data)) => Ok(Json(data)),
        Some(Err(err)) => Err(AppError::Json(
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!(err),
        )),
    }
}

/// Returns scan state summary for a specific block (by height or hash).
async fn summary_for_block(
    State(state): State<AppState>,
    Path(block): Path<String>,
) -> AppResult<Json<RpcScanStateSummary>> {
    // Try parsing as height first, then as hash
    let query = if let Ok(height) = block.parse::<u32>() {
        RpcScanStateSummaryGetQuery::ForBlockWithHeight(height)
    } else {
        match block.parse() {
            Ok(hash) => RpcScanStateSummaryGetQuery::ForBlockWithHash(hash),
            Err(_) => {
                return Err(AppError::Json(
                    StatusCode::BAD_REQUEST,
                    serde_json::json!("invalid arg! Expected block hash or height"),
                ))
            }
        }
    };

    let result: Option<RpcScanStateSummaryGetResponse> = state
        .rpc_sender()
        .oneshot_request(RpcRequest::ScanStateSummaryGet(query))
        .await;

    match result {
        None => Err(AppError::ChannelDropped),
        Some(Ok(data)) => Ok(Json(data)),
        Some(Err(err)) => Err(AppError::Json(
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!(err),
        )),
    }
}
