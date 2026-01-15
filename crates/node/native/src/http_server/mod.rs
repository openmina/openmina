//! Axum-based HTTP server for the Mina node RPC API.
//!
//! This module provides REST endpoints for node status, state inspection,
//! snark pool management, and transaction handling.

mod routes;
mod types;

pub use types::{AppError, AppResult, AppState};

use axum::Router;
use mina_node_common::rpc::RpcSender;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use types::cors_layer;

/// Runs the HTTP server on the specified port.
///
/// Returns an error if binding to the port fails or the server encounters an I/O error.
pub async fn run(port: u16, rpc_sender: RpcSender) -> std::io::Result<()> {
    let state = AppState::new(rpc_sender);

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &axum::http::Request<_>| {
            tracing::debug_span!(
                "http_request",
                method = %request.method(),
                uri = %request.uri(),
            )
        })
        .on_response(
            |response: &axum::http::Response<_>, latency, _span: &tracing::Span| {
                let status = response.status();
                if status.is_server_error() || status.is_client_error() {
                    tracing::error!(status = %status, latency = ?latency, "request failed");
                } else {
                    tracing::info!(status = %status, latency = ?latency, "request completed");
                }
            },
        );

    let app = Router::new();
    let app = routes::status::routes(app);
    let app = routes::state::routes(app);
    let app = routes::scan_state::routes(app);
    let app = routes::snark_pool::routes(app);
    let app = routes::snarker::routes(app);
    let app = routes::transaction::routes(app);
    let app = routes::discovery::routes(app);
    #[cfg(feature = "p2p-webrtc")]
    let app = routes::webrtc::routes(app);

    let app = app.layer(trace_layer).layer(cors_layer()).with_state(state);

    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!(port, "HTTP server listening");
    axum::serve(listener, app).await
}
