//! Axum-based HTTP server for the Mina node RPC API.
//!
//! This module provides REST endpoints for node status, state inspection,
//! snark pool management, and transaction handling.

#[macro_use]
mod macros;
mod openapi;
mod routes;
mod types;

pub use routes::openapi_spec;
pub use types::{AppError, AppResult, AppState, JsonErrorResponse};

use mina_node_common::rpc::RpcSender;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use types::cors_layer;

#[cfg(feature = "swagger-ui")]
use utoipa_swagger_ui::SwaggerUi;

#[cfg(feature = "scalar")]
use utoipa_scalar::{Scalar, Servable};

/// Runs the HTTP server on the specified port.
///
/// Returns an error if binding to the port fails or the server encounters an I/O error.
pub async fn run(port: u16, rpc_sender: RpcSender) -> std::io::Result<()> {
    let state = AppState::new(rpc_sender);

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &axum::http::Request<_>| {
            tracing::info_span!(
                "http_request",
                method = %request.method(),
                uri = %request.uri(),
            )
        })
        .on_response(
            |response: &axum::http::Response<_>, latency, span: &tracing::Span| {
                let status = response.status();
                if status.is_server_error() || status.is_client_error() {
                    tracing::error!(parent: span, status = %status, latency = ?latency, "request failed");
                } else {
                    tracing::info!(parent: span, status = %status, latency = ?latency, "request completed");
                }
            },
        );

    // Split to get Router and OpenApi spec
    let (app, api) = routes::openapi_router().split_for_parts();

    // GraphQL (not documented in OpenAPI)
    let app = routes::graphql::routes(app);

    // OpenAPI documentation UIs
    #[cfg(feature = "swagger-ui")]
    let app = app
        .merge(SwaggerUi::new("/api-docs/swagger-ui").url("/api-docs/openapi.json", api.clone()));

    #[cfg(feature = "scalar")]
    let app = app.merge(Scalar::with_url("/api-docs/scalar", api));

    #[cfg(feature = "stoplight-elements")]
    let app = app.route(
        "/api-docs/stoplight",
        axum::routing::get(openapi::stoplight_elements),
    );

    let app = app.layer(trace_layer).layer(cors_layer()).with_state(state);

    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!(port, "HTTP server listening");
    axum::serve(listener, app).await
}
