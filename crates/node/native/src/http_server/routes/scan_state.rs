//! Scan state endpoints.
//!
//! - `GET /scan-state/summary` - Get scan state summary for best tip
//! - `GET /scan-state/summary/{block}` - Get scan state summary for specific block

use std::str::FromStr;

use axum::{
    extract::{Path, State},
    Json,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use mina_node::rpc::{
    RpcRequest, RpcScanStateSummary, RpcScanStateSummaryGetQuery, RpcScanStateSummaryGetResponse,
};
use mina_p2p_messages::v2::StateHash;

use crate::http_server::{AppError, AppResult, AppState, JsonErrorResponse};

/// Block identifier for scan state queries.
///
/// Can be "latest" for best tip, a block height (u32), or a block hash.
#[derive(Debug, Clone, utoipa::ToSchema)]
#[allow(unused, reason = "schema type for block identifier query param")]
enum BlockIdentifier {
    /// Use best tip block
    Latest,
    /// Block height (e.g., 490467)
    Height(u32),
    /// Block hash (e.g., 3NLrbJrSvDVEqnMMEeWvk1TiCmcDpnUiHZqdEKVEZcqieKu1TBkS)
    Hash(StateHash),
}

impl FromStr for BlockIdentifier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "latest" {
            Ok(BlockIdentifier::Latest)
        } else if let Ok(height) = s.parse::<u32>() {
            Ok(BlockIdentifier::Height(height))
        } else {
            s.parse::<StateHash>()
                .map(BlockIdentifier::Hash)
                .map_err(|_| "invalid arg! Expected 'latest', block height, or block hash".into())
        }
    }
}

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
        (status = 200, description = "Scan state summary", body = RpcScanStateSummary),
        (status = 500, description = "Target block not found", body = JsonErrorResponse,
            example = json!({"error": "target block not found"}))
    )
)]
async fn summary(State(state): State<AppState>) -> AppResult<Json<RpcScanStateSummary>> {
    // TODO: "target block not found" should arguably be 404, not 500
    let result: Option<RpcScanStateSummaryGetResponse> = state
        .rpc_sender()
        .oneshot_request(RpcRequest::ScanStateSummaryGet(
            RpcScanStateSummaryGetQuery::ForBestTip,
        ))
        .await;

    match result {
        None => Err(AppError::ChannelDropped),
        Some(Ok(data)) => Ok(Json(data)),
        Some(Err(err)) => Err(AppError::Internal(err)),
    }
}

/// Scan state summary for specific block
#[utoipa::path(
    get,
    path = "/scan-state/summary/{block}",
    tag = "scan-state",
    params(
        ("block" = BlockIdentifier, Path, description = "\"latest\" for best tip, block height, or block hash")
    ),
    responses(
        (status = 200, description = "Scan state summary", body = RpcScanStateSummary),
        (status = 400, description = "Invalid block identifier", body = JsonErrorResponse,
            example = json!({"error": "invalid arg! Expected block hash or height"})),
        (status = 500, description = "Target block not found", body = JsonErrorResponse,
            example = json!({"error": "target block not found"}))
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
                return Err(AppError::BadRequest(
                    "invalid arg! Expected block hash or height".to_string(),
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
        Some(Err(err)) => Err(AppError::Internal(err)),
    }
}
