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
    Json,
};
use mina_p2p_messages::binprot::BinProtWrite;
use serde::Deserialize;
use utoipa_axum::{router::OpenApiRouter, routes};

use mina_node::{
    core::snark::SnarkJobId,
    rpc::{
        RpcRequest, RpcSnarkerConfigGetResponse, RpcSnarkerJobCommitResponse,
        RpcSnarkerJobSpecResponse, RpcSnarkerWorkersResponse,
    },
};

use crate::http_server::{AppError, AppResult, AppState};

/// Snarker routes
pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(job_commit))
        .routes(routes!(job_spec))
        .routes(routes!(workers))
        .routes(routes!(config))
}

/// Commit to a snark job
#[utoipa::path(
    post,
    path = "/snarker/job/commit",
    tag = "snarker",
    responses(
        (status = 201, description = "Job committed"),
        (status = 400, description = "Invalid input or job error")
    )
)]
async fn job_commit(
    State(state): State<AppState>,
    body: String,
) -> AppResult<(StatusCode, Json<RpcSnarkerJobCommitResponse>)> {
    // TODO(binier): make endpoint only accessible locally.
    // TODO(axum-migration): Error returns bare JSON string for warp compatibility.
    // Migrate to structured error (e.g., `{"error": "...", "details": {...}}`).
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

/// Snark job specification
///
/// Supports JSON and binary (binprot) output based on Accept header.
#[utoipa::path(
    get,
    path = "/snarker/job/spec",
    tag = "snarker",
    params(
        ("id" = String, Query, description = "Snark job ID")
    ),
    responses(
        (status = 200, description = "JSON job spec", content_type = "application/json"),
        (status = 200, description = "Binprot job spec", content_type = "application/octet-stream"),
        (status = 400, description = "Job not found")
    )
)]
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

/// Snarker workers
#[utoipa::path(
    get,
    path = "/snarker/workers",
    tag = "snarker",
    responses(
        (status = 200, description = "Snarker workers")
    )
)]
async fn workers(State(state): State<AppState>) -> AppResult<Json<RpcSnarkerWorkersResponse>> {
    jsonify_rpc!(state, RpcRequest::SnarkerWorkers)
}

/// Snarker configuration
#[utoipa::path(
    get,
    path = "/snarker/config",
    tag = "snarker",
    responses(
        (status = 200, description = "Snarker configuration")
    )
)]
async fn config(State(state): State<AppState>) -> AppResult<Json<RpcSnarkerConfigGetResponse>> {
    jsonify_rpc!(state, RpcRequest::SnarkerConfig)
}
