//! Snarker endpoints.
//!
//! - `POST /snarker/job/commit` - Commit to a snark job
//! - `GET /snarker/job/spec` - Get snark job specification
//! - `GET /snarker/workers` - Get snarker workers
//! - `GET /snarker/config` - Get snarker configuration

use std::str::FromStr;

use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, HeaderMap, Response, StatusCode},
    routing::{get, post},
    Json, Router,
};
use mina_p2p_messages::binprot::BinProtWrite;
use serde::Deserialize;

use mina_node::{
    core::snark::SnarkJobId,
    rpc::{
        RpcRequest, RpcSnarkerConfigGetResponse, RpcSnarkerJobCommitResponse,
        RpcSnarkerJobSpecResponse, RpcSnarkerWorkersResponse,
    },
};

use crate::http_server::{AppError, AppResult, AppState};

/// Registers snarker routes on the router.
pub fn routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/snarker/job/commit", post(job_commit))
        .route("/snarker/job/spec", get(job_spec))
        .route("/snarker/workers", get(workers))
        .route("/snarker/config", get(config))
}

/// Commits to a snark job.
///
/// TODO(binier): make endpoint only accessible locally.
///
/// TODO(axum-migration): Error returns bare JSON string for warp compatibility.
/// Migrate to structured error (e.g., `{"error": "...", "details": {...}}`).
async fn job_commit(
    State(state): State<AppState>,
    body: String,
) -> AppResult<(StatusCode, Json<RpcSnarkerJobCommitResponse>)> {
    let job_id = SnarkJobId::from_str(&body)
        .map_err(|_| AppError::Json(StatusCode::BAD_REQUEST, serde_json::json!("invalid_input")))?;

    let resp: RpcSnarkerJobCommitResponse =
        rpc_request!(state, RpcRequest::SnarkerJobCommit { job_id })?;

    let status = match &resp {
        RpcSnarkerJobCommitResponse::Ok => StatusCode::CREATED,
        _ => StatusCode::BAD_REQUEST,
    };

    Ok((status, Json(resp)))
}

#[derive(Deserialize)]
struct JobSpecQuery {
    id: SnarkJobId,
}

/// Returns snark job specification.
///
/// Supports both JSON and binary (binprot) output based on Accept header.
async fn job_spec(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(JobSpecQuery { id: job_id }): Query<JobSpecQuery>,
) -> AppResult<Response<Body>> {
    let resp: RpcSnarkerJobSpecResponse =
        rpc_request!(state, RpcRequest::SnarkerJobSpec { job_id })?;

    let accept = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    match resp {
        RpcSnarkerJobSpecResponse::Ok(spec) if accept == "application/octet-stream" => {
            // Binary output (binprot format with length prefix)
            let mut vec = Vec::new();
            spec.binprot_write(&mut vec)
                .map_err(|e| AppError::Internal(format!("binprot serialization failed: {e}")))?;

            let mut result = Vec::with_capacity(vec.len() + std::mem::size_of::<u64>());
            result.extend((vec.len() as u64).to_le_bytes());
            result.extend(vec);

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/octet-stream")
                .body(Body::from(result))
                .map_err(|e| AppError::Internal(e.to_string()))
        }
        RpcSnarkerJobSpecResponse::Ok(spec) => {
            // JSON output
            let body = serde_json::to_vec(&spec)
                .map_err(|e| AppError::Internal(format!("JSON serialization failed: {e}")))?;

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .map_err(|e| AppError::Internal(e.to_string()))
        }
        _ => {
            // Error response
            let body = serde_json::to_vec(&"error")
                .map_err(|e| AppError::Internal(format!("JSON serialization failed: {e}")))?;

            Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .map_err(|e| AppError::Internal(e.to_string()))
        }
    }
}

/// Returns snarker workers.
async fn workers(State(state): State<AppState>) -> AppResult<Json<RpcSnarkerWorkersResponse>> {
    jsonify_rpc!(state, RpcRequest::SnarkerWorkers)
}

/// Returns snarker configuration.
async fn config(State(state): State<AppState>) -> AppResult<Json<RpcSnarkerConfigGetResponse>> {
    jsonify_rpc!(state, RpcRequest::SnarkerConfig)
}
