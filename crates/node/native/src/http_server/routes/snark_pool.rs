//! Snark pool endpoints.
//!
//! - `GET /snark-pool/jobs` - Get all snark pool jobs
//! - `GET /snark-pool/job/{id}` - Get specific snark pool job

use axum::{
    extract::{Path, State},
    Json,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use mina_node::{
    core::snark::SnarkJobId,
    rpc::{RpcRequest, RpcSnarkPoolGetResponse, RpcSnarkPoolJobGetResponse},
};

use crate::http_server::{AppResult, AppState};

/// Snark pool routes
pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(jobs))
        .routes(routes!(job))
}

/// All snark pool jobs
#[utoipa::path(
    get,
    path = "/snark-pool/jobs",
    tag = "snark-pool",
    responses(
        (status = 200, description = "Snark pool jobs")
    )
)]
async fn jobs(State(state): State<AppState>) -> AppResult<Json<RpcSnarkPoolGetResponse>> {
    jsonify_rpc!(state, RpcRequest::SnarkPoolGet)
}

/// Specific snark pool job
#[utoipa::path(
    get,
    path = "/snark-pool/job/{job_id}",
    tag = "snark-pool",
    params(
        ("job_id" = String, Path, description = "Snark job ID")
    ),
    responses(
        (status = 200, description = "Snark pool job")
    )
)]
async fn job(
    State(state): State<AppState>,
    Path(job_id): Path<SnarkJobId>,
) -> AppResult<Json<RpcSnarkPoolJobGetResponse>> {
    jsonify_rpc!(state, RpcRequest::SnarkPoolJobGet { job_id })
}
