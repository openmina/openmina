//! Shared types for the axum HTTP server.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use tower_http::cors::{Any, CorsLayer};

use mina_node_common::rpc::RpcSender;

/// Application state shared across all axum handlers.
#[derive(Clone)]
pub struct AppState {
    rpc_sender: RpcSender,
}

impl AppState {
    /// Creates a new application state with the given RPC sender.
    pub fn new(rpc_sender: RpcSender) -> Self {
        Self { rpc_sender }
    }

    /// Returns a reference to the RPC sender.
    pub fn rpc_sender(&self) -> &RpcSender {
        &self.rpc_sender
    }
}

/// Result type alias for HTTP handlers.
pub type AppResult<T> = Result<T, AppError>;

/// HTTP API error type that converts to appropriate HTTP responses.
#[derive(Debug, thiserror::Error, utoipa::ToSchema)]
pub enum AppError {
    /// The RPC channel was dropped before a response was received.
    #[error("response channel dropped, see error log for details")]
    ChannelDropped,

    /// An internal server error occurred.
    #[error("{0}")]
    Internal(String),

    /// A bad request was made by the client.
    #[error("{0}")]
    BadRequest(String),

    /// The service is temporarily unavailable (e.g., not ready).
    #[error("{0}")]
    ServiceUnavailable(String),

    /// JSON error response with custom status code.
    #[error("{0}: {1}")]
    Json(StatusCode, serde_json::Value),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::ChannelDropped => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "response channel dropped, see error log for details".to_owned(),
            ),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            AppError::Json(status, value) => return (status, Json(value)).into_response(),
        };
        (status, Json(serde_json::json!({"error": message}))).into_response()
    }
}

/// Creates the CORS layer with permissive settings matching the warp server.
pub fn cors_layer() -> CorsLayer {
    use axum::http::{header::*, Method};

    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            USER_AGENT,
            REFERER,
            ORIGIN,
            ACCESS_CONTROL_REQUEST_METHOD,
            ACCESS_CONTROL_REQUEST_HEADERS,
            CONTENT_TYPE,
            HeaderName::from_static("sec-fetch-mode"),
        ])
}
