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
async fn status(State(state): State<AppState>) -> Json<RpcStatusGetResponse> {
    let result = state
        .rpc_sender()
        .oneshot_request(RpcRequest::StatusGet)
        .await
        .flatten();
    Json(result)
}

/// Kubernetes liveness probe endpoint.
///
/// Returns empty body with 200 OK on success, or error string with 503 on failure.
async fn healthz(State(state): State<AppState>) -> AppResult<&'static str> {
    state
        .rpc_sender()
        .oneshot_request(RpcRequest::HealthCheck)
        .await
        .ok_or(AppError::ChannelDropped)
        .and_then(|reply: RpcHealthCheckResponse| match reply {
            Ok(()) => Ok(""),
            Err(err) => Err(AppError::ServiceUnavailable(err)),
        })
}

/// Kubernetes readiness probe endpoint.
///
/// Returns empty body with 200 OK on success, or error string with 503 on failure.
async fn readyz(State(state): State<AppState>) -> AppResult<&'static str> {
    state
        .rpc_sender()
        .oneshot_request(RpcRequest::ReadinessCheck)
        .await
        .ok_or(AppError::ChannelDropped)
        .and_then(|reply: RpcReadinessCheckResponse| match reply {
            Ok(()) => Ok(""),
            Err(err) => Err(AppError::ServiceUnavailable(err)),
        })
}

/// Triggers a heartbeat.
async fn make_heartbeat(State(state): State<AppState>) -> Json<RpcHeartbeatGetResponse> {
    let result = state
        .rpc_sender()
        .oneshot_request(RpcRequest::HeartbeatGet)
        .await
        .flatten();
    Json(result)
}
