//! State inspection endpoints.
//!
//! - `GET /state` - Get node state with optional JSONPath filter
//! - `POST /state` - Get node state with JSONPath filter in body
//! - `GET /state/peers` - Get connected peers
//! - `GET /state/message-progress` - Get message progress
//!
//! The JSONPath filter implementation is in `impl mina_node::rpc_effectful::RpcService for NodeService`.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use mina_node::rpc::{RpcMessageProgressResponse, RpcPeersGetResponse, RpcRequest};
use mina_node_common::rpc::RpcStateGetResponse;

use crate::http_server::{AppError, AppResult, AppState};

/// Registers state routes on the router.
pub fn routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/state", get(state_get).post(state_post))
        .route("/state/peers", get(peers))
        .route("/state/message-progress", get(message_progress))
}

#[derive(Deserialize, Default)]
struct StateQueryParams {
    /// Optional JSONPath filter expression.
    filter: Option<String>,
}

/// Returns node state with optional JSONPath filter (query param).
async fn state_get(
    State(state): State<AppState>,
    Query(params): Query<StateQueryParams>,
) -> AppResult<Json<serde_json::Value>> {
    state_handler(state, params.filter).await
}

/// Returns node state with optional JSONPath filter (JSON body).
async fn state_post(
    State(state): State<AppState>,
    Json(params): Json<StateQueryParams>,
) -> AppResult<Json<serde_json::Value>> {
    state_handler(state, params.filter).await
}

/// Shared handler for state requests. Returns `serde_json::Value` because the
/// JSONPath filter produces a dynamic response shape.
async fn state_handler(
    state: AppState,
    filter: Option<String>,
) -> AppResult<Json<serde_json::Value>> {
    let result: Option<RpcStateGetResponse> = state
        .rpc_sender()
        .oneshot_request(RpcRequest::StateGet(filter))
        .await;

    match result {
        None => Err(AppError::ChannelDropped),
        Some(Ok(value)) => Ok(Json(value)),
        Some(Err(err)) => Err(AppError::Json(
            StatusCode::BAD_REQUEST,
            serde_json::to_value(err).unwrap_or_default(),
        )),
    }
}

/// Returns connected peers.
async fn peers(State(state): State<AppState>) -> Json<Option<RpcPeersGetResponse>> {
    let result = state
        .rpc_sender()
        .oneshot_request(RpcRequest::PeersGet)
        .await;
    Json(result)
}

/// Returns message progress information.
async fn message_progress(
    State(state): State<AppState>,
) -> Json<Option<RpcMessageProgressResponse>> {
    let result = state
        .rpc_sender()
        .oneshot_request(RpcRequest::MessageProgressGet)
        .await;
    Json(result)
}
