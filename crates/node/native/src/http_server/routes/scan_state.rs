//! Scan state endpoints.
//!
//! - `GET /scan-state/summary` - Get scan state summary for best tip
//! - `GET /scan-state/summary/{block}` - Get scan state summary for specific block

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use mina_node::rpc::{
    RpcRequest, RpcScanStateSummary, RpcScanStateSummaryGetQuery, RpcScanStateSummaryGetResponse,
};

use crate::http_server::{AppError, AppResult, AppState};

/// Scan state routes
pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(summary))
        .routes(routes!(summary_for_block))
}

/// Scan state summary for best tip
#[utoipa::path(
    get,
    path = "/scan-state/summary",
    tag = "scan-state",
    responses(
        (status = 200, description = "Scan state summary"),
        (status = 500, description = "Target block not found")
    )
)]
async fn summary(State(state): State<AppState>) -> AppResult<Json<RpcScanStateSummary>> {
    // TODO(axum-migration): "target block not found" should arguably be 404, not 500.
    // Keeping 500 for warp compatibility. Also returns bare JSON string for errors
    // to match warp; should migrate to `{"error": "..."}` format.
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

/// Scan state summary for specific block
#[utoipa::path(
    get,
    path = "/scan-state/summary/{block}",
    tag = "scan-state",
    params(
        ("block" = String, Path, description = "Block height or hash")
    ),
    responses(
        (status = 200, description = "Scan state summary"),
        (status = 400, description = "Invalid block identifier"),
        (status = 500, description = "Target block not found")
    )
)]
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
