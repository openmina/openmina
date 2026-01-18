//! Status and health check endpoints.
//!
//! - `GET /build_env` - Build environment information
//! - `GET /status` - Node status
//! - `GET /healthz` - Kubernetes health check
//! - `GET /readyz` - Kubernetes readiness check
//! - `POST /make_heartbeat` - Trigger a heartbeat

use axum::{extract::State, Json};
use utoipa_axum::{router::OpenApiRouter, routes};

use mina_node::rpc::{
    RpcHealthCheckResponse, RpcHeartbeatGetResponse, RpcReadinessCheckResponse, RpcRequest,
    RpcStatusGetResponse,
};

use crate::http_server::{AppError, AppResult, AppState};

/// Returns status routes as an OpenApiRouter.
pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(build_env))
        .routes(routes!(status))
        .routes(routes!(healthz))
        .routes(routes!(readyz))
        .routes(routes!(make_heartbeat))
}

/// Build environment information
#[utoipa::path(
    get,
    path = "/build_env",
    tag = "status",
    responses(
        (status = 200, description = "Build environment information")
    )
)]
async fn build_env() -> Json<mina_node::BuildEnv> {
    Json(mina_node::BuildEnv::get())
}

/// Current node status
#[utoipa::path(
    get,
    path = "/status",
    tag = "status",
    responses(
        (status = 200, description = "Current node status")
    )
)]
async fn status(State(state): State<AppState>) -> AppResult<Json<RpcStatusGetResponse>> {
    jsonify_rpc!(state, RpcRequest::StatusGet)
}

/// Liveness probe
#[utoipa::path(
    get,
    path = "/healthz",
    tags = ["status", "kubernetes"],
    responses(
        (status = 200, description = "Node is healthy"),
        (status = 503, description = "Node is unhealthy")
    )
)]
async fn healthz(State(state): State<AppState>) -> AppResult<&'static str> {
    let reply: RpcHealthCheckResponse = rpc_request!(state, RpcRequest::HealthCheck)?;
    reply.map(|()| "").map_err(AppError::ServiceUnavailable)
}

/// Readiness probe
#[utoipa::path(
    get,
    path = "/readyz",
    tags = ["status", "kubernetes"],
    responses(
        (status = 200, description = "Node is ready to accept traffic"),
        (status = 503, description = "Node is not ready")
    )
)]
async fn readyz(State(state): State<AppState>) -> AppResult<&'static str> {
    let reply: RpcReadinessCheckResponse = rpc_request!(state, RpcRequest::ReadinessCheck)?;
    reply.map(|()| "").map_err(AppError::ServiceUnavailable)
}

/// Trigger heartbeat
#[utoipa::path(
    post,
    path = "/make_heartbeat",
    tag = "status",
    responses(
        (status = 200, description = "Heartbeat triggered successfully")
    )
)]
async fn make_heartbeat(State(state): State<AppState>) -> AppResult<Json<RpcHeartbeatGetResponse>> {
    jsonify_rpc!(state, RpcRequest::HeartbeatGet)
}
