//! Snark pool endpoints.
//!
//! - `GET /snark-pool/jobs` - Get all snark pool jobs
//! - `GET /snark-pool/job/{id}` - Get specific snark pool job

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};

use mina_node::{
    core::snark::SnarkJobId,
    rpc::{RpcRequest, RpcSnarkPoolGetResponse, RpcSnarkPoolJobGetResponse},
};

use crate::http_server::{AppResult, AppState};

/// Registers snark pool routes on the router.
pub fn routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/snark-pool/jobs", get(jobs))
        .route("/snark-pool/job/{job_id}", get(job))
}

/// Returns all snark pool jobs.
async fn jobs(State(state): State<AppState>) -> AppResult<Json<RpcSnarkPoolGetResponse>> {
    jsonify_rpc!(state, RpcRequest::SnarkPoolGet)
}

/// Returns a specific snark pool job.
async fn job(
    State(state): State<AppState>,
    Path(job_id): Path<SnarkJobId>,
) -> AppResult<Json<RpcSnarkPoolJobGetResponse>> {
    jsonify_rpc!(state, RpcRequest::SnarkPoolJobGet { job_id })
}
