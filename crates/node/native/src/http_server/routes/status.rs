//! Status and health check endpoints.
//!
//! - `GET /build_env` - Build environment information
//! - `GET /status` - Node status
//! - `GET /healthz` - Kubernetes health check
//! - `GET /readyz` - Kubernetes readiness check
//! - `POST /make_heartbeat` - Trigger a heartbeat

use axum::{extract::State, Json};
use utoipa_axum::{router::OpenApiRouter, routes};

use mina_node::{
    rpc::{
        RpcHealthCheckResponse, RpcHeartbeatGetResponse, RpcNodeStatus, RpcReadinessCheckResponse,
        RpcRequest, RpcStatusGetResponse,
    },
    BuildEnv,
};

use crate::http_server::{AppError, AppResult, AppState, JsonErrorResponse};

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
        (status = 200, body = BuildEnv)
    )
)]
async fn build_env() -> Json<BuildEnv> {
    Json(mina_node::BuildEnv::get())
}

/// Current node status
#[utoipa::path(
    get,
    path = "/status",
    tag = "status",
    responses(
        (status = 200, description = "Current node status", body = RpcNodeStatus)
    )
)]
async fn status(State(state): State<AppState>) -> AppResult<Json<RpcNodeStatus>> {
    let reply: RpcStatusGetResponse = rpc_request!(state, RpcRequest::StatusGet)?;
    reply.map(Json).ok_or(AppError::Internal(
        "StatusGet should always return Some(...)".into(),
    ))
}

/// Liveness probe
#[utoipa::path(
    get,
    path = "/healthz",
    tags = ["status", "kubernetes"],
    responses(
        (status = 200, description = "Node is healthy"),
        (status = 503, description = "Node is unhealthy", body = JsonErrorResponse,
            examples(
                ("NoReadyPeers" = (value = json!({"error": "no ready peers"}))),
            ))
    )
)]
async fn healthz(State(state): State<AppState>) -> AppResult<()> {
    let reply: RpcHealthCheckResponse = rpc_request!(state, RpcRequest::HealthCheck)?;
    reply.map_err(AppError::ServiceUnavailable)
}

/// Readiness probe
#[utoipa::path(
    get,
    path = "/readyz",
    tags = ["status", "kubernetes"],
    responses(
        (status = 200, description = "Node is ready to accept traffic"),
        (status = 503, description = "Node is not ready", body = JsonErrorResponse,
            examples(
                ("NotSynced" = (value = json!({"error": "not synced"}))),
                ("Desynced" = (value = json!({"error": "Synced 2000s ago, which is more than the threshold 1800s"}))),
            )),
    )
)]
async fn readyz(State(state): State<AppState>) -> AppResult<()> {
    let reply: RpcReadinessCheckResponse = rpc_request!(state, RpcRequest::ReadinessCheck)?;
    reply.map_err(AppError::ServiceUnavailable)
}

/// Trigger heartbeat
#[utoipa::path(
    post,
    path = "/make_heartbeat",
    tag = "status",
    responses(
        // inline: type alias for Option<T> would register as "Option"
        (status = 200, description = "Heartbeat triggered successfully", body = inline(RpcHeartbeatGetResponse))
    )
)]
async fn make_heartbeat(State(state): State<AppState>) -> AppResult<Json<RpcHeartbeatGetResponse>> {
    jsonify_rpc!(state, RpcRequest::HeartbeatGet)
}
