//! Status and health check endpoints.
//!
//! - `GET /build_env` - Build environment information
//! - `GET /status` - Node status
//! - `GET /healthz` - Kubernetes health check
//! - `GET /readyz` - Kubernetes readiness check

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};

use mina_node::rpc::{
    RpcHealthCheckResponse, RpcHeartbeatGetResponse, RpcReadinessCheckResponse, RpcRequest,
    RpcStatusGetResponse,
};

use crate::http_server::{AppError, AppResult, AppState};

/// Registers status routes on the router.
pub fn routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/build_env", get(build_env))
        .route("/status", get(status))
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/make_heartbeat", post(make_heartbeat))
}

/// Returns build environment information.
async fn build_env() -> Json<mina_node::BuildEnv> {
    Json(mina_node::BuildEnv::get())
}

/// Returns the current node status.
async fn status(State(state): State<AppState>) -> AppResult<Json<RpcStatusGetResponse>> {
    jsonify_rpc!(state, RpcRequest::StatusGet)
}

/// Kubernetes liveness probe endpoint.
///
/// Returns empty body with 200 OK on success, or error string with 503 on failure.
async fn healthz(State(state): State<AppState>) -> AppResult<&'static str> {
    let reply: RpcHealthCheckResponse = rpc_request!(state, RpcRequest::HealthCheck)?;
    reply.map(|()| "").map_err(AppError::ServiceUnavailable)
}

/// Kubernetes readiness probe endpoint.
///
/// Returns empty body with 200 OK on success, or error string with 503 on failure.
async fn readyz(State(state): State<AppState>) -> AppResult<&'static str> {
    let reply: RpcReadinessCheckResponse = rpc_request!(state, RpcRequest::ReadinessCheck)?;
    reply.map(|()| "").map_err(AppError::ServiceUnavailable)
}

/// Triggers a heartbeat.
async fn make_heartbeat(State(state): State<AppState>) -> AppResult<Json<RpcHeartbeatGetResponse>> {
    jsonify_rpc!(state, RpcRequest::HeartbeatGet)
}
